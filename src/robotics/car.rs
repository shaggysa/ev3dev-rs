use ev3dev_lang_rust::Ev3Result;

use crate::{parameters::Stop, pupdevices::Motor};

pub struct Car {
    steering_motor: Motor,
    drive_motor: Motor,
    left_max: i32,
    right_max: i32,
}

impl Car {
    pub async fn new(steering_motor: Motor, drive_motor: Motor) -> Ev3Result<Self> {
        steering_motor.run_until_stalled(-250).await?;

        let left_max = steering_motor.get_angle()? as i32;

        steering_motor.run_until_stalled(250).await?;

        let right_max = steering_motor.get_angle()? as i32;

        Ok(Self {
            steering_motor,
            drive_motor,
            left_max,
            right_max,
        })
    }

    pub fn steer(&self, percentage: u8) -> Ev3Result<()> {
        self.steering_motor.set_stop_action(Stop::Hold)?;
        self.steering_motor.motor.run_to_rel_pos(Some(
            self.left_max + self.right_max / 2 * (percentage / 100) as i32,
        ))
    }

    pub fn drive_speed(&self, speed: i32) -> Ev3Result<()> {
        self.drive_motor.run(speed)
    }

    pub fn drive_power(&self, power: i32) -> Ev3Result<()> {
        self.drive_motor.dc(power)
    }
}
