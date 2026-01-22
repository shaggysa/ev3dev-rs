use crate::Ev3Error;
use crate::pid::Pid;
use crate::pupdevices::GyroSensor;
use crate::robotics::GyroController;
use crate::{Ev3Result, parameters::Stop, pupdevices::Motor};
use fixed::traits::{LossyInto, ToFixed};
use fixed::types::I32F32;
use scopeguard::defer;
use std::cell::Cell;
use std::time::Duration;
use tokio::time::interval;

pub struct DriveBase<'a> {
    left_motor: &'a Motor,
    right_motor: &'a Motor,
    left_start_angle: i32,
    right_start_angle: i32,
    min_speed: I32F32,
    wheel_diameter: I32F32,
    axle_track: I32F32,
    straight_speed: Cell<I32F32>,
    turn_speed: Cell<I32F32>,
    distance_pid: Pid,
    heading_pid: Pid,
    distance_target: Cell<I32F32>,
    heading_target: Cell<I32F32>,
    distance_tolerance: Cell<I32F32>,
    heading_tolerance: Cell<I32F32>,
    using_gyros: Cell<bool>,
    gyros: Option<GyroController<'a>>,
}

impl<'a> DriveBase<'a> {
    pub fn new<Number>(
        left_motor: &'a Motor,
        right_motor: &'a Motor,
        wheel_diameter: Number,
        axle_track: Number,
    ) -> Ev3Result<Self>
    where
        Number: ToFixed,
    {
        left_motor.set_ramp_up_setpoint(5000)?;
        right_motor.set_ramp_up_setpoint(5000)?;

        left_motor.set_ramp_down_setpoint(1800)?;
        right_motor.set_ramp_down_setpoint(1800)?;

        Ok(Self {
            left_motor,
            right_motor,
            left_start_angle: left_motor.angle()?,
            right_start_angle: right_motor.angle()?,
            min_speed: I32F32::from_num(65),
            wheel_diameter: I32F32::from_num(wheel_diameter),
            axle_track: I32F32::from_num(axle_track),
            straight_speed: Cell::new(I32F32::from_num(500)),
            turn_speed: Cell::new(I32F32::from_num(550)),
            distance_pid: Pid::new(10, 0, 8, 0, 0),
            heading_pid: Pid::new(10, 0, 5, 0, 0),
            distance_target: Cell::new(I32F32::from_num(0)),
            heading_target: Cell::new(I32F32::from_num(0)),
            distance_tolerance: Cell::new(I32F32::from_num(4)),
            heading_tolerance: Cell::new(I32F32::from_num(0.5)),
            using_gyros: Cell::new(false),
            gyros: None,
        })
    }

