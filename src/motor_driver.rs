use crate::attribute::{Attribute, AttributeName, FileMode};
use crate::enum_string::AsStr;
use crate::error::{Ev3Error, Ev3Result};
use crate::parameters::MotorPort;
use std::cell::RefCell;
use std::path::PathBuf;
use std::str::FromStr;

use std::{collections::HashMap, fs};

static MOTOR_DIR: &str = "/sys/class/tacho-motor";

pub(crate) struct MotorDriver {
    base_path: PathBuf,
    attributes: RefCell<HashMap<AttributeName, Attribute>>,
}

impl MotorDriver {
    pub(crate) fn new(port: MotorPort) -> Ev3Result<Self> {
        if let Ok(entries) = fs::read_dir(MOTOR_DIR) {
            for entry in entries {
                if let Ok(direntry) = entry.map(|e| e.path().to_path_buf())
                    && let Ok(address) = MotorPort::from_str(
                        fs::read_to_string(direntry.join("address"))
                            .or(Err(Ev3Error::InvalidPath))?
                            .trim(),
                    )
                    && address == port
                {
                    let mut attributes = HashMap::new();
                    let rot_attr = Attribute::new(
                        direntry.join(AttributeName::CountPerRotation.to_string()),
                        FileMode::Read,
                    )?;
                    attributes.insert(AttributeName::Mode, rot_attr);

                    return Ok(Self {
                        base_path: direntry,
                        attributes: RefCell::new(attributes),
                    });
                }
            }
        }

        Err(Ev3Error::MotorNotFound { port })
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

    pub(crate) fn set_attribute_enum<T>(&self, name: AttributeName, value: T) -> Ev3Result<()>
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

    pub(crate) fn set_attribute<T>(&self, name: AttributeName, value: T) -> Ev3Result<()>
    where
        T: ToString,
    {
        if let Some(attr) = self.attributes.borrow().get(&name) {
            attr.set(&value.to_string())
        } else {
            // if the value if not in the hashmap, create a new attribue,
            // set its value, and insert it into the hashmap
            let attr = Attribute::new(self.base_path.join(name.to_string()), name.filemode())?;
            attr.set(&value.to_string())?;
            _ = self.attributes.borrow_mut().insert(name, attr);
            Ok(())
        }
    }
}
