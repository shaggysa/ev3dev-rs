use crate::tools::wait;
use scopeguard::defer;
use std::time::Duration;

use crate::parameters::{Direction, Ev3Result, MotorPort, Stop};
use ev3dev_lang_rust::motors::TachoMotor;

pub struct Motor {
    pub(crate) motor: TachoMotor,
    counts_per_rot: i32,
}

impl Motor {
    pub fn new(port: MotorPort, direction: Direction) -> Ev3Result<Self> {
        let motor = TachoMotor::get(port)?;
        let counts_per_rot = motor.get_count_per_rot()?;
        if direction == Direction::CounterClockWise {
            motor.set_polarity("inversed")?;
        }

        motor.set_position(0)?;

        Ok(Motor {
            motor,
            counts_per_rot,
        })
    }

    pub fn get_angle(&self) -> Ev3Result<f32> {
        Ok((self.counts_per_rot * self.motor.get_position()?) as f32 / 360.0)
    }

    async fn wait_for_stop(&self) -> Ev3Result<()> {
        defer! {
            _ = self.motor.stop();
        }

        while self.motor.is_running().is_ok_and(|f| f) {
            wait(Duration::from_millis(5)).await;
        }

        Ok(())
    }

    pub async fn run_angle(&self, angle: i32, speed: i32) -> Ev3Result<()> {
        self.motor
            .set_speed_sp((speed * self.counts_per_rot) / 360)?;

        self.motor.run_to_rel_pos(Some(angle))?;

        self.wait_for_stop().await
    }

    pub async fn run_target(&self, target: i32, speed: i32) -> Ev3Result<()> {
        self.motor
            .set_speed_sp((speed * self.counts_per_rot) / 360)?;

        self.motor
            .run_to_abs_pos(Some(target * self.counts_per_rot / 360))?;

        self.wait_for_stop().await
    }

    pub async fn run_time(&self, time: Duration, speed: i32) -> Ev3Result<()> {
        self.motor.set_speed_sp(speed * self.counts_per_rot / 360)?;

        self.motor.run_timed(Some(time))?;

        self.wait_for_stop().await
    }

    pub async fn run_until_stalled(&self, speed: i32) -> Ev3Result<()> {
        defer! {
            _ = self.motor.stop()
        }

        self.motor
            .set_speed_sp((speed * self.motor.get_count_per_rot()?) / 360)?;

        self.motor.run_forever()?;

        while self.motor.is_stalled().is_ok_and(|f| !f) {
            wait(Duration::from_millis(5)).await;
        }

        Ok(())
    }

    pub async fn run_while<F>(&self, speed: i32, condition: F) -> Ev3Result<()>
    where
        F: Fn() -> bool,
    {
        defer! {
            _ = self.motor.stop();
        }

        self.motor
            .set_speed_sp((speed * self.motor.get_count_per_rot()?) / 360)?;

        self.motor.run_forever()?;

        while condition() {
            wait(Duration::from_millis(5)).await;
        }

        Ok(())
    }

    pub fn run(&self, speed: i32) -> Ev3Result<()> {
        self.motor.set_speed_sp(speed)?;
        self.motor.run_forever()
    }

    pub fn dc(&self, duty: i32) -> Ev3Result<()> {
        self.motor.set_duty_cycle_sp(duty)?;
        self.motor.run_forever()
    }

    pub fn stop(&self) -> Ev3Result<()> {
        self.motor.set_stop_action("coast")?;
        self.motor.stop()
    }

    pub fn brake(&self) -> Ev3Result<()> {
        self.motor.set_stop_action("brake")?;
        self.motor.stop()
    }

    pub fn hold(&self) -> Ev3Result<()> {
        self.motor.set_stop_action("hold")?;
        self.motor.stop()
    }

    pub fn set_stop_action(&self, action: Stop) -> Ev3Result<()> {
        let str = match action {
            Stop::Coast => "stop",
            Stop::Brake => "brake",
            Stop::Hold => "hold",
        };
        self.motor.set_stop_action(str)
    }
}
