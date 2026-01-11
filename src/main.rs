use std::time::Duration;

use crate::{error::Ev3Result, parameters::SensorPort, sensors::GyroSensor};

mod attribute;
mod error;
mod motor_driver;
mod parameters;
mod sensor_driver;
mod sensors;

#[tokio::main]
async fn main() -> Ev3Result<()> {
    let s1 = GyroSensor::new(SensorPort::In2)?;

    println!("{}", s1.heading()?);

    println!("{}", s1.tilt()?);

    println!("{:?}", s1.heading_and_velocity()?);

    println!("{}", s1.tilt_velocity()?);

    Ok(())
}
