use crate::Ev3Error;
use crate::{robotics::GyroController, tools::wait};
use scopeguard::defer;
use smol::Timer;
use smol::stream::StreamExt;
use std::cell::{Cell, RefCell};
use std::cmp::max;
use std::f32::consts::PI;
use std::time::Duration;

use crate::{Ev3Result, parameters::Stop, pupdevices::Motor};
use pid::Pid;

pub struct DriveBase<'a> {
    left_motor: &'a Motor,
    right_motor: &'a Motor,
    wheel_diameter: f32,
    axle_track: f32,
    straight_speed: Cell<i32>,
    turn_speed: Cell<i32>,
    motor_pid: RefCell<Pid<f32>>,
    gyro_pid: RefCell<Pid<f32>>,
    heading_target: Cell<f32>,
    using_gyros: Cell<bool>,
    gyros: Option<&'a GyroController>,
}

impl<'a> DriveBase<'a> {
    pub fn new(
        left_motor: &'a Motor,
        right_motor: &'a Motor,
        wheel_diameter: f32,
        axle_track: f32,
    ) -> Self {
        let correction_max = 500.0;
        Self {
            left_motor,
            right_motor,
            wheel_diameter,
            axle_track,
            straight_speed: Cell::new(300),
            turn_speed: Cell::new(300),
            motor_pid: RefCell::new(
                *Pid::new(0.0, correction_max)
                    .p(10.0, correction_max)
                    .d(8.0, correction_max),
            ),
            gyro_pid: RefCell::new(
                *Pid::new(0.0, correction_max)
                    .p(10.0, correction_max)
                    .d(5.0, correction_max),
            ),
            heading_target: Cell::new(0.0),
            using_gyros: Cell::new(false),
            gyros: None,
        }
    }

    pub fn with_gyro(self, controller: &'a GyroController) -> Self {
        Self {
            left_motor: self.left_motor,
            right_motor: self.right_motor,
            wheel_diameter: self.wheel_diameter,
            axle_track: self.axle_track,
            straight_speed: self.straight_speed,
            turn_speed: self.turn_speed,
            motor_pid: self.motor_pid,
            gyro_pid: self.gyro_pid,
            heading_target: self.heading_target,
            using_gyros: self.using_gyros,
            gyros: Some(controller),
        }
    }

    pub fn use_gyro(&self, use_gyro: bool) {
        self.using_gyros.set(use_gyro);
    }

    pub fn set_straight_speed(&self, straight_speed: i32) {
        self.straight_speed.set(straight_speed);
    }

    pub fn set_turn_speed(&self, turn_speed: i32) {
        self.turn_speed.set(turn_speed);
    }

    pub fn set_stop_option(&self, action: Stop) -> Ev3Result<()> {
        self.left_motor.set_stop_action(action)?;
        self.right_motor.set_stop_action(action)
    }

    pub fn stop(&self) -> Ev3Result<()> {
        self.left_motor.stop()?;
        self.right_motor.stop()
    }

    pub async fn straight(&self, distance_mm: i32) -> Ev3Result<()> {
        // bind borrowed motors to variables for the guard

        defer! {
            _ = self.left_motor.stop();
            _ = self.right_motor.stop();
        }

        let left_start_angle = self.left_motor.angle()?;
        let right_start_angle = self.right_motor.angle()?;

        let distance_target = distance_mm as f32 * (2.0 * self.wheel_diameter * PI) / 360.0;
        let heading_target = self.heading_target.get();

        let speed = if distance_mm > 0 {
            self.straight_speed.get()
        } else {
            -self.straight_speed.get()
        };

        let mut timer = Timer::interval(Duration::from_millis(5));

        if distance_mm < 0 {
            while (self.left_motor.angle()? - left_start_angle + self.right_motor.angle()?
                - right_start_angle) as f32
                / 2.0
                > distance_target
            {
                let left_angle = self.left_motor.angle()? - left_start_angle;
                let right_angle = self.right_motor.angle()? - right_start_angle;

                let motor_err = left_angle - right_angle;

                let motor_correction = self
                    .motor_pid
                    .borrow_mut()
                    .next_control_output(motor_err as f32)
                    .output;

                let correction;

                if self.using_gyros.get()
                    && let Some(gyros) = &self.gyros
                {
                    let gyro_err = gyros.heading()? - heading_target;

                    let yaw_correction = self
                        .gyro_pid
                        .borrow_mut()
                        .next_control_output(gyro_err)
                        .output;

                    correction = (motor_correction * 0.15 + yaw_correction * 0.85) as i32;
                } else {
                    correction = motor_correction as i32;
                }

                self.left_motor.run(speed + correction)?;

                self.right_motor.run(speed - correction)?;

                timer.next().await;
            }
        } else {
            while (self.left_motor.angle()? - left_start_angle + self.right_motor.angle()?
                - right_start_angle) as f32
                / 2.0
                < distance_target
            {
                let left_angle = self.left_motor.angle()? - left_start_angle;
                let right_angle = self.right_motor.angle()? - right_start_angle;

                let motor_err = left_angle - right_angle;

                let motor_correction = self
                    .motor_pid
                    .borrow_mut()
                    .next_control_output(motor_err as f32)
                    .output;

                let correction;

                if self.using_gyros.get()
                    && let Some(gyros) = &self.gyros
                {
                    let gyro_err = gyros.heading()? - heading_target;

                    let yaw_correction = self
                        .gyro_pid
                        .borrow_mut()
                        .next_control_output(gyro_err)
                        .output;

                    correction = (motor_correction * 0.15 + yaw_correction * 0.85) as i32;
                } else {
                    correction = motor_correction as i32
                }

                self.left_motor.run(speed + correction)?;
                self.right_motor.run(speed - correction)?;

                timer.next().await;
            }
        }
        Ok(())
    }

    pub async fn turn(&self, angle: f32) -> Ev3Result<()> {
        self.heading_target.update(|target| target + angle);

        defer! {
            _ = self.left_motor.stop();
            _ = self.right_motor.stop();
        }

        let left_start_angle = self.left_motor.angle()?;
        let right_start_angle = self.right_motor.angle()?;

        if self.using_gyros.get()
            && let Some(gyros) = &self.gyros
        {
            let start_heading = gyros.heading()?;
            let target_heading = self.heading_target.get();

            loop {
                let current_heading = gyros.heading()?;
                let remaining = target_heading - current_heading;
                let abs_remaining = remaining.abs();

                if abs_remaining < 1.0 {
                    break;
                }

                let speed = if abs_remaining <= 35.0 {
                    max(
                        (self.turn_speed.get() as f32 * (abs_remaining / 35.0)) as i32,
                        100,
                    )
                } else {
                    self.turn_speed.get()
                };

                let direction = if remaining > 0.0 { 1 } else { -1 };

                let (left_duty, right_duty) = if direction > 0 {
                    (speed, -speed)
                } else {
                    (-speed, speed)
                };

                self.left_motor.run(left_duty)?;
                self.right_motor.run(right_duty)?;

                wait(Duration::from_millis(5)).await;
            }
        }
        Ok(())
    }
}