    /// Adds a single gyro sensor to the drive base.
    pub fn with_gyro<'b>(mut self, gyro_sensor: &'b GyroSensor) -> Ev3Result<Self>
    where
        'b: 'a,
    {
        self.gyros = Some(GyroController::new(vec![gyro_sensor])?);
        Ok(self)
    }

    /// Adds multiple gyro sensors to the drive base.
    pub fn with_gyros<'b>(mut self, gyro_sensors: Vec<&'b GyroSensor>) -> Ev3Result<Self>
    where
        'b: 'a,
    {
        self.gyros = Some(GyroController::new(gyro_sensors)?);
        Ok(self)
    }

    pub fn use_gyro(&self, use_gyro: bool) -> Ev3Result<()> {
        if use_gyro && self.gyros.is_none() {
            return Err(Ev3Error::NoSensorProvided);
        }
        self.using_gyros.set(use_gyro);
        Ok(())
    }

    pub fn set_straight_speed<Number>(&self, straight_speed: Number)
    where
        Number: ToFixed,
    {
        self.straight_speed.set(I32F32::from_num(straight_speed));
    }

    pub fn set_turn_speed<Number>(&self, turn_speed: Number)
    where
        Number: ToFixed,
    {
        self.turn_speed.set(I32F32::from_num(turn_speed));
    }

    pub fn set_stop_action(&self, action: Stop) -> Ev3Result<()> {
        self.left_motor.set_stop_action(action)?;
        self.right_motor.set_stop_action(action)
    }

    pub fn distance_pid_settings<Number>(
        &self,
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) where
        Number: ToFixed,
    {
        self.distance_pid
            .settings(kp, ki, kd, integral_deadzone, integral_rate);
    }

    pub fn heading_pid_settings<Number>(
        &self,
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) where
        Number: ToFixed,
    {
        self.heading_pid
            .settings(kp, ki, kd, integral_deadzone, integral_rate);
    }

    pub fn stop(&self) -> Ev3Result<()> {
        self.left_motor.stop_prev_action()?;
        self.right_motor.stop_prev_action()
    }

    async fn drive_relative(&self, distance_mm: I32F32, angle_deg: I32F32) -> Ev3Result<()> {
        defer! {
            _ = self.stop()
        }

        self.distance_pid.reset();
        self.heading_pid.reset();

        let target_distance = self.distance_target.get() + distance_mm;
        let target_heading = self.heading_target.get() + angle_deg;

        self.distance_target.set(target_distance);
        self.heading_target.set(target_heading);

        let mut timer = interval(Duration::from_millis(5));

        // the first tick completes immediately
        timer.tick().await;

        let straight_speed = self.straight_speed.get();
        let turn_speed = self.turn_speed.get();

        loop {
            let left_angle = I32F32::from_num(self.left_motor.angle()? - self.left_start_angle);
            let right_angle = I32F32::from_num(self.right_motor.angle()? - self.right_start_angle);
            let current_distance = self.encoders_to_distance(left_angle, right_angle);
            let current_heading = if self.using_gyros.get()
                && let Some(ref gyro) = self.gyros
            {
                let encoders = self.encoders_to_heading(left_angle, right_angle);
                I32F32::from_num(gyro.heading()?) * I32F32::from_num(0.8)
                    + encoders * I32F32::from_num(0.2)
            } else {
                self.encoders_to_heading(left_angle, right_angle)
            };

            let distance_error = target_distance - current_distance;
            let heading_error = target_heading - current_heading;

            if distance_error.abs() < self.distance_tolerance.get()
                && heading_error.abs() < self.heading_tolerance.get()
            {
                break;
            }

            let dive_effort = self.distance_pid.next(distance_error);
            let turn_effort = -self.heading_pid.next(heading_error);

            let drive_speed_out = dive_effort * straight_speed;
            let turn_speed_out = turn_effort * turn_speed;

            let left_speed = (drive_speed_out - turn_speed_out)
                .clamp(-self.right_motor.max_speed, self.left_motor.max_speed);

            let right_speed = (drive_speed_out + turn_speed_out)
                .clamp(-self.left_motor.max_speed, self.right_motor.max_speed);

            self.left_motor.run(
                (if left_speed.abs() < self.min_speed {
                    self.min_speed * left_speed.signum()
                } else {
                    left_speed
                })
                .lossy_into(),
            )?;
            self.right_motor.run(
                (if right_speed.abs() < self.min_speed {
                    self.min_speed * right_speed.signum()
                } else {
                    right_speed
                })
                .lossy_into(),
            )?;

            timer.tick().await;
        }

        Ok(())
    }

    pub async fn straight<Number>(&self, distance: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        self.drive_relative(I32F32::from_num(distance), I32F32::from_num(0))
            .await
    }

    pub async fn turn<Number>(&self, angle: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        self.drive_relative(I32F32::from_num(0), I32F32::from_num(angle))
            .await
    }

    pub async fn curve<Number>(&self, radius: Number, angle: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        let fixed_angle = I32F32::from_num(angle);

        let angle_rad = fixed_angle * I32F32::PI / 180;
        let arc_length = I32F32::from_num(radius).abs() * I32F32::from_num(angle_rad).abs();

        self.drive_relative(arc_length, I32F32::from_num(fixed_angle))
            .await
    }

    pub async fn veer<Number>(&self, radius: Number, distance: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        let fixed_distance = I32F32::from_num(distance);

        let angle_rad = fixed_distance / I32F32::from_num(radius);
        let angle_deg = angle_rad * 180 / I32F32::PI;

        self.drive_relative(fixed_distance, I32F32::from_num(angle_deg))
            .await
    }

    pub async fn find_calibrated_axle_track<Number>(
        &mut self,
        margin_of_error: Number,
    ) -> Ev3Result<I32F32>
    where
        Number: ToFixed,
    {
        if let Some(ref gyros) = self.gyros {
            self.use_gyro(true)?;

            let fixed_estimate = I32F32::from_num(self.axle_track);

            let fixed_margin_of_error = I32F32::from_num(margin_of_error);

            let mut min = fixed_estimate - fixed_margin_of_error;
            let mut max = fixed_estimate + fixed_margin_of_error;

            let mut last_gyro_head = I32F32::from_num(0);
            let mut last_encoder_head = I32F32::from_num(0);

            loop {
                let mid = (min + max) / 2;
                println!("trying {}", mid);
                self.axle_track = mid;
                self.turn(90).await?;

                let gyro_head = gyros.heading()?;
                let encoder_head = self.encoders_to_heading(
                    I32F32::from_num(self.left_motor.angle()? - self.left_start_angle),
                    I32F32::from_num(self.right_motor.angle()? - self.right_start_angle),
                );

                println!(
                    "gyro: {}, encoder: {}",
                    gyro_head - last_gyro_head,
                    encoder_head - last_encoder_head
                );

                if ((gyro_head - last_gyro_head) - (encoder_head - last_encoder_head)).abs() < 0.25
                {
                    break;
                }

                if gyro_head - last_gyro_head < encoder_head - last_encoder_head {
                    min = mid;
                } else {
                    max = mid;
                }
                last_gyro_head = gyro_head;
                last_encoder_head = encoder_head;
            }

            let final_val = (min + max) / 2;
            println!("Final value: {}", final_val);
            Ok(final_val)
        } else {
            Err(Ev3Error::NoSensorProvided)
        }
    }

    // Convert encoder positions to distance traveled (average of both wheels)
    fn encoders_to_distance(&self, left_deg: I32F32, right_deg: I32F32) -> I32F32 {
        let wheel_circ = I32F32::PI * self.wheel_diameter;
        let left_mm = wheel_circ * left_deg / 360;
        let right_mm = wheel_circ * right_deg / 360;
        (left_mm + right_mm) / 2
    }

    /// Convert encoder positions to heading (differential between wheels)
    fn encoders_to_heading(&self, left_deg: I32F32, right_deg: I32F32) -> I32F32 {
        let wheel_circ = I32F32::PI * self.wheel_diameter;
        let left_mm = wheel_circ * left_deg / 360;
        let right_mm = wheel_circ * right_deg / 360;
        let arc_diff = left_mm - right_mm;
        let turn_rad = arc_diff / self.axle_track;
        turn_rad * 180 / I32F32::PI
    }
}
