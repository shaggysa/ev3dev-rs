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
/// ``` no_run
/// use ev3dev_rs::pupdevices::ColorSensor;
/// use ev3dev_rs::parameters::{SensorPort, Color};
///
/// let color_sensor = ColorSensor::new(SensorPort::In1)?;
///
/// println!("Reflected light intensity: {}", color_sensor.reflection().await?);
/// println!("Ambient light intensity: {}", color_sensor.ambient().await?);
/// println!("Current color: {}", color_sensor.color().await?);
///
/// let (r, g, b) = color_sensor.raw_rgb().await?;
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
    pub async fn reflection(&self) -> Ev3Result<u8> {
        self.driver.set_mode(ColorReflectedLight).await?;
        
        
        Ok(self.driver.read_attribute(Value0).await?.parse()?)
    }

    /// Get the ambient light intensity of the sensor as a percentage (0 to 100).
    pub async fn ambient(&self) -> Ev3Result<u8> {
        
        self.driver.set_mode(ColorAmbientLight).await?;
        
        Ok(self.driver.read_attribute(Value0).await?.parse()?)
    }

    /// Get the color detected by the sensor as a `Color`.
    pub async fn color(&self) -> Ev3Result<Color> {

            self.driver.set_mode(ColorColor).await?;
        
        Color::from_str(&self.driver.read_attribute(Value0).await?)
    }

    /// Get the raw RGB values of the sensor (0-1020).
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// let (r, g, b) = sensor.raw_rgb().await?;
    /// ```
    pub async fn raw_rgb(&self) -> Ev3Result<(u16, u16, u16)> {
        self.driver.set_mode(ColorRawRGB).await?;

        let r = self.driver.read_attribute(Value0).await?.parse()?;
        let g = self.driver.read_attribute(Value1).await?.parse()?;
        let b = self.driver.read_attribute(Value2).await?.parse()?;

        Ok((r, g, b))
    }
}
