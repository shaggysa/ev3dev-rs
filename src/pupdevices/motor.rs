use fixed::types::I32F32;
use scopeguard::defer;
use tokio::time::interval;

use crate::{
    attribute::AttributeName,
    enum_str,
    enum_string::AsStr,
    error::{Ev3Error, Ev3Result},
    motor_driver::MotorDriver,
    parameters::{Direction, MotorPort, Stop},
};
use std::{cell::Cell, collections::HashSet, str::FromStr, time::Duration};

enum_str! {
    Command,

    (RunForever, "run-forever"),
    (RunToAbsolutePosition, "run-to-abs-pos"),
    (RunToRelativePosition, "run-to-rel-pos"),
    (RunTimed, "run-timed"),
    (RunDirect, "run-direct"),
    (Stop, "stop"),
    (Reset, "reset"),
}

enum_str! {
    State,

    (Running, "running"),
    (Ramping, "ramping"),
    (Holding, "holding"),
    (Overloaded, "overloaded"),
    (Stalled, "stalled"),
}

/// NXT motor, EV3 large and medium motors
#[allow(dead_code)]
pub struct Motor {
    driver: MotorDriver,
    direction: Direction,
    last_command: Cell<Option<Command>>,
    count_per_rot: u32,
    count_per_degree: u32,
    pub(crate) max_speed: I32F32,
}

impl Motor {
    /// Tries to find a `Motor` on the given port
    ///
    /// If no motor is found, returns `MotorNotFound`.
    ///
    /// Note that the motor is not reset upon initialization.
    ///
    /// #Examples
    ///
    /// ```
    /// use ev3dev_rs::pupdevices::Motor;
    /// use ev3dev_rs::parameters::{MotorPort, Direction};
    ///
    /// let motor = Motor::new(MotorPort::OutA, Direction::Clockwise);
    /// motor.reset()?;
    /// motor.run_target(300, 360)?;
    /// ```
    pub fn new(port: MotorPort, direction: Direction) -> Ev3Result<Self> {
        let driver = MotorDriver::new(port)?;

        // reset the motor upon initialization
        driver.set_attribute_enum(AttributeName::Command, Command::Reset)?;

        driver.set_attribute_enum(AttributeName::Polarity, direction)?;

        let count_per_rot: u32 = driver
            .read_attribute(AttributeName::CountPerRotation)?
            .parse()?;

        Ok(Self {
            driver,
            direction,
            last_command: Cell::new(None),
            count_per_rot,
            count_per_degree: count_per_rot / 360,
            max_speed: I32F32::from_num(1000),
        })
    }

    fn get_states(&self) -> Ev3Result<HashSet<State>> {
        let mut states = HashSet::new();

        for flag in self
            .driver
            .read_attribute(AttributeName::State)?
            .split_ascii_whitespace()
        {
            if let Ok(state) = State::from_str(flag) {
                states.insert(state);
            }
        }

        Ok(states)
    }

    fn send_command(&self, command: Command) -> Ev3Result<()> {
        self.driver
            .set_attribute_enum(AttributeName::Command, command)?;
        self.last_command.set(Some(command));
        Ok(())
    }

    /// sets the stop action for the `Motor`
    pub fn set_stop_action(&self, action: Stop) -> Ev3Result<()> {
        self.driver
            .set_attribute_enum(AttributeName::StopAction, action)
    }

    async fn wait_for_stop(&self) -> Ev3Result<()> {
        defer! {
            _ = self.send_command(Command::Stop);
        }

        let mut timer = interval(Duration::from_millis(5));

        // the first tick completes immediately
        timer.tick().await;

        while self.get_states()?.contains(&State::Running) {
            timer.tick().await;
        }

        Ok(())
    }

    fn set_speed(&self, speed: i32) -> Ev3Result<()> {
        self.driver
            .set_attribute(AttributeName::SpeedSetpoint, speed)
    }

    pub fn set_ramp_up_setpoint(&self, sp: u32) -> Ev3Result<()> {
        self.driver.set_attribute(AttributeName::RampUpSetpoint, sp)
    }

    pub fn set_ramp_down_setpoint(&self, sp: u32) -> Ev3Result<()> {
        self.driver
            .set_attribute(AttributeName::RampDownSetpoint, sp)
    }

