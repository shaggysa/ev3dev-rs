use ev3dev_lang_rust::Ev3Result;
mod parameters;
mod pupdevices;
mod robotics;
mod tools;

use parameters::{Direction, MotorPort, SensorPort, Stop};
use pupdevices::{GyroSensor, Motor};
use robotics::DriveBase;

#[tokio::main]
async fn main() -> Ev3Result<()> {
    let left_motor = Motor::new(MotorPort::OutA, Direction::CounterClockWise)?;
    let right_motor = Motor::new(MotorPort::OutD, Direction::CounterClockWise)?;

    left_motor.set_stop_action(Stop::Hold)?;
    right_motor.set_stop_action(Stop::Hold)?;

    let gyro_1 = GyroSensor::new(SensorPort::In2)?;
    let gyro_2 = GyroSensor::new(SensorPort::In3)?;

    let mut drive = DriveBase::new(left_motor, right_motor, 72.0, 128.0);

    drive.add_gyro(gyro_1)?;
    drive.add_gyro(gyro_2)?;

    drive.set_stop_option(Stop::Hold)?;

    drive.use_gyro(false);

    drive.straight(750).await?;

    drive.straight(-750).await?;

    Ok(())
}
