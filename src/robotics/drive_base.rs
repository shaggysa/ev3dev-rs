use crate::Ev3Error;
use crate::pid::Pid;
use crate::pupdevices::GyroSensor;
use crate::robotics::GyroController;
use crate::{Ev3Result, parameters::Stop, pupdevices::Motor};
use fixed::traits::{LossyInto, ToFixed};
use fixed::types::I32F32;
use scopeguard::defer;
use smol::Timer;
use smol::stream::StreamExt;
use std::cell::Cell;
use std::time::Duration;

const PI: I32F32 = I32F32::PI;

pub struct DriveBase<'a> {
    left_motor: &'a Motor,
    right_motor: &'a Motor,
    wheel_diameter: I32F32,
    axle_track: I32F32,
    straight_speed: Cell<i32>,
    turn_speed: Cell<i32>,
    distance_pid: Pid,
    heading_pid: Pid,
    heading_target: Cell<I32F32>,
    distance_tolerance: Cell<I32F32>,
    heading_tolerance: Cell<I32F32>,
    using_gyros: Cell<bool>,
    gyros: Option<GyroController<'a>>,
    max_speed: I32F32,
}

impl<'a> DriveBase<'a> {
    pub fn new<Number>(
        left_motor: &'a Motor,
        right_motor: &'a Motor,
        wheel_diameter: Number,
        axle_track: Number,
    ) -> Self
    where
        Number: ToFixed,
    {
        Self {
            left_motor,
            right_motor,
            wheel_diameter: I32F32::from_num(wheel_diameter),
            axle_track: I32F32::from_num(axle_track),
            straight_speed: Cell::new(300),
            turn_speed: Cell::new(300),
            distance_pid: Pid::new(10, 0, 8, 0, 0),
            heading_pid: Pid::new(10, 0, 5, 0, 0),
            heading_target: Cell::new(I32F32::from_num(0)),
            distance_tolerance: Cell::new(I32F32::from_num(4)),
            heading_tolerance: Cell::new(I32F32::from_num(0.5)),
            using_gyros: Cell::new(false),
            gyros: None,
            max_speed: I32F32::from_num(1000),
        }
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

    pub fn set_straight_speed(&self, straight_speed: i32) {
        self.straight_speed.set(straight_speed);
    }

    pub fn set_turn_speed(&self, turn_speed: i32) {
        self.turn_speed.set(turn_speed);
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

    async fn drive_relative(
        &self,
        distance_mm: I32F32,
        drive_speed: I32F32,
        angle_deg: I32F32,
        turn_speed: I32F32,
    ) -> Ev3Result<()> {
        defer! {
            _ = self.stop()
        }

        self.distance_pid.reset();
        self.heading_pid.reset();

        let left_angle = I32F32::from_num(self.left_motor.angle()?);
        let right_angle = I32F32::from_num(self.right_motor.angle()?);

        let current_distance = self.encoders_to_distance(left_angle, right_angle);

        let current_heading = if self.using_gyros.get()
            && let Some(ref gyro) = self.gyros
        {
            I32F32::from_num(gyro.heading()?) * I32F32::from_num(0.8)
                + self.encoders_to_heading(left_angle, right_angle) * I32F32::from_num(0.2)
        } else {
            self.encoders_to_heading(left_angle, right_angle)
        };

        let target_distance = current_distance + distance_mm;
        let target_heading = current_heading + angle_deg;

        let mut timer = Timer::interval(Duration::from_millis(5));

        loop {
            let left_angle = I32F32::from_num(self.left_motor.angle()?);
            let right_angle = I32F32::from_num(self.right_motor.angle()?);
            let current_distance = self.encoders_to_distance(left_angle, right_angle);
            let current_heading = if self.using_gyros.get()
                && let Some(ref gyro) = self.gyros
            {
                I32F32::from_num(gyro.heading()?) * I32F32::from_num(0.8)
                    + self.encoders_to_heading(left_angle, right_angle) * I32F32::from_num(0.2)
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

            let drive_speed_out = self.distance_pid.next(distance_error);
            let turn_speed_out = self.heading_pid.next(heading_error);

            self.left_motor.run(
                self.clamp_speed(drive_speed_out - turn_speed_out)
                    .lossy_into(),
            )?;
            self.right_motor.run(
                self.clamp_speed(drive_speed_out + turn_speed_out)
                    .lossy_into(),
            )?;

            timer.next().await;
        }

        Ok(())
    }

    pub async fn straight<Number>(&self, distance: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        self.drive_relative(
            I32F32::from_num(distance),
            I32F32::from_num(0),
            I32F32::from_num(0),
            I32F32::from_num(0),
        )
        .await
    }

    pub async fn turn<Number>(&self, angle: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        self.drive_relative(
            I32F32::from_num(0),
            I32F32::from_num(0),
            I32F32::from_num(angle),
            I32F32::from_num(0),
        )
        .await
    }

    pub async fn curve<Number>(&self, radius: Number, angle: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        let fixed_angle = I32F32::from_num(angle);

        let angle_rad = fixed_angle * I32F32::PI / 180;
        let arc_length = I32F32::from_num(radius).abs() * I32F32::from_num(angle_rad).abs();

        self.drive_relative(
            arc_length,
            I32F32::from_num(0),
            fixed_angle,
            I32F32::from_num(0),
        )
        .await
    }

    pub async fn veer<Number>(&self, radius: Number, distance: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        let fixed_distance = I32F32::from_num(distance);

        let angle_rad = fixed_distance / I32F32::from_num(radius);
        let angle_deg = angle_rad * 180 / I32F32::PI;

        self.drive_relative(
            fixed_distance,
            I32F32::from_num(0),
            angle_deg,
            I32F32::from_num(0),
        )
        .await
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
        let arc_diff = right_mm - left_mm;
        let turn_rad = arc_diff / self.axle_track;
        turn_rad * 180 / I32F32::PI
    }

    /// Clamp speed to max motor speed
    fn clamp_speed(&self, speed: I32F32) -> I32F32 {
        if speed > self.max_speed {
            self.max_speed
        } else if speed < -self.max_speed {
            -self.max_speed
        } else {
            speed
        }
    }
}
