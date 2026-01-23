use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::error::{Ev3Error, Ev3Result};

#[derive(Debug)]
pub(crate) enum FileMode {
    Read,
    Write,
    ReadWrite,
}

pub(crate) struct Attribute {
    fd: Arc<Mutex<File>>,
    path: PathBuf,
    mode: FileMode,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[allow(dead_code)]
pub(crate) enum AttributeName {
    Address,
    DriverName,
    // motor attributes
    Command,
    Commands,
    CountPerRotation,
    CountPerMeter,
    FullTravelCount,
    DutyCycle,
    DutyCycleSetpoint,
    Polarity,
    Position,
    HoldPidKd,
    HoldPidKi,
    HoldPidKp,
    MaxSpeed,
    PositionSetpoint,
    Speed,
    SpeedSetpoint,
    RampUpSetpoint,
    RampDownSetpoint,
    SpeedPidKd,
    SpeedPidKi,
    SpeedPidKp,
    State,
    StopAction,
    StopActions,
    TimeSetpoint,

    // sensor attributes
    Mode,
    Modes,
    Value0,
    Value1,
    Value2,
    Value3,
    Value4,
    Value5,
    Value6,
    Value7,
    Value8,
}

impl Display for AttributeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeName::Address => write!(f, "address"),
            AttributeName::DriverName => write!(f, "driver_name"),

            AttributeName::Command => write!(f, "command"),
            AttributeName::Commands => write!(f, "commands"),
            AttributeName::CountPerRotation => write!(f, "count_per_rot"),
            AttributeName::CountPerMeter => write!(f, "count_per_m"),
            AttributeName::FullTravelCount => write!(f, "full_travel_count"),
            AttributeName::DutyCycle => write!(f, "duty_cycle"),
            AttributeName::DutyCycleSetpoint => write!(f, "duty_cycle_sp"),
            AttributeName::Polarity => write!(f, "polarity"),
            AttributeName::Position => write!(f, "position"),
            AttributeName::HoldPidKd => write!(f, "hold_pid/Kd"),
            AttributeName::HoldPidKi => write!(f, "hold_pid/Ki"),
            AttributeName::HoldPidKp => write!(f, "hold_pid/Kp"),
            AttributeName::MaxSpeed => write!(f, "max_speed"),
            AttributeName::PositionSetpoint => write!(f, "position_sp"),
            AttributeName::Speed => write!(f, "speed"),
            AttributeName::SpeedSetpoint => write!(f, "speed_sp"),
            AttributeName::RampUpSetpoint => write!(f, "ramp_up_sp"),
            AttributeName::RampDownSetpoint => write!(f, "ramp_down_sp"),
            AttributeName::SpeedPidKd => write!(f, "speed_pid/Kd"),
            AttributeName::SpeedPidKi => write!(f, "speed_pid/Ki"),
            AttributeName::SpeedPidKp => write!(f, "speed_pid/Kp"),
            AttributeName::State => write!(f, "state"),
            AttributeName::StopAction => write!(f, "stop_action"),
            AttributeName::StopActions => write!(f, "stop_actions"),
            AttributeName::TimeSetpoint => write!(f, "time_sp"),

            AttributeName::Mode => write!(f, "mode"),
            AttributeName::Modes => write!(f, "modes"),
            AttributeName::Value0 => write!(f, "value0"),
            AttributeName::Value1 => write!(f, "value1"),
            AttributeName::Value2 => write!(f, "value2"),
            AttributeName::Value3 => write!(f, "value3"),
            AttributeName::Value4 => write!(f, "value4"),
            AttributeName::Value5 => write!(f, "value5"),
            AttributeName::Value6 => write!(f, "value6"),
            AttributeName::Value7 => write!(f, "value7"),
            AttributeName::Value8 => write!(f, "value8"),
        }
    }
}

impl AttributeName {
    pub(crate) fn filemode(&self) -> FileMode {
        use AttributeName::*;
        use FileMode::*;
        match self {
            Address => Read,
            DriverName => Read,

            Command => Write,
            Commands => Read,
            CountPerRotation => Read,
            CountPerMeter => Read,
            FullTravelCount => Read,
            DutyCycle => Read,
            DutyCycleSetpoint => ReadWrite,
            Polarity => ReadWrite,
            Position => ReadWrite,
            HoldPidKd => ReadWrite,
            HoldPidKi => ReadWrite,
            HoldPidKp => ReadWrite,
            MaxSpeed => Read,
            PositionSetpoint => ReadWrite,
            Speed => Read,
            SpeedSetpoint => ReadWrite,
            RampUpSetpoint => ReadWrite,
            RampDownSetpoint => ReadWrite,
            SpeedPidKd => ReadWrite,
            SpeedPidKi => ReadWrite,
            SpeedPidKp => ReadWrite,
            State => Read,
            StopAction => ReadWrite,
            StopActions => Read,
            TimeSetpoint => ReadWrite,

            Mode => ReadWrite,
            Modes => Read,
            Value0 => Read,
            Value1 => Read,
            Value2 => Read,
            Value3 => Read,
            Value4 => Read,
            Value5 => Read,
            Value6 => Read,
            Value7 => Read,
            Value8 => Read,
        }
    }
}

impl Attribute {
    pub(crate) fn new(filename: PathBuf, mode: FileMode) -> Ev3Result<Self> {
        let path = filename.clone();

        let fd = Arc::new(Mutex::new(match mode {
            FileMode::Read => {
                File::open(&filename).or(Err(Ev3Error::FileNotFound { path: filename }))?
            }
            FileMode::Write => OpenOptions::new()
                .write(true)
                .open(&filename)
                .or(Err(Ev3Error::FileNotFound { path: filename }))?,
            FileMode::ReadWrite => OpenOptions::new()
                .read(true)
                .write(true)
                .open(&filename)
                .or(Err(Ev3Error::FileNotFound { path: filename }))?,
        }));
        Ok(Attribute { fd, mode, path })
    }
    pub(crate) fn get(&self) -> Ev3Result<String> {
        match self.mode {
            FileMode::Read | FileMode::ReadWrite => {
                let mut fd = self.fd.lock().unwrap();
                let mut buffer = String::new();
                fd.seek(SeekFrom::Start(0))
                    .map_err(|e| Ev3Error::ReadAttributeFailure {
                        filename: self.path.clone(),
                        os_error: e,
                    })?;

                // the file is occasionally in an invalid state
                // this usually clears up on a retry
                for _ in 0..5 {
                    if fd.read_to_string(&mut buffer).is_ok() {
                        return Ok(buffer.trim().into());
                    }
                }

                // if 5 tries fail in a row, return the error
                Err(Ev3Error::InvalidStringBytes)
            }
            _ => Err(Ev3Error::PermissionDenied {
                required_permission: FileMode::Read,
            }),
        }
    }
    pub(crate) fn set(&self, value: &str) -> Ev3Result<()> {
        match self.mode {
            FileMode::Write | FileMode::ReadWrite => {
                let mut fd = self.fd.lock().unwrap();
                fd.seek(SeekFrom::Start(0))
                    .map_err(|e| Ev3Error::ReadAttributeFailure {
                        filename: self.path.clone(),
                        os_error: e,
                    })?;
                fd.write_all(value.as_bytes())
                    .map_err(|e| Ev3Error::WriteAttributeFailure {
                        filename: self.path.clone(),
                        value: value.into(),
                        os_error: e,
                    })?;
                Ok(())
            }
            _ => Err(Ev3Error::PermissionDenied {
                required_permission: FileMode::Write,
            }),
        }
    }
}
