use crate::tools::wait;
use std::f32::consts::PI;
use std::time::Duration;

use ev3dev_lang_rust::Ev3Result;
use scopeguard::defer;

use crate::{
    parameters::Stop,
    pupdevices::{GyroSensor, Motor},
};
use pid::Pid;

pub struct DriveBase {
    left_motor: Motor,
    right_motor: Motor,
    wheel_diameter: f32,
    axle_track: f32,
    speed: i32,
    motor_pid: Pid<f32>,
    gyro_pid: Pid<f32>,
    heading_target: f32,
    using_gyros: bool,
    gyros: Vec<(GyroSensor, i32)>,
}

impl DriveBase {
    pub fn new(
        left_motor: Motor,
        right_motor: Motor,
        wheel_diameter: f32,
        axle_track: f32,
    ) -> Self {
        let correction_max = 300.0 / 2.0;
        Self {
            left_motor,
            right_motor,
            wheel_diameter,
            axle_track,
            speed: 300,
            motor_pid: *Pid::new(0.0, correction_max)
                .p(2.0, correction_max)
                .d(8.0, correction_max),
            gyro_pid: *Pid::new(0.0, correction_max)
                .p(20.0, correction_max)
                .d(12.0, correction_max),
            heading_target: 0.0,
            using_gyros: false,
            gyros: Vec::new(),
        }
    }

    pub fn add_gyro(&mut self, gyro: GyroSensor) -> Ev3Result<()> {
        let heading = gyro.heading()?;
        self.gyros.push((gyro, heading));
        Ok(())
    }

    pub fn use_gyro(&mut self, use_gyro: bool) {
        self.using_gyros = use_gyro;
    }

    pub fn set_stop_option(&self, action: Stop) -> Ev3Result<()> {
        self.left_motor.set_stop_action(action)?;
        self.right_motor.set_stop_action(action)
    }

    pub fn stop(&self) -> Ev3Result<()> {
        self.left_motor.stop()?;
        self.right_motor.stop()
    }

    pub async fn straight(&mut self, distance_mm: i32) -> Ev3Result<()> {
        // bind borrowed motors to variables to prevent the guard from crating a multi-borrow scenario
        let l = &self.left_motor;
        let r = &self.right_motor;

        defer! {
            _ = l.stop();
            _ = r.stop();
        }

        let left_start_angle = self.left_motor.get_angle()?;
        let right_start_angle = self.right_motor.get_angle()?;

        let distance_target = distance_mm as f32 * (2.0 * self.wheel_diameter * PI) / 360.0;

        let speed;

        if distance_mm > 0 {
            speed = self.speed;
        } else {
            speed = -self.speed;
        }

        if distance_mm < 0 {
            while (self.left_motor.get_angle()? - left_start_angle
                + self.right_motor.get_angle()?
                - right_start_angle)
                / 2.0
                > distance_target
            {
                let left_angle = self.left_motor.get_angle()? - left_start_angle;
                let right_angle = self.right_motor.get_angle()? - right_start_angle;

                let motor_err = (left_angle - right_angle) as f32;

                let motor_correction = self.motor_pid.next_control_output(motor_err).output;

                let correction;

                if self.using_gyros == true {
                    let gyro_err = self
                        .gyros
                        .iter()
                        .map(|s| (s.0.heading().unwrap_or(0) - s.1) as f32)
                        .sum::<f32>()
                        / self.gyros.len() as f32
                        - self.heading_target;

                    let yaw_correction = self.gyro_pid.next_control_output(gyro_err).output;

                    correction = (motor_correction * 0.15 + yaw_correction * 0.85) as i32;
                } else {
                    correction = motor_correction as i32;
                }

                self.left_motor.motor.set_speed_sp(speed + correction)?;
                self.right_motor.motor.set_speed_sp(speed - correction)?;

                self.left_motor.motor.run_forever()?;
                self.right_motor.motor.run_forever()?;
            }
        } else {
            while (self.left_motor.get_angle()? - left_start_angle
                + self.right_motor.get_angle()?
                - right_start_angle)
                / 2.0
                < distance_target
            {
                let left_angle = self.left_motor.get_angle()? - left_start_angle;
                let right_angle = self.right_motor.get_angle()? - right_start_angle;

                let motor_err = right_angle - left_angle;

                let motor_correction = self.motor_pid.next_control_output(motor_err).output;

                let correction;

                if self.using_gyros == true {
                    let gyro_err = self
                        .gyros
                        .iter()
                        .map(|s| (s.0.heading().unwrap_or(0) - s.1) as f32)
                        .sum::<f32>()
                        / self.gyros.len() as f32
                        - self.heading_target;

                    let yaw_correction = self.gyro_pid.next_control_output(gyro_err).output;

                    correction = (motor_correction * 0.15 + yaw_correction * 0.85) as i32;
                } else {
                    correction = motor_correction as i32
                }

                self.left_motor.motor.set_speed_sp(speed + correction)?;
                self.right_motor.motor.set_speed_sp(speed - correction)?;

                self.left_motor.motor.run_forever()?;
                self.right_motor.motor.run_forever()?;

                wait(Duration::from_millis(5)).await;
            }
        }
        Ok(())
    }
}
