use crate::attribute::{Attribute, AttributeName, FileMode};
use crate::crate_enum_str;
use crate::enum_string::AsStr;
use crate::error::{Ev3Error, Ev3Result};
use crate::parameters::SensorPort;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::str::FromStr;
use std::{collections::HashMap, fs};

const SENSOR_DIR: &str = "/sys/class/lego-sensor";

crate_enum_str! {
    SensorType,

    (Gyro, "lego-ev3-gyro"),
    (Color, "lego-ev3-color"),
    (Ultrasonic, "lego-ev3-us"),
    (Touch, "lego-ev3-touch"),
    (Infrared, "lego-ev3-ir"),
}

crate_enum_str! {
    SensorMode,

    (GyroAngle, "GYRO-ANG"),
    (GyroRate, "GYRO-RATE"),
    (GyroRateUnscaled, "GYRO-FAS"),
    (GyroAngleAndRate, "GYRO-G&A"),
    (GyroTiltAngle, "TILT-ANG"),
    (GyroTiltRate, "TILT-RATE"),
    (GyroCalibration, "GYRO-CAL"),

    (ColorReflectedLight, "COL-REFLECT"),
    (ColorAmbientLight, "COL-AMBIENT"),
    (ColorColor, "COL-COLOR"),
    (ColorRawReflected, "REF-RAW"),
    (ColorRawRGB, "RGB-RAW"),
    (ColorCalibration, "COL-CAL"),

    (TouchState, "TOUCH"),

    (UltrasonicDistanceCm, "US-DIST-CM"),
    (UltrasonicDistanceIn, "US-DIST-IN"),
    (UltrasonicListen, "US-LISTEN"),
    (UltrasonicSiCm, "US-SI-CM"),
    (UltrasonicSiIn, "US-SI-IN"),
    (UltrasonicDcCm, "US-DC-CM"),
    (UltrasonicDcIn, "US-DC-IN"),

    (InfraredProximity, "IR-PROX"),
    (InfraredSeek, "IR-SEEK"),
    (InfraredRemote, "IR-REMOTE"),
    (InfraredRemA, "IR-REM-A"),
    (InfraredAltSeeker, "IR-S-ALT"),
    (InfraredCalibration, "IR-CAL"),

}

pub(crate) struct SensorDriver {
    base_path: PathBuf,
    attributes: RefCell<HashMap<AttributeName, Attribute>>,
    pub(crate) mode: Cell<SensorMode>,
}

impl SensorDriver {
    pub(crate) fn new(sensor_type: SensorType, port: SensorPort) -> Ev3Result<Self> {
        if let Ok(entries) = fs::read_dir(SENSOR_DIR) {
            for entry in entries {
                if let Ok(direntry) = entry.map(|e| e.path().to_path_buf())
                    && let Ok(address) = SensorPort::from_str(
                        fs::read_to_string(direntry.join("address"))
                            .or(Err(Ev3Error::InvalidPath))?
                            .trim(),
                    )
                    && let Ok(driver) = SensorType::from_str(
                        fs::read_to_string(direntry.join("driver_name"))
                            .or(Err(Ev3Error::InvalidPath))?
                            .trim(),
                    )
                    && address == port
                {
                    if driver == sensor_type {
                        let mut attributes = HashMap::new();
                        let mode_attr = Attribute::new(
                            direntry.join(AttributeName::Mode.to_string()),
                            FileMode::ReadWrite,
                        )?;

                        let mode = SensorMode::from_str(&mode_attr.get()?)?;

                        attributes.insert(AttributeName::Mode, mode_attr);

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

    pub(crate) fn read_attribute(&self, name: AttributeName) -> Ev3Result<String> {
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

    pub(crate) fn set_attribute<T>(&self, name: AttributeName, value: T) -> Ev3Result<()>
    where
        T: AsStr,
    {
        if let Some(attr) = self.attributes.borrow().get(&name) {
            attr.set(value.as_str())
        } else {
            // if the value if not in the hashmap, create a new attribue,
            // set its value, and insert it into the hashmap
            let attr = Attribute::new(self.base_path.join(name.to_string()), name.filemode())?;
            attr.set(value.as_str())?;
            _ = self.attributes.borrow_mut().insert(name, attr);
            Ok(())
        }
    }

    pub(crate) fn set_mode(&self, mode: SensorMode) -> Ev3Result<()> {
        self.set_attribute(AttributeName::Mode, mode)?;
        self.mode.set(mode);
        Ok(())
    }
}
