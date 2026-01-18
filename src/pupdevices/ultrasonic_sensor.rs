use crate::{
    attribute::AttributeName,
    error::Ev3Result,
    parameters::SensorPort,
    sensor_driver::{SensorDriver, SensorMode, SensorType},
};

/// A stock EV3 ultrasonic sensor.
///
/// # Examples
/// ```
/// use ev3dev_rs::pupdevices::UltrasonicSensor;
/// use ev3dev_rs::parameters::SensorPort;
///
/// let ultrasonic_sensor = UltrasonicSensor::new(SensorPort::In1)?;
///
/// println!("Distance (in): {}", ultrasonic_sensor.distance_in()?);
/// println!("Distance (cm): {}", ultrasonic_sensor.distance_cm()?);
///
/// ```
pub struct UltrasonicSensor {
    driver: SensorDriver,
}

impl UltrasonicSensor {
    /// Find an `UltrasonicSensor` on the given port.
    ///
    /// Will return `SensorNotFound` if no sensor is found
    /// or `IncorrectSensorType` if the found sensor is not an `UltrasonicSensor`.
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        let driver = SensorDriver::new(SensorType::Ultrasonic, port)?;
        Ok(Self { driver })
    }

    /// Get the distance value of the sensor in inches to one decimal place (0-2550).
    pub fn distance_in(&self) -> Ev3Result<f32> {
        if self.driver.mode.get() != SensorMode::UltrasonicDistanceIn {
            self.driver.set_mode(SensorMode::UltrasonicDistanceIn)?;
        }
        Ok(self
            .driver
            .read_attribute(AttributeName::Value0)?
            .parse::<f32>()?
            / 10.0)
    }

    /// Get the distance value of the sensor in centimeters to one decimal place (0-1003).
    pub fn distance_cm(&self) -> Ev3Result<f32> {
        if self.driver.mode.get() != SensorMode::UltrasonicDistanceCm {
            self.driver.set_mode(SensorMode::UltrasonicDistanceCm)?;
        }
        Ok(self
            .driver
            .read_attribute(AttributeName::Value0)?
            .parse::<f32>()?
            / 10.0)
    }
}
