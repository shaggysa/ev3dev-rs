use std::collections::HashSet;

use crate::{
    attribute::AttributeName,
    error::{Ev3Error, Ev3Result},
    parameters::{Button, SensorPort},
    sensor_driver::{SensorDriver, SensorMode, SensorType},
};

/// Stock EV3 Infrared Sensor
///
/// Note that this sensor does not support direct measurement of distance
/// and that the `proximity()` percentage does not scale linearly with distance.
///
/// If you want to get an accurate distance measurement, you should use an`UltrasonicSensor`.
pub struct InfraredSensor {
    driver: SensorDriver,
}

impl InfraredSensor {
    /// Find an `InfraredSensor` on the given port.
    ///
    /// Will return `SensorNotFound` if no sensor is found
    /// or `IncorrectSensorType` if the found sensor is not an `InfraredSensor`.
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        let driver = SensorDriver::new(SensorType::Infrared, port)?;
        Ok(Self { driver })
    }

    /// Get the proximity value of the sensor as a percentage (0 to 100).
    ///
    /// 100% is approximately 70cm/27in.
    ///
    /// Note that this sensor does not support direct measurement of distance
    /// and that the percentage does not scale linearly with distance.
    ///
    /// If you want to get an accurate distance measurement, you should use an `UltrasonicSensor`.
    pub fn proximity(&self) -> Ev3Result<u8> {
        if self.driver.mode.get() != SensorMode::InfraredProximity {
            self.driver.set_mode(SensorMode::InfraredProximity)?;
        }
        Ok(self.driver.read_attribute(AttributeName::Value0)?.parse()?)
    }

    #[inline]
    /// Get a `HashSet` of buttons currently pressed on the remote control channel 1.
    ///
    /// Note that the set will be empty if three or more buttons are pressed.
    pub fn get_remote_channel_1_buttons(&self) -> Ev3Result<HashSet<Button>> {
        self.get_remote_buttons(AttributeName::Value0)
    }

    /// Get a `HashSet` of buttons currently pressed on the remote control channel 2.
    ///
    /// Note that the set will be empty if three or more buttons are pressed.
    #[inline]
    pub fn get_remote_channel_2_buttons(&self) -> Ev3Result<HashSet<Button>> {
        self.get_remote_buttons(AttributeName::Value1)
    }

    /// Get a `HashSet` of buttons currently pressed on the remote control channel 3.
    ///
    /// Note that the set will be empty if three or more buttons are pressed.
    #[inline]
    pub fn get_remote_channel_3_buttons(&self) -> Ev3Result<HashSet<Button>> {
        self.get_remote_buttons(AttributeName::Value2)
    }

    /// Get a `HashSet` of buttons currently pressed on the remote control channel 4.
    ///
    /// Note that the set will be empty if three or more buttons are pressed.
    #[inline]
    pub fn get_remote_channel_4_buttons(&self) -> Ev3Result<HashSet<Button>> {
        self.get_remote_buttons(AttributeName::Value3)
    }

    /// Seeks a remote control in beacon mode on channel 1.
    ///
    /// The first value is heading (-25 to 25),
    /// and the second value is distance as a percentage (-128 and 0 to 100).
    ///
    /// The distance is -128 when the remote is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// let (heading, distance) = infrared_sensor.seek_channel_1()?;
    /// ```
    #[inline]
    pub fn seek_channel_1(&self) -> Ev3Result<(i8, i8)> {
        self.seek(AttributeName::Value0, AttributeName::Value1)
    }

    /// Seeks a remote control in beacon mode on channel 2.
    ///
    /// The first value is heading (-25 to 25),
    /// and the second value is distance as a percentage (-128 and 0 to 100).
    ///
    /// The distance is -128 when the remote is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// let (heading, distance) = infrared_sensor.seek_channel_2()?;
    /// ```
    #[inline]
    pub fn seek_channel_2(&self) -> Ev3Result<(i8, i8)> {
        self.seek(AttributeName::Value2, AttributeName::Value3)
    }

    /// Seeks a remote control in beacon mode on channel 3.
    ///
    /// The first value is heading (-25 to 25),
    /// and the second value is distance as a percentage (-128 and 0 to 100).
    ///
    /// The distance is -128 when the remote is out of range.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// let (heading, distance) = infrared_sensor.seek_channel_3()?;
    /// ```
    #[inline]
    pub fn seek_channel_3(&self) -> Ev3Result<(i8, i8)> {
        self.seek(AttributeName::Value4, AttributeName::Value5)
    }

    /// Seeks a remote control in beacon mode on channel 4.
    ///
    /// The first value is heading (-25 to 25),
    /// and the second value is distance as a percentage (-128 and 0 to 100).
    ///
    /// The distance is -128 when the remote is out of range.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// let (heading, distance) = infrared_sensor.seek_channel_4()?;
    /// ```
    #[inline]
    pub fn seek_channel_4(&self) -> Ev3Result<(i8, i8)> {
        self.seek(AttributeName::Value6, AttributeName::Value7)
    }

    fn get_remote_buttons(&self, attr: AttributeName) -> Ev3Result<HashSet<Button>> {
        if self.driver.mode.get() != SensorMode::InfraredRemote {
            self.driver.set_mode(SensorMode::InfraredRemote)?;
        }

        let val = self.driver.read_attribute(attr)?;
        let mut set = HashSet::new();

        match val.parse::<u8>()? {
            0 => (),
            1 => _ = set.insert(Button::RedUp),
            2 => _ = set.insert(Button::RedDown),
            3 => _ = set.insert(Button::BlueUp),
            4 => _ = set.insert(Button::BlueDown),
            5 => {
                _ = set.insert(Button::RedUp);
                _ = set.insert(Button::BlueUp);
            }
            6 => {
                _ = set.insert(Button::RedUp);
                _ = set.insert(Button::BlueDown);
            }
            7 => {
                _ = set.insert(Button::RedDown);
                _ = set.insert(Button::BlueUp);
            }
            8 => {
                _ = set.insert(Button::RedDown);
                _ = set.insert(Button::BlueDown);
            }
            9 => _ = set.insert(Button::BeaconOn),
            10 => {
                _ = set.insert(Button::RedUp);
                _ = set.insert(Button::RedDown);
            }
            11 => {
                _ = set.insert(Button::BlueUp);
                _ = set.insert(Button::BlueDown);
            }
            _ => {
                return Err(Ev3Error::InvalidValue {
                    func: "InfraredSensor::get_remote_buttons".into(),
                    value: val,
                });
            }
        };

        Ok(set)
    }

    fn seek(&self, attr1: AttributeName, attr2: AttributeName) -> Ev3Result<(i8, i8)> {
        if self.driver.mode.get() != SensorMode::InfraredSeek {
            self.driver.set_mode(SensorMode::InfraredSeek)?;
        }
        Ok((
            self.driver.read_attribute(attr1)?.parse()?,
            self.driver.read_attribute(attr2)?.parse()?,
        ))
    }
}
