#![feature(decl_macro)]
#![feature(macro_metavar_expr)]
#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Basic usage
//!
//! ```no_run
//! extern crate ev3dev_rs;
//! extern crate tokio;
//!
//! use ev3dev_rs::Ev3Result;
//! use ev3dev_rs::pupdevices::{GyroSensor, Motor, ColorSensor};
//! use ev3dev_rs::robotics::DriveBase;
//! use ev3dev_rs::parameters::{MotorPort, MotorPort, Direction}
//!
//! #[tokio::main]
//! async fn main() -> Ev3Result<()> {
//!
//!     use ev3dev_rs::parameters::{Direction, SensorPort};
//!     let left_motor = Motor::new(MotorPort::OutA, Direction::Clockwise)?;
//!     let right_motor = Motor::new(MotorPort::OutD, Direction::Clockwise)?;
//!
//!     let gyro_sensor = GyroSensor::new(SensorPort::In1)?;
//!     let color_sensor = ColorSensor::new(SensorPort::In4)?;
//!
//!     println!("Detected color: {}", color_sensor.color()?);
//!
//!     let drive = DriveBase::new(&left_motor, &right_motor, 62.4, 130.5)?.with_gyro(&gyro_sensor)?;
//!
//!     drive.use_gyro(true)?;
//!
//!     drive.straight(500).await?;
//!     drive.turn(90).await?;
//!     drive.curve(600, 90).await?;
//!     drive.veer(600, 500).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # `DriveBase` calibration
//!
//! ```no_run
//! extern crate ev3dev_rs;
//! extern crate tokio;
//!
//! use ev3dev_rs::pupdevices::{GyroSensor, Motor, ColorSensor};
//! use ev3dev_rs::robotics::DriveBase;
//! use ev3dev_rs::parameters::{MotorPort, MotorPort}
//!
//! #[tokio::main]
//! async fn main() -> Ev3Result<()> {
//!
//!     use ev3dev_rs::parameters::{Direction, SensorPort};
//!     let left_motor = Motor::new(MotorPort::OutA, Direction::Clockwise)?;
//!     let right_motor = Motor::new(MotorPort::OutD, Direction::Clockwise)?;
//!
//!     // A gyro sensor is required for calibration.
//!     let gyro_sensor = GyroSensor::new(SensorPort::In1)?;
//!
//!     // find_calibrated_axle_track requires a mutable reference
//!     let mut drive = DriveBase::new(&left, &right, 62.4, 130.5)?.with_gyro(&gyro)?;
//!
//!     drive.use_gyro(true)?;
//!
//!     // This will test a bunch of axle tracks, compare them with the gyro, and report the optimal value.
//!     // Note that it is highly experimental and different surfaces may heavily affect the reported value.
//!     // If you have issues with drive wheel slippage, please see "set_ramp_up_setpoint".
//!     // Even if you perfectly dial this in, using the gyro is still highly recommended to get the most accurate readings.
//!     drive.find_calibrated_axle_track(50).await?;
//!
//!     Ok(())
//! }
//! ```
//! # Async functions
//!
//! Because all the `Motor` and `DriveBase` actions are async, you can run them simultaneously.
//!
//! ```no_run
//! extern crate ev3dev_rs;
//! extern crate tokio;
//!
//! use ev3dev_rs::pupdevices::{GyroSensor, Motor, ColorSensor};
//! use ev3dev_rs::robotics::DriveBase;
//! use ev3dev_rs::parameters::{MotorPort, MotorPort}
//! use ev3dev_rs::tools;
//!
//! #[tokio::main]
//! async fn main() -> Ev3Result<()> {
//!
//!     use ev3dev_rs::parameters::{Direction, SensorPort};
//!     let left_motor = Motor::new(MotorPort::OutA, Direction::Clockwise)?;
//!     let right_motor = Motor::new(MotorPort::OutD, Direction::Clockwise)?;
//!
//!     let attachment_motor = Motor::new(MotorPort::OutB, Direction::Clockwise)?;
//!
//!     let gyro_sensor = GyroSensor::new(SensorPort::In1)?;
//!
//!     // find_calibrated_axle_track requires a mutable reference
//!     let drive = DriveBase::new(&left, &right, 62.4, 130.5)?.with_gyro(&gyro)?;
//!
//!     drive.use_gyro(true)?;
//!
//!     // join is like pybricks' non-racing multitask
//!     // it will wait for all actions to complete before moving on
//!     // if either task returns an error, it will return that error
//!     tools::join!(drive.straight(100), attachment_motor.run_until_stalled(-45))?;
//!
//!     // select is like pybricks' racing multitask
//!     // once one action completes, the other will be canceled
//!     tools::select!(drive.turn(90), attachment_motor.run_until_stalled(45))?
//!
//!     Ok(())
//! }
//! ```

mod attribute;
mod enum_string;
mod error;
mod motor_driver;
/// Parameters used in the ev3dev_rs crate.
pub mod parameters;
mod pid;
/// Devices that can connect to the robot.
pub mod pupdevices;
/// Higher level abstractions.
pub mod robotics;
mod sensor_driver;
/// Additional tools.
pub mod tools;

pub use error::{Ev3Error, Ev3Result};
