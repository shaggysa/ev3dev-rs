use smol::Timer;

use crate::{
    error::{Ev3Error, Ev3Result},
    parameters::SensorPort,
    pupdevices::{ColorSensor, GyroSensor, InfraredSensor, TouchSensor, UltrasonicSensor},
};

mod attribute;
mod enum_string;
mod error;
mod motor_driver;
mod parameters;
mod pupdevices;
mod sensor_driver;

#[tokio::main]
async fn main() -> Ev3Result<()> {
    let s1 = UltrasonicSensor::new(SensorPort::In4)?;
    let s2 = InfraredSensor::new(SensorPort::In2)?;

    loop {
        let (heading, distance) = s2.seek_channel_1()?;
        println!("heading: {:?} distance: {:?}", heading, distance);

        Timer::after(std::time::Duration::from_millis(100)).await;
    }
}
