use crate::{
    attribute::AttributeName,
    error::Ev3Result,
    parameters::SensorPort,
    sensor_driver::{
        SensorDriver,
        SensorMode::{self, *},
        SensorType,
    },
};

/// Stock EV3 Gyro Sensor
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

    /// Get the current heading of the sensor in degrees (-32768 to 32767).
    pub fn heading(&self) -> Ev3Result<i16> {
        match self.driver.mode.get() {
            GyroAngle | GyroAngleAndRate => {
                Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
            }
            _ => {
                self.driver.set_mode(GyroAngleAndRate)?;
                Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
            }
        }
    }

    /// Get the current angular velocity of the sensor in degrees per second (-440 to 440).
    pub fn angular_velocity(&self) -> Ev3Result<i16> {
        match self.driver.mode.get() {
            GyroRate => Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?),
            GyroAngleAndRate => Ok(self.driver.read_attribute(AttributeName::Value1)?.parse()?),
            _ => {
                self.driver.set_mode(GyroAngleAndRate)?;
                Ok(self.driver.read_attribute(AttributeName::Value1)?.parse()?)
            }
        }
    }

    /// Get the current heading and angular velocity of the sensor.
    ///
    /// This does the same job as calling both `heading()` and `angular_velocity()`,
    /// but it is more efficient because it reads them simultaneously.
    ///
    /// # Examples
    ///
    /// ```
    /// let (heading, velocity) = sensor.heading_and_velocity()?;
    /// assert_eq!(heading, sensor.heading()?);
    /// assert_eq!(velocity, sensor.angular_velocity()?);
    /// ```
    pub fn heading_and_velocity(&self) -> Ev3Result<(i16, i16)> {
        if self.driver.mode.get() != GyroAngleAndRate {
            self.driver.set_mode(GyroAngleAndRate)?;
        }

        let heading = self.driver.read_attribute(AttributeName::Value0)?.parse()?;
        let velocity = self.driver.read_attribute(AttributeName::Value1)?.parse()?;

        Ok((heading, velocity))
    }

    /// Get the current tilt angle of the sensor in degrees (-32768 to 32767).
    pub fn tilt(&self) -> Ev3Result<i16> {
        if self.driver.mode.get() != GyroTiltAngle {
            self.driver.set_mode(GyroTiltAngle)?;
        }

        Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
    }

    /// Get the current tilt velocity of the sensor in degrees per second (-440 to 440).
    pub fn tilt_velocity(&self) -> Ev3Result<i16> {
        if self.driver.mode.get() != GyroTiltRate {
            self.driver.set_mode(GyroTiltRate)?;
        }

        Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
    }
}
