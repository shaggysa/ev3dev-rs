use crate::{
    attribute::AttributeName::*,
    error::Ev3Result,
    parameters::{Color, SensorPort},
    sensor_driver::{SensorDriver, SensorMode::*, SensorType},
};
use std::str::FromStr;

/// Stock EV3 Color Sensor
///
/// # Examples
/// ```
/// use ev3dev_rs::pupdevices::ColorSensor;
/// use ev3dev_rs::parameters::{SensorPort, Color};
///
/// let color_sensor = ColorSensor::new(SensorPort::In1)?;
///
/// println!("Reflected light intensity: {}", color_sensor.reflection()?);
/// println!("Ambient light intensity: {}", color_sensor.ambient()?);
/// println!("Current color: {}", color_sensor.color()?);
///
/// let (r, g, b) = color_sensor.rgb()?;
/// println!("red: {}", r);
/// println!("green: {}", g);
/// println!("blue: {}", b);
/// ```
pub struct ColorSensor {
    driver: SensorDriver,
}

impl ColorSensor {
    /// Find a `ColorSensor` on the given port.
    ///
    /// Will return `SensorNotFound` if no sensor is found
    /// or `IncorrectSensorType` if the found sensor is not a `ColorSensor`.
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        let driver = SensorDriver::new(SensorType::Color, port)?;
        Ok(Self { driver })
    }

    /// Get the reflected light intensity of the sensor as a percentage (0 to 100).
    pub fn reflection(&self) -> Ev3Result<u8> {
        if self.driver.mode.get() != ColorReflectedLight {
            self.driver.set_mode(ColorReflectedLight)?;
        }
        Ok(self.driver.read_attribute(Value0)?.parse()?)
    }

    /// Get the ambient light intensity of the sensor as a percentage (0 to 100).
    pub fn ambient(&self) -> Ev3Result<u8> {
        if self.driver.mode.get() != ColorAmbientLight {
            self.driver.set_mode(ColorAmbientLight)?;
        }
        Ok(self.driver.read_attribute(Value0)?.parse()?)
    }

    /// Get the color detected by the sensor as a `Color`.
    pub fn color(&self) -> Ev3Result<Color> {
        if self.driver.mode.get() != ColorColor {
            self.driver.set_mode(ColorColor)?;
        }
        Color::from_str(&self.driver.read_attribute(Value0)?)
    }

    /// Get the raw RGB values of the sensor (0-1020).
    ///
    /// # Examples
    ///
    /// ```
    /// let (r, g, b) = sensor.raw_rgb()?;
    /// ```
    pub fn raw_rgb(&self) -> Ev3Result<(u16, u16, u16)> {
        if self.driver.mode.get() != ColorRawRGB {
            self.driver.set_mode(ColorRawRGB)?;
        }

        let r = self.driver.read_attribute(Value0)?.parse()?;
        let g = self.driver.read_attribute(Value1)?.parse()?;
        let b = self.driver.read_attribute(Value2)?.parse()?;

        Ok((r, g, b))
    }
}
