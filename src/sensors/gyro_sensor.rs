use crate::{
    attribute::AttributeName,
    error::Ev3Result,
    parameters::SensorPort,
    sensor_driver::{SensorDriver, SensorMode::*, SensorType},
};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GyroMode {
    Angle,
    Rate,
    RateUnscaled,
    AngleAndRate,
    TiltAngle,
    TiltRate,
    Calibration,
}

impl Display for GyroMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GyroMode::Angle => write!(f, "GYRO-ANG"),
            GyroMode::Rate => write!(f, "GYRO-RATE"),
            GyroMode::RateUnscaled => write!(f, "GYRO-FAS"),
            GyroMode::AngleAndRate => write!(f, "GYRO-G&A"),
            GyroMode::TiltAngle => write!(f, "TILT-ANG"),
            GyroMode::TiltRate => write!(f, "TILT-RATE"),
            GyroMode::Calibration => write!(f, "GYRO-CAL"),
        }
    }
}

pub struct GyroSensor {
    driver: SensorDriver,
}

impl GyroSensor {
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        let driver = SensorDriver::new(SensorType::Gyro, port)?;
        Ok(Self { driver })
    }

    fn set_mode(&self, mode: GyroMode) -> Ev3Result<()> {
        self.driver.set_mode(Gyro(mode))
    }

    pub fn heading(&self) -> Ev3Result<i16> {
        match self.driver.mode.get() {
            Gyro(GyroMode::Angle) | Gyro(GyroMode::AngleAndRate) => {
                Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
            }
            _ => {
                self.driver.set_mode(Gyro(GyroMode::AngleAndRate))?;
                Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
            }
        }
    }

    pub fn angular_velocity(&self) -> Ev3Result<i16> {
        match self.driver.mode.get() {
            Gyro(GyroMode::Rate) => {
                Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
            }
            Gyro(GyroMode::AngleAndRate) => {
                Ok(self.driver.read_attribute(AttributeName::Value1)?.parse()?)
            }
            _ => {
                self.driver.set_mode(Gyro(GyroMode::AngleAndRate))?;
                Ok(self.driver.read_attribute(AttributeName::Value1)?.parse()?)
            }
        }
    }

    pub fn heading_and_velocity(&self) -> Ev3Result<(i16, i16)> {
        if self.driver.mode.get() != Gyro(GyroMode::AngleAndRate) {
            self.driver.set_mode(Gyro(GyroMode::AngleAndRate))?;
        }

        let heading = self.driver.read_attribute(AttributeName::Value0)?.parse()?;
        let velocity = self.driver.read_attribute(AttributeName::Value1)?.parse()?;

        Ok((heading, velocity))
    }

    pub fn tilt(&self) -> Ev3Result<i16> {
        if self.driver.mode.get() != Gyro(GyroMode::TiltAngle) {
            self.driver.set_mode(Gyro(GyroMode::TiltAngle))?;
        }

        Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
    }

    pub fn tilt_velocity(&self) -> Ev3Result<i16> {
        if self.driver.mode.get() != Gyro(GyroMode::TiltRate) {
            self.driver.set_mode(Gyro(GyroMode::TiltRate))?;
        }

        Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
    }
}
