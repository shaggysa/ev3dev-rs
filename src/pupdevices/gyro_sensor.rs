use ev3dev_lang_rust::Ev3Result;

use crate::parameters::SensorPort;

pub struct GyroSensor {
    sensor: ev3dev_lang_rust::sensors::GyroSensor,
}

impl GyroSensor {
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        let sensor = ev3dev_lang_rust::sensors::GyroSensor::get(port)?;
        sensor.set_mode_gyro_g_and_a()?;
        Ok(Self { sensor })
    }
    pub fn heading(&self) -> Ev3Result<i32> {
        self.sensor.get_angle()
    }
    pub fn angular_velocity(&self) -> Ev3Result<i32> {
        self.sensor.get_rotational_speed()
    }
}
