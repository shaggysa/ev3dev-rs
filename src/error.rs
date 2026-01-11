use std::{num::ParseIntError, path::PathBuf};

use crate::{attribute::FileMode, parameters::SensorPort, sensor_driver::SensorType};

#[derive(Debug)]
pub enum Ev3Error {
    SensorNotFound {
        port: SensorPort,
        expected_sensor_type: SensorType,
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
        str: String,
        enum_type: String,
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
}

impl From<std::num::ParseIntError> for Ev3Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Ev3Error::ParseInt { err }
    }
}

pub type Ev3Result<T> = Result<T, Ev3Error>;
