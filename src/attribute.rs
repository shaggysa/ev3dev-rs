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
pub(crate) enum AttributeName {
    Address,
    DriverName,
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
        match self {
            AttributeName::Address => FileMode::Read,
            AttributeName::DriverName => FileMode::Read,
            AttributeName::Mode => FileMode::ReadWrite,
            AttributeName::Modes => FileMode::Read,
            AttributeName::Value0 => FileMode::Read,
            AttributeName::Value1 => FileMode::Read,
            AttributeName::Value2 => FileMode::Read,
            AttributeName::Value3 => FileMode::Read,
            AttributeName::Value4 => FileMode::Read,
            AttributeName::Value5 => FileMode::Read,
            AttributeName::Value6 => FileMode::Read,
            AttributeName::Value7 => FileMode::Read,
            AttributeName::Value8 => FileMode::Read,
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
                fd.read_to_string(&mut buffer)
                    .map_err(|_| Ev3Error::InvalidStringBytes)?;
                Ok(buffer.trim_end().into())
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
