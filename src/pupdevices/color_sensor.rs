use crate::parameters::{Color, Ev3Result, SensorPort};

pub struct ColorSensor {
    sensor: ev3dev_lang_rust::sensors::ColorSensor,
}

impl ColorSensor {
    pub fn new(port: SensorPort) -> Ev3Result<Self> {
        Ok(Self {
            sensor: ev3dev_lang_rust::sensors::ColorSensor::get(port)?,
        })
    }

    pub fn color(&self) -> Ev3Result<Color> {
        let (r, g, b) = self.sensor.get_rgb()?;

        Ok(Color::from_rgb(r, g, b))
    }

    pub fn reflection(&self) -> Ev3Result<u8> {
        let (r, g, b) = self.sensor.get_rgb()?;

        let flr = r as f32 / 1020.0;
        let flg = g as f32 / 1020.0;
        let flb = b as f32 / 1020.0;

        // Perceived luminance formula
        let intensity = 0.2126 * flr + 0.7152 * flg + 0.0722 * flb;

        Ok((intensity * 100.0) as u8)
    }

    pub fn ambient(&self) -> Ev3Result<u8> {
        let (r, g, b) = self.sensor.get_rgb()?;
        let avg = (r + g + b) as f32 / 3.0;

        Ok(((avg / 1020.0) * 100.0) as u8)
    }
}
