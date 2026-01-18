use crate::Ev3Error;
use crate::pid::Pid;
use crate::pupdevices::GyroSensor;
use crate::robotics::GyroController;
use crate::{Ev3Result, parameters::Stop, pupdevices::Motor};
use fixed::traits::ToFixed;
use fixed::types::I32F32;
use std::cell::Cell;

const PI: I32F32 = I32F32::PI;

pub struct DriveBase<'a> {
    left_motor: &'a Motor,
    right_motor: &'a Motor,
    wheel_diameter: f32,
    axle_track: f32,
    straight_speed: Cell<i32>,
    turn_speed: Cell<i32>,
    motor_pid: Pid,
    gyro_pid: Pid,
    heading_target: Cell<f32>,
    using_gyros: Cell<bool>,
    gyros: Option<GyroController<'a>>,
}

impl<'a> DriveBase<'a> {
    pub fn new(
        left_motor: &'a Motor,
        right_motor: &'a Motor,
        wheel_diameter: f32,
        axle_track: f32,
    ) -> Self {
        Self {
            left_motor,
            right_motor,
            wheel_diameter,
            axle_track,
            straight_speed: Cell::new(300),
            turn_speed: Cell::new(300),
            motor_pid: Pid::new(10, 0, 8, 0, 0),
            gyro_pid: Pid::new(10, 0, 5, 0, 0),
            heading_target: Cell::new(0.0),
            using_gyros: Cell::new(false),
            gyros: None,
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

    pub fn motor_pid_settings<Number>(
        &self,
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) where
        Number: ToFixed,
    {
        self.motor_pid
            .settings(kp, ki, kd, integral_deadzone, integral_rate);
    }

    pub fn gyro_pid_settings<Number>(
        &self,
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) where
        Number: ToFixed,
    {
        self.gyro_pid
            .settings(kp, ki, kd, integral_deadzone, integral_rate);
    }

    pub fn stop(&self) -> Ev3Result<()> {
        self.left_motor.stop_prev_action()?;
        self.right_motor.stop_prev_action()
    }
}
