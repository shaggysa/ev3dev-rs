use crate::{
    attribute::AttributeName,
    error::Ev3Result,
    parameters::SensorPort,
    sensor_driver::{SensorDriver, SensorMode::*, SensorType},
};

/// Stock EV3 Gyro Sensor
///
/// # Examples
/// ``` no_run
/// use ev3dev_rs::pupdevices::GyroSensor;
/// use ev3dev_rs::parameters::SensorPort;
///
/// let gyro_sensor = GyroSensor::new(SensorPort::In1)?;
///
/// println!("Heading: {}", gyro_sensor.heading().await?);
/// println!("Velocity: {}", gyro_sensor.rate().await?);
/// println!("Tilt: {}", gyro_sensor.tilt().await?);
/// println!("Tilt Velocity: {}", gyro_sensor.tilt_velocity().await?);
///
/// ```
pub struct GyroSensor {
    driver: SensorDriver,
}

impl GyroSensor {
    /// Find a `GyroSensor` on the given port.
    ///
    /// Will return `SensorNotFound` if no sensor is found
    /// or `IncorrectSensorType` if the found sensor is not a `GyroSensor`.
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        let driver = SensorDriver::new(SensorType::Gyro, port)?;
        Ok(Self { driver })
    }

    /// Get the current heading of the sensor in degrees (-32768 to 32,767).
    pub async fn heading(&self) -> Ev3Result<i16> {
        self.driver.set_mode(GyroAngleAndRate).await?;
        Ok(self.driver.read_attribute(AttributeName::Value0).await?.parse()?)
    }

    /// Get the current angular velocity of the sensor in degrees per second (-440 to 440).
    pub async fn angular_velocity(&self) -> Ev3Result<i16> {
        self.driver.set_mode(GyroAngleAndRate).await?;

        Ok(self.driver.read_attribute(AttributeName::Value1).await?.parse()?)

    }

    /// Get the current heading and angular velocity of the sensor.
    ///
    /// This does the same job as calling both `heading()` and `angular_velocity()`,
    /// but it is more efficient because it reads them simultaneously.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// let (heading, velocity) = sensor.heading_and_velocity()?;
    /// assert_eq!(heading, sensor.heading().await?);
    /// assert_eq!(velocity, sensor.angular_velocity().await?);
    /// ```
    pub async fn heading_and_velocity(&self) -> Ev3Result<(i16, i16)> {
        self.driver.set_mode(GyroAngleAndRate).await?;

        let heading = self.driver.read_attribute(AttributeName::Value0).await?.parse()?;
        let velocity = self.driver.read_attribute(AttributeName::Value1).await?.parse()?;

        Ok((heading, velocity))
    }

    /// Get the current tilt angle of the sensor in degrees (-32768 to, 32,767).
    pub async fn tilt(&self) -> Ev3Result<i16> {
        self.driver.set_mode(GyroTiltAngle).await?;

        Ok(self.driver.read_attribute(AttributeName::Value0).await?.parse()?)
    }

    /// Get the current tilt velocity of the sensor in degrees per second (-440 to 440).
    pub async fn tilt_velocity(&self) -> Ev3Result<i16> {
        self.driver.set_mode(GyroTiltRate).await?;

        Ok(self.driver.read_attribute(AttributeName::Value0).await?.parse()?)
    }
}
