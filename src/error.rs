use std::{
    error::Error,
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
};

use crate::{
    attribute::FileMode,
    parameters::{MotorPort, SensorPort},
    sensor_driver::SensorType,
};

#[derive(Debug)]
#[allow(private_interfaces)]
pub enum Ev3Error {
    SensorNotFound {
        port: SensorPort,
        expected_sensor_type: SensorType,
    },
    MotorNotFound {
        port: MotorPort,
    },
    FileNotFound {
        path: PathBuf,
    },
    InvalidPath,
    IncorrectSensorType {
        expected: SensorType,
        found: SensorType,
    },
    ParseStr {
        input: String,
        to: String,
    },
    PermissionDenied {
        required_permission: FileMode,
    },
    InvalidStringBytes,
    ReadAttributeFailure {
        filename: PathBuf,
        os_error: std::io::Error,
    },
    WriteAttributeFailure {
        filename: PathBuf,
        value: String,
        os_error: std::io::Error,
    },
    ParseInt {
        err: ParseIntError,
    },
    ParseFloat {
        err: ParseFloatError,
    },
    InvalidValue {
        func: String,
        value: String,
    },
    NoSensorProvided,
}

impl Display for Ev3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for Ev3Error {}

impl From<std::num::ParseIntError> for Ev3Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Ev3Error::ParseInt { err }
    }
}

impl From<std::num::ParseFloatError> for Ev3Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Ev3Error::ParseFloat { err }
    }
}

pub type Ev3Result<T> = Result<T, Ev3Error>;
