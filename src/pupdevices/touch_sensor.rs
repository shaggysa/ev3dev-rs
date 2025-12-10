use ev3dev_lang_rust::Ev3Result;

use crate::parameters::SensorPort;

pub struct TouchSensor {
    sensor: ev3dev_lang_rust::sensors::TouchSensor,
}

impl TouchSensor {
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        Ok(Self {
            sensor: ev3dev_lang_rust::sensors::TouchSensor::get(port)?,
        })
    }

    pub fn pressed(&self) -> Ev3Result<bool> {
        self.sensor.get_pressed_state()
    }
}
