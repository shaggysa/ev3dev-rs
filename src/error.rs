use crate::{
    attribute::FileMode,
    parameters::{MotorPort, SensorPort},
    sensor_driver::SensorType,
};
use std::num::ParseFloatError;
use std::{error::Error, fmt::Display, num::ParseIntError, path::PathBuf};

#[derive(Debug)]
#[allow(private_interfaces)]
/// The error type for the ev3dev_rs crate
pub enum Ev3Error {
    /// Unable to find a sensor.
    SensorNotFound {
        ///  The port the sensor was supposed to be on.
        port: SensorPort,
        /// The expected sensor type.
        expected_sensor_type: SensorType,
    },
    /// Unable to find a motor.
    MotorNotFound {
        /// The port the motor was supposed to be on.
        port: MotorPort,
    },
    /// An internal error
    FileNotFound {
        /// The name of the expected file
        path: PathBuf,
    },
    /// An internal error
    InvalidPath,
    /// Found the incorrect sensor
    IncorrectSensorType {
        /// The type of the expected sensor.
        expected: SensorType,
        /// The type of the found sensor
        found: SensorType,
    },
    /// Failed to parse a string into an enum variant.
    ParseStr {
        /// The string that was unable to be parsed.
        input: String,
        /// The name of the enum
        to: String,
    },
    /// An internal error
    PermissionDenied {
        /// The permission that was expected
        required_permission: FileMode,
    },
    /// Failed to read the contents of a sysfs file to a string.
    InvalidStringBytes,
    /// Unable to read a motor or sensor attribute.
    ReadAttributeFailure {
        /// The name of the file that failed to read
        filename: PathBuf,
        /// The raw OS error.
        os_error: std::io::Error,
    },
    /// Failed to write to an attribute of a sensor or motor.
    WriteAttributeFailure {
        /// The name of the file that we failed to write to.
        filename: PathBuf,
        /// The value that we tried to write.
        value: String,
        /// The raw OS error.
        os_error: std::io::Error,
    },
    /// Failed to parse an integer
    ParseInt {
        /// The raw error
        err: ParseIntError,
    },
    /// Failed to parse a floating point number.
    ParseFloat {
        /// The raw error
        err: ParseFloatError,
    },

    /// Failed to read a string into an enum variant.
    InvalidValue {
        /// The function that tried to read the value.
        func: String,
        /// The value.
        value: String,
    },
    /// No sensor was provided to something that needed it.
    ///
    /// This was most likely caused by a `DriveBase` not being provided a `GyroSensor`.
    NoSensorProvided,
}

impl Display for Ev3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for Ev3Error {}

impl From<ParseIntError> for Ev3Error {
    fn from(err: ParseIntError) -> Self {
        Ev3Error::ParseInt { err }
    }
}

impl From<ParseFloatError> for Ev3Error {
    fn from(err: ParseFloatError) -> Self {
        Ev3Error::ParseFloat { err }
    }
}

/// The result type for the ev3dev_rs crate.
pub type Ev3Result<T> = Result<T, Ev3Error>;