    /// Resets all the motor parameters to their default values.
    ///
    /// This also has the effect of stopping the motor.
    pub fn reset(&self) -> Ev3Result<()> {
        self.send_command(Command::Reset)?;
        self.driver
            .set_attribute_enum(AttributeName::Polarity, self.direction)
    }

    /// Stops the motor with the previously selected stop action.
    pub fn stop_prev_action(&self) -> Ev3Result<()> {
        self.send_command(Command::Stop)
    }

    /// Stops the motor and lets it spin freely.
    ///
    /// This also has the effect of setting the motor's stop action to `coast`.
    pub fn stop(&self) -> Ev3Result<()> {
        self.set_stop_action(Stop::Coast)?;
        self.send_command(Command::Stop)
    }

    /// Stops the motor and passively brakes it by generating current.
    ///
    /// This also has the effect of setting the motor's stop action to `brake`.
    pub fn brake(&self) -> Ev3Result<()> {
        self.set_stop_action(Stop::Brake)?;
        self.send_command(Command::Stop)
    }

    /// Stops the motor and actively holds it at its current angle.
    ///
    /// This also has the effect of setting the motor's stop action to `hold`.
    pub fn hold(&self) -> Ev3Result<()> {
        self.set_stop_action(Stop::Hold)?;
        self.send_command(Command::Stop)
    }

    /// Gets the rotation angle of the motor.
    pub fn angle(&self) -> Ev3Result<i32> {
        Ok(self
            .driver
            .read_attribute(AttributeName::Position)?
            .parse::<i32>()?
            / self.count_per_degree as i32)
    }

    /// Runs the motor at a constant speed by a given angle.
    pub async fn run_angle(&self, speed: i32, rotation_angle: i32) -> Ev3Result<()> {
        self.set_speed(speed)?;

        self.driver
            .set_attribute(AttributeName::PositionSetpoint, rotation_angle)?;

        self.send_command(Command::RunToRelativePosition)?;

        self.wait_for_stop().await
    }

    /// Runs the motor at a constant speed to a torwards a target angle.
    ///
    /// Note that the angle is continuous and does not wrap around at 360 degrees.
    ///
    /// Additionally, the motor's position is not necessarily zero at the start of your program.
    ///
    /// To guarantee that the starting position is zero, you can use the `reset` method.
    pub async fn run_target(&self, speed: i32, target_angle: i32) -> Ev3Result<()> {
        self.set_speed(speed)?;

        self.driver
            .set_attribute(AttributeName::PositionSetpoint, target_angle)?;

        self.send_command(Command::RunToAbsolutePosition)?;

        self.wait_for_stop().await
    }

    /// Runs the motor at a constant speed.
    ///
    /// The motor will run at this speed until manually stopped or you give it a new command.
    pub fn run(&self, speed: i32) -> Ev3Result<()> {
        self.set_speed(speed)?;

        self.send_command(Command::RunForever)
    }

    /// Runs the motor at a constant speed for a given duration.
    pub async fn run_time(&self, speed: i32, time: Duration) -> Ev3Result<()> {
        self.set_speed(speed)?;

        self.driver
            .set_attribute(AttributeName::TimeSetpoint, time.as_millis())?;

        self.send_command(Command::RunTimed)?;

        self.wait_for_stop().await
    }

    /// Runs at a given duty cycle percentage (-100 to 100) until stalled.
    pub async fn run_until_stalled(&self, power: i32) -> Ev3Result<()> {
        defer! {
            _ = self.send_command(Command::Stop)
        }

        self.dc(power)?;
        let mut timer = interval(Duration::from_millis(5));

        let mut states = self.get_states()?;

        // the first tick completes immediately
        timer.tick().await;

        while states.contains(&State::Running) && !states.contains(&State::Stalled) {
            timer.tick().await;
            states = self.get_states()?;
        }

        Ok(())
    }

    /// Rotates the motor at a given duty cycle percentage (-100 to 100) until stopped or you give it a new command.
    pub fn dc(&self, duty: i32) -> Ev3Result<()> {
        if self.last_command.get() != Some(Command::RunDirect) {
            self.send_command(Command::RunDirect)?;
        }

        self.driver
            .set_attribute(AttributeName::DutyCycleSetpoint, duty)
    }
}
