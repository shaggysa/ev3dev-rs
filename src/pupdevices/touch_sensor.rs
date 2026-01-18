use crate::{
    attribute::AttributeName::*,
    error::{Ev3Error, Ev3Result},
    parameters::SensorPort,
    sensor_driver::{SensorDriver, SensorType},
};

/// A stock EV3 touch sensor.
///
/// # Examples
/// ```
/// use ev3dev_rs::pupdevices::TouchSensor;
/// use ev3dev_rs::parameters::SensorPort;
///
/// let touch_sensor = TouchSensor::new(SensorPort::In1)?;
///
/// println!("Is the sensor pressed? : {}", if touch_sensor.pressed()? {"Yes"} else {"No"});
///
/// ```
pub struct TouchSensor {
    driver: SensorDriver,
}

impl TouchSensor {
    /// Find a `TouchSensor` on the given port.
    ///
    /// Will return `SensorNotFound` if no sensor is found
    /// or `IncorrectSensorType` if the found sensor is not a `TouchSensor`.
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        let driver = SensorDriver::new(SensorType::Touch, port)?;
        Ok(Self { driver })
    }

    /// Returns `true` if the sensor is currently pressed and `false` otherwise.
    pub fn pressed(&self) -> Ev3Result<bool> {
        // only one possible mode, no need to check
        let raw_value = self.driver.read_attribute(Value0)?;

        let int_value: u8 = raw_value.parse()?;

        match int_value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Ev3Error::InvalidValue {
                func: "TouchSensor::pressed".into(),
                value: raw_value,
            }),
        }
    }
}
