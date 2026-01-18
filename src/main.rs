use std::time::Duration;

use smol::Timer;

use crate::{
    motor_driver::MotorDriver,
    parameters::{MotorPort, SensorPort, Stop},
    pupdevices::{ColorSensor, GyroSensor, InfraredSensor, Motor, TouchSensor, UltrasonicSensor},
    robotics::{DriveBase, GyroController},
    tools::wait,
};

mod attribute;
mod enum_string;
mod error;
mod motor_driver;
mod parameters;
pub mod pupdevices;
pub mod robotics;
mod sensor_driver;
pub mod tools;

pub use error::{Ev3Error, Ev3Result};

use attribute::AttributeName;
use parameters::Direction;

#[tokio::main]
async fn main() -> Ev3Result<()> {
    let motor = Motor::new(MotorPort::OutA, Direction::CounterClockwise)?;
    let motor2 = Motor::new(MotorPort::OutD, Direction::CounterClockwise)?;
    let gyro = GyroSensor::new(SensorPort::In3)?;

    let controller = GyroController::new(vec![gyro])?;
    let drive = DriveBase::new(&motor, &motor2, 60.0, 140.0).with_gyro(&controller);
    drive.use_gyro(true);
    drive.set_stop_option(Stop::Hold)?;
    drive.set_turn_speed(350);

    drive.straight(10000).await?;

    Ok(())
}
