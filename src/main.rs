use crate::{
    parameters::{MotorPort, SensorPort, Stop},
    pupdevices::{GyroSensor, Motor},
    robotics::DriveBase,
};

mod attribute;
mod enum_string;
mod error;
mod motor_driver;
mod parameters;
mod pid;
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

    let mut drive = DriveBase::new(&motor, &motor2, 62.4, 130.5)?.with_gyro(&gyro)?;

    drive.use_gyro(true)?;
    drive.set_turn_speed(400);
    drive.find_calibrated_axle_track(50).await?;
    Ok(())
}
