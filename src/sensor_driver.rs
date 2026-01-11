use crate::attribute::{Attribute, AttributeName, FileMode};
use crate::error::{Ev3Error, Ev3Result};
use crate::parameters::SensorPort;
use crate::sensors::gyro_sensor::GyroMode;
use std::cell::{Cell, RefCell};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use std::{collections::HashMap, fs};

const SENSOR_DIR: &str = "/sys/class/lego-sensor";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum SensorType {
    Ultrasonic,
    Gyro,
    Color,
    Touch,
    Infrared,
}

impl FromStr for SensorType {
    type Err = Ev3Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_suffix("\n").unwrap() {
            "lego-ev3-us" => Ok(SensorType::Ultrasonic),
            "lego-ev3-gyro" => Ok(SensorType::Gyro),
            "lego-ev3-color" => Ok(SensorType::Color),
            "lego-ev3-touch" => Ok(SensorType::Touch),
            "lego-ev3-ir" => Ok(SensorType::Infrared),
            _ => Err(Ev3Error::ParseStr {
                str: s.into(),
                enum_type: "SensorType".to_string(),
            }),
        }
    }
}

impl FromStr for SensorPort {
    type Err = Ev3Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_suffix("\n").unwrap() {
            "ev3-ports:in1" => Ok(SensorPort::In1),
            "ev3-ports:in2" => Ok(SensorPort::In2),
            "ev3-ports:in3" => Ok(SensorPort::In3),
            "ev3-ports:in4" => Ok(SensorPort::In4),
            _ => Err(Ev3Error::ParseStr {
                str: s.into(),
                enum_type: "SensorPort".to_string(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SensorMode {
    Gyro(GyroMode),
    None,
}

impl Display for SensorMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorMode::Gyro(mode) => mode.fmt(f),
            SensorMode::None => write!(f, "NONE"),
        }
    }
}

impl FromStr for SensorMode {
    type Err = Ev3Error;

    fn from_str(s: &str) -> Result<Self, Ev3Error> {
        match s {
            "GYRO-ANG" => Ok(SensorMode::Gyro(GyroMode::Angle)),
            "GYRO-RATE" => Ok(SensorMode::Gyro(GyroMode::Rate)),
            "GYRO-FAS" => Ok(SensorMode::Gyro(GyroMode::RateUnscaled)),
            "GYRO-G&A" => Ok(SensorMode::Gyro(GyroMode::AngleAndRate)),
            "TILT-ANG" => Ok(SensorMode::Gyro(GyroMode::TiltAngle)),
            "TILT-RATE" => Ok(SensorMode::Gyro(GyroMode::TiltRate)),
            "GYRO-CAL" => Ok(SensorMode::Gyro(GyroMode::Calibration)),
            "" => Ok(SensorMode::None),
            _ => Err(Ev3Error::ParseStr {
                str: s.into(),
                enum_type: "SensorMode".to_string(),
            }),
        }
    }
}

pub struct SensorDriver {
    base_path: PathBuf,
    attributes: RefCell<HashMap<AttributeName, Attribute>>,
    pub(crate) mode: Cell<SensorMode>,
}

impl SensorDriver {
    pub fn new(sensor_type: SensorType, port: SensorPort) -> Ev3Result<Self> {
        if let Ok(entries) = fs::read_dir(SENSOR_DIR) {
            for entry in entries {
                if let Ok(direntry) = entry.map(|e| e.path().to_path_buf())
                    && let Ok(address) = SensorPort::from_str(
                        &fs::read_to_string(format!(
                            "{}/address",
                            direntry.to_str().ok_or(Ev3Error::InvalidPath)?
                        ))
                        .or(Err(Ev3Error::InvalidPath))?,
                    )
                    && let Ok(driver) = SensorType::from_str(
                        &fs::read_to_string(format!(
                            "{}/driver_name",
                            direntry.to_str().ok_or(Ev3Error::InvalidPath)?
                        ))
                        .or(Err(Ev3Error::InvalidPath))?,
                    )
                    && address == port
                {
                    let mut attributes = HashMap::new();
                    let mode_attr = Attribute::new(
                        direntry.join(AttributeName::Mode.to_string()),
                        FileMode::ReadWrite,
                    )?;

                    let mode = SensorMode::from_str(&mode_attr.get()?)?;

                    attributes.insert(AttributeName::Mode, mode_attr);

                    if driver == sensor_type {
                        return Ok(Self {
                            base_path: direntry,
                            attributes: RefCell::new(attributes),
                            mode: Cell::new(mode),
                        });
                    } else {
                        return Err(Ev3Error::IncorrectSensorType {
                            expected: sensor_type,
                            found: driver,
                        });
                    }
                }
            }
        }
        Err(Ev3Error::SensorNotFound {
            port,
            expected_sensor_type: sensor_type,
        })
    }

    pub fn read_attribute(&self, name: AttributeName) -> Ev3Result<String> {
        if let Some(attr) = self.attributes.borrow().get(&name) {
            attr.get()
        } else {
            // if the value if not in the hashmap, create a new attribue,
            // get its current value, and insert it into the hashmap
            let attr = Attribute::new(self.base_path.join(name.to_string()), name.filemode())?;
            let val = attr.get()?;
            _ = self.attributes.borrow_mut().insert(name, attr);
            Ok(val)
        }
    }

    pub fn set_attribute(&self, name: AttributeName, value: &str) -> Ev3Result<()> {
        if let Some(attr) = self.attributes.borrow().get(&name) {
            attr.set(value)
        } else {
            // if the value if not in the hashmap, create a new attribue,
            // set its value, and insert it into the hashmap
            let attr = Attribute::new(self.base_path.join(name.to_string()), name.filemode())?;
            attr.set(value)?;
            _ = self.attributes.borrow_mut().insert(name, attr);
            Ok(())
        }
    }

    pub fn set_mode(&self, mode: SensorMode) -> Ev3Result<()> {
        self.set_attribute(AttributeName::Mode, mode.to_string().as_str())?;
        self.mode.set(mode);
        Ok(())
    }
}
