use std::vec;

use crate::error::Ev3Result;
use crate::pupdevices::GyroSensor;

/// A structure that can get the average reading from multiple gyro sensors in a single command.
///
/// This is also how you allow a `DriveBase` to use your gyro sensor(s)
///
/// # Examples
///
/// ```
/// use ev3dev_rs::robotics::GyroController;
/// use ev3dev_rs::pupdevices::GyroSensor;
/// use ev3dev_rs::parameters::SensorPort;
///
/// let gyro1 = GyroSensor::new(SensorPort::In1)?;
/// let gyro2 = GyroSensor::new(SensorPort::In2)?;
///
/// let controller = GyroController::new(vec![gyro1, gyro2])?;
///
/// let heading = controller.heading()?;
/// let angular_velocity = controller.angular_velocity()?;
///
/// println!("Heading: {}", heading);
/// println!("Angular Velocity: {}", angular_velocity);
/// ```
pub struct GyroController {
    pub(crate) gyros: Vec<(GyroSensor, i16)>,
}

impl GyroController {
    pub fn new(gyros: Vec<GyroSensor>) -> Ev3Result<Self> {
        let mut gyros_with_offsets = Vec::new();
        for gyro in gyros {
            let heading = gyro.heading()?;
            gyros_with_offsets.push((gyro, heading));
        }
        Ok(Self {
            gyros: gyros_with_offsets,
        })
    }

    pub fn heading(&self) -> Ev3Result<f32> {
        let mut sum = 0.0;
        for (gyro, offset) in &self.gyros {
            sum += (gyro.heading()? - *offset) as f32;
        }

        Ok(sum / self.gyros.len() as f32)
    }

    pub fn angular_velocity(&self) -> Ev3Result<f32> {
        let mut sum = 0.0;
        for (gyro, _) in &self.gyros {
            sum += gyro.angular_velocity()? as f32;
        }

        Ok(sum / self.gyros.len() as f32)
    }
}
