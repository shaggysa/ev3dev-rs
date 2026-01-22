use std::cell::RefCell;

use fixed::types::I32F32;

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
pub struct GyroController<'a> {
    pub(crate) gyros: RefCell<Vec<(&'a GyroSensor, i16)>>,
}

impl<'a> GyroController<'a> {
    pub fn new(gyros: Vec<&'a GyroSensor>) -> Ev3Result<Self> {
        let mut gyros_with_offsets = Vec::new();
        for gyro in gyros {
            let heading = gyro.heading()?;
            gyros_with_offsets.push((gyro, heading));
        }
        Ok(Self {
            gyros: RefCell::new(gyros_with_offsets),
        })
    }

    pub fn heading(&self) -> Ev3Result<I32F32> {
        let mut sum = I32F32::from_num(0.0);
        for (gyro, offset) in self.gyros.borrow().iter() {
            sum += I32F32::from_num(gyro.heading()? - offset);
        }

        Ok(sum / I32F32::from_num(self.gyros.borrow().len()))
    }

    pub fn angular_velocity(&self) -> Ev3Result<I32F32> {
        let mut sum = I32F32::from_num(0.0);
        for (gyro, _) in self.gyros.borrow().iter() {
            sum += I32F32::from_num(gyro.angular_velocity()?);
        }

        Ok(sum / I32F32::from_num(self.gyros.borrow().len()))
    }

    pub fn reset(&self) -> Ev3Result<()> {
        for (gyro, heading) in self.gyros.borrow_mut().iter_mut() {
            *heading = gyro.heading()?;
        }
        Ok(())
    }
}
