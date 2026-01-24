use crate::pid::Pid;
use crate::pupdevices::GyroSensor;
use crate::robotics::GyroController;
use crate::Ev3Error;
use crate::{parameters::Stop, pupdevices::Motor, Ev3Result};
use fixed::traits::{LossyInto, ToFixed};
use fixed::types::I32F32;
use scopeguard::defer;
use std::cell::Cell;
use std::time::Duration;
use tokio::time::interval;

/// A pybricks-like `DriveBase`.
///
/// Using gyroscope(s) is highly recommended in order to get the most accurate actions
///
/// # Examples
///
/// ``` no_run
/// use ev3dev_rs::parameters::{Direction, MotorPort, SensorPort, Stop};
/// use ev3dev_rs::pupdevices::{Motor, GyroSensor};
/// use ev3dev_rs::robotics::DriveBase;
///
/// let left = Motor::new(MotorPort::OutA, Direction::CounterClockwise)?;
/// let right = Motor::new(MotorPort::OutD, Direction::CounterClockwise)?;
///
/// // no gyro
/// let drive = DriveBase::new(&left, &right, 62.4, 130.5)?;
///
/// // with gyro
/// let gyro = GyroSensor::new(SensorPort::In1)?;
///
/// let drive = DriveBase::new(&left, &right, 62.4, 130.5)?.with_gyro(&gyro)?;
///
/// // you have to explicitly enable the gyro
/// drive.use_gyro(true)?;
///
/// // default is 500
/// drive.set_straight_speed(600)?;
///
/// // default should be coast
/// // unlike pybricks, the stop action doesn't affect whether the robot tracks it's position and heading
/// drive.set_stop_action(Stop::Hold)?;
///
/// drive.straight(500).await?;
/// drive.turn(90).await?;
/// ```
pub struct DriveBase<'a> {
    left_motor: &'a Motor,
    right_motor: &'a Motor,
    left_start_angle: i32,
    right_start_angle: i32,
    min_speed: I32F32,
    wheel_diameter: I32F32,
    axle_track: I32F32,
    straight_speed: Cell<I32F32>,
    turn_speed: Cell<I32F32>,
    prev_encoder_heading: Cell<I32F32>,
    distance_pid: Pid,
    heading_pid: Pid,
    distance_target: Cell<I32F32>,
    heading_target: Cell<I32F32>,
    distance_tolerance: Cell<I32F32>,
    heading_tolerance: Cell<I32F32>,
    using_gyros: Cell<bool>,
    gyros: Option<GyroController<'a>>,
}

impl<'a> DriveBase<'a> {
    /// Creates a new `DriveBase` with the defined parameters.
    ///
    /// Wheel diameter and axle track are in mm.
    ///
    /// Using a gyroscope is highly recommended, see `with_gyro` or `with_gyros`.
    pub fn new<Number>(
        left_motor: &'a Motor,
        right_motor: &'a Motor,
        wheel_diameter: Number,
        axle_track: Number,
    ) -> Ev3Result<Self>
    where
        Number: ToFixed,
    {
        left_motor.set_ramp_up_setpoint(2000)?;
        right_motor.set_ramp_up_setpoint(2000)?;

        left_motor.set_ramp_down_setpoint(1800)?;
        right_motor.set_ramp_down_setpoint(1800)?;

        Ok(Self {
            left_motor,
            right_motor,
            left_start_angle: left_motor.angle()?,
            right_start_angle: right_motor.angle()?,
            min_speed: I32F32::from_num(100),
            wheel_diameter: I32F32::from_num(wheel_diameter),
            axle_track: I32F32::from_num(axle_track),
            straight_speed: Cell::new(I32F32::from_num(500)),
            turn_speed: Cell::new(I32F32::from_num(550)),
            prev_encoder_heading: Cell::new(I32F32::ZERO),
            distance_pid: Pid::new(10, 0, 8, 0, 0),
            heading_pid: Pid::new(10, 0, 5, 0, 0),
            distance_target: Cell::new(I32F32::ZERO),
            heading_target: Cell::new(I32F32::ZERO),
            distance_tolerance: Cell::new(I32F32::from_num(4)),
            heading_tolerance: Cell::new(I32F32::from_num(0.75)),
            using_gyros: Cell::new(false),
            gyros: None,
        })
    }

    /// Adds a single gyro sensor to the `DriveBase`.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// use ev3dev_rs::parameters::{Direction, MotorPort, SensorPort, Stop};
    ///
    /// use ev3dev_rs::pupdevices::{Motor, GyroSensor};
    ///
    /// use ev3dev_rs::robotics::DriveBase;
    ///
    /// let left = Motor::new(MotorPort::OutA, Direction::CounterClockwise)?;
    ///
    /// let right = Motor::new(MotorPort::OutD, Direction::CounterClockwise)?;
    ///
    /// let gyro = GyroSensor::new(SensorPort::In1)?;
    ///
    /// let drive = DriveBase::new(&left, &right, 62.4, 130.5)?.with_gyro(&gyro)?;
    ///
    /// // you have to explicitly enable the gyro
    ///
    /// drive.use_gyro(true)?;
    /// ```
    pub fn with_gyro<'b>(mut self, gyro_sensor: &'b GyroSensor) -> Ev3Result<Self>
    where
        'b: 'a,
    {
        self.gyros = Some(GyroController::new(vec![gyro_sensor])?);
        Ok(self)
    }

    /// Adds multiple gyro sensors to the `DriveBase`.
    ///
    /// # Examples
    ///
    /// ``` no_run
    /// use ev3dev_rs::parameters::{Direction, MotorPort, SensorPort, Stop};
    ///
    /// use ev3dev_rs::pupdevices::{Motor, GyroSensor};
    ///
    /// use ev3dev_rs::robotics::DriveBase;
    ///
    /// let left = Motor::new(MotorPort::OutA, Direction::CounterClockwise)?;
    ///
    /// let right = Motor::new(MotorPort::OutD, Direction::CounterClockwise)?;
    ///
    /// let left_gyro = GyroSensor::new(SensorPort::In1)?;
    ///
    /// let right_gyro = GyroSensor::new(SensorPort::In4)?;
    ///
    /// let drive = DriveBase::new(&left, &right, 62.4, 130.5)?.with_gyros(vec![ &left_gyro, &right_gyro ])?;
    ///
    /// // you have to explicitly enable the gyro
    ///
    /// drive.use_gyro(true)?;
    /// ```
    pub fn with_gyros<'b>(mut self, gyro_sensors: Vec<&'b GyroSensor>) -> Ev3Result<Self>
    where
        'b: 'a,
    {
        self.gyros = Some(GyroController::new(gyro_sensors)?);
        Ok(self)
    }

    /// True makes the `DriveBase` use the gyro, while false makes the `DriveBase` use the motor encoders.
    ///
    /// Using the gyro is highly recommended for accurate drive actions.
    pub fn use_gyro(&self, use_gyro: bool) -> Ev3Result<()> {
        if use_gyro && self.gyros.is_none() {
            return Err(Ev3Error::NoSensorProvided);
        }
        self.using_gyros.set(use_gyro);
        Ok(())
    }

    /// Sets the straight speed in motor degrees per second.
    ///
    /// The default is 500 and the max is 1000.
    pub fn set_straight_speed<Number>(&self, straight_speed: Number)
    where
        Number: ToFixed,
    {
        self.straight_speed.set(I32F32::from_num(straight_speed));
    }

    /// Sets the max turn speed in motor degrees per second.
    ///
    /// The default is 500 and the max is 1000.
    pub fn set_turn_speed<Number>(&self, turn_speed: Number)
    where
        Number: ToFixed,
    {
        self.turn_speed.set(I32F32::from_num(turn_speed));
    }

    /// Units are in milliseconds and must be positive.
    ///
    /// When set to a non-zero value, the motor speed will increase from 0 to 100% of max_speed over the span of this setpoint.
    ///
    /// This is especially useful for avoiding wheel slip.
    ///
    /// The default for `DriveBase` motors 2000.
    pub fn set_ramp_up_setpoint(&self, sp: u32) -> Ev3Result<()> {
        self.left_motor.set_ramp_up_setpoint(sp)?;
        self.right_motor.set_ramp_up_setpoint(sp)
    }

    /// Units are in milliseconds and must be positive.
    ///
    /// When set to a non-zero value, the motor speed will decrease from 0 to 100% of max_speed over the span of this setpoint.
    ///
    /// This is especially useful for avoiding wheel slip.
    ///
    /// The default for `DriveBase` motors 1800.
    pub fn set_ramp_down_setpoint(&self, sp: u32) -> Ev3Result<()> {
        self.left_motor.set_ramp_down_setpoint(sp)?;
        self.right_motor.set_ramp_down_setpoint(sp)
    }
    /// Sets the stop action of the `DriveBase`
    ///
    /// Unlike pybricks, this doesn't affect whether the `DriveBase` tracks heading and distance
    pub fn set_stop_action(&self, action: Stop) -> Ev3Result<()> {
        self.left_motor.set_stop_action(action)?;
        self.right_motor.set_stop_action(action)
    }

    /// Sets the distance PID settings
    ///
    /// default: 10, 0, 8, 0, 0
    pub fn distance_pid_settings<Number>(
        &self,
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) where
        Number: ToFixed,
    {
        self.distance_pid
            .settings(kp, ki, kd, integral_deadzone, integral_rate);
    }

    /// Sets the heading PID settings
    ///
    /// default: 10, 0, 5, 0, 0
    pub fn heading_pid_settings<Number>(
        &self,
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) where
        Number: ToFixed,
    {
        self.heading_pid
            .settings(kp, ki, kd, integral_deadzone, integral_rate);
    }

    /// Stops the `DriveBase` with the selected stop action.
    ///
    /// Async driving actions automatically do this
    pub fn stop(&self) -> Ev3Result<()> {
        self.left_motor.stop_prev_action()?;
        self.right_motor.stop_prev_action()
    }

    async fn drive_relative(&self, distance_mm: I32F32, angle_deg: I32F32) -> Ev3Result<()> {
        defer! {
            _ = self.stop()
        }

        self.distance_pid.reset();
        self.heading_pid.reset();

        let target_distance = self.distance_target.get() + distance_mm;
        let target_heading = self.heading_target.get() + angle_deg;

        self.distance_target.set(target_distance);
        self.heading_target.set(target_heading);

        let mut timer = interval(Duration::from_millis(5));

        // the first tick completes immediately
        timer.tick().await;

        let straight_speed = self.straight_speed.get();
        let turn_speed = self.turn_speed.get();

        loop {
            let left_angle = I32F32::from_num(self.left_motor.angle()? - self.left_start_angle);
            let right_angle = I32F32::from_num(self.right_motor.angle()? - self.right_start_angle);
            let current_distance = self.encoders_to_distance(left_angle, right_angle);
            let current_heading = if self.using_gyros.get()
                && let Some(ref gyro) = self.gyros
            {
                let encoders = self.encoders_to_heading()?;
                I32F32::from_num(gyro.heading()?) * I32F32::from_num(0.9)
                    + encoders * I32F32::from_num(0.1)
            } else {
                self.encoders_to_heading()?
            };

            let distance_error = target_distance - current_distance;
            let heading_error = target_heading - current_heading;

            if distance_error.abs() < self.distance_tolerance.get()
                && heading_error.abs() < self.heading_tolerance.get()
            {
                break;
            }

            let dive_effort = self.distance_pid.next(distance_error);
            let turn_effort = -self.heading_pid.next(heading_error);

            let drive_speed_out = dive_effort * straight_speed;
            let turn_speed_out = turn_effort * turn_speed;

            let left_speed = (drive_speed_out - turn_speed_out)
                .clamp(-self.right_motor.max_speed, self.left_motor.max_speed);

            let right_speed = (drive_speed_out + turn_speed_out)
                .clamp(-self.left_motor.max_speed, self.right_motor.max_speed);

            self.left_motor.run(
                (if left_speed.abs() < self.min_speed {
                    self.min_speed * left_speed.signum()
                } else {
                    left_speed
                })
                .lossy_into(),
            )?;
            self.right_motor.run(
                (if right_speed.abs() < self.min_speed {
                    self.min_speed * right_speed.signum()
                } else {
                    right_speed
                })
                .lossy_into(),
            )?;

            timer.tick().await;
        }

        Ok(())
    }

    /// Drives straight by the given distance in mm.
    pub async fn straight<Number>(&self, distance: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        self.drive_relative(I32F32::from_num(distance), I32F32::from_num(0))
            .await
    }

    /// Turns by the angle in degrees.
    pub async fn turn<Number>(&self, angle: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        self.drive_relative(I32F32::from_num(0), I32F32::from_num(angle))
            .await
    }

    /// Curves with the given radius and a target angle.
    pub async fn curve<Number>(&self, radius: Number, angle: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        let fixed_angle = I32F32::from_num(angle);

        let angle_rad = fixed_angle * I32F32::PI / 180;
        let arc_length = I32F32::from_num(radius).abs() * I32F32::from_num(angle_rad).abs();

        self.drive_relative(arc_length, I32F32::from_num(fixed_angle))
            .await
    }

    /// Turns with the given radius and distance.
    pub async fn veer<Number>(&self, radius: Number, distance: Number) -> Ev3Result<()>
    where
        Number: ToFixed,
    {
        let fixed_distance = I32F32::from_num(distance);

        let angle_rad = fixed_distance / I32F32::from_num(radius);
        let angle_deg = angle_rad * 180 / I32F32::PI;

        self.drive_relative(fixed_distance, I32F32::from_num(angle_deg))
            .await
    }

    /// Experimental function to find the best axle track for the robot
    ///
    /// This should print out the ideal axle track once it is finished testing.
    ///
    /// If you are having trouble with inaccurate heading readings due to wheel slipping, see `set_ramp_up_setpoint`.
    ///
    /// Note that the value can vary wildly based on surface.
    pub async fn find_calibrated_axle_track<Number>(
        &mut self,
        margin_of_error: Number,
    ) -> Ev3Result<I32F32>
    where
        Number: ToFixed,
    {
        self.use_gyro(true)?;
        let fixed_estimate = I32F32::from_num(self.axle_track);
        let fixed_margin_of_error = I32F32::from_num(margin_of_error);
        let resphi = I32F32::FRAC_1_PHI;

        let mut a = fixed_estimate - fixed_margin_of_error;
        let mut b = fixed_estimate + fixed_margin_of_error;
        let tolerance = I32F32::from_num(0.5);

        // Initial test points
        let mut x1 = a + resphi * (b - a);
        let mut x2 = b - resphi * (b - a);

        let mut f1 = self.test_axle_track(x1).await?;
        let mut f2 = self.test_axle_track(x2).await?;

        // Golden section search loop
        while (b - a).abs() > tolerance {
            if f1 < f2 {
                // Minimum is between a and x2
                a = x2;
                x2 = x1;
                f2 = f1;
                x1 = a + resphi * (b - a);
                f1 = self.test_axle_track(x1).await?;
            } else {
                // Minimum is between x1 and b
                b = x1;
                x1 = x2;
                f1 = f2;
                x2 = b - resphi * (b - a);
                f2 = self.test_axle_track(x2).await?;
            }
        }

        let best = (a + b) / I32F32::from_num(2);
        println!("Best axle track: {}", best);

        Ok(best)
    }

    // Helper function to test a single axle track value
    async fn test_axle_track(&mut self, candidate: I32F32) -> Ev3Result<I32F32> {
        println!("testing {}", candidate);
        self.axle_track = candidate;

        // Reset to known position
        if let Some(ref gyros) = self.gyros {
            let start_encoder_heading = self.encoders_to_heading()?;
            gyros.reset()?;
            // Do a test turn (90 degrees)
            self.turn(90).await?;

            // Measure error
            let gyro_turned = gyros.heading()?;
            let encoder_turned = self.encoders_to_heading()? - start_encoder_heading;

            let error = (gyro_turned - encoder_turned).abs();
            println!("Error: {}", error);

            Ok(error)
        } else {
            Err(Ev3Error::NoSensorProvided)
        }
    }

    // Convert encoder positions to distance traveled (average of both wheels)
    fn encoders_to_distance(&self, left_deg: I32F32, right_deg: I32F32) -> I32F32 {
        let wheel_circ = I32F32::PI * self.wheel_diameter;
        let left_mm = wheel_circ * left_deg / 360;
        let right_mm = wheel_circ * right_deg / 360;
        (left_mm + right_mm) / 2
    }

    /// Convert encoder positions to heading (differential between wheels)
    fn encoders_to_heading(&self) -> Ev3Result<I32F32> {
        let left_deg = I32F32::from_num(self.left_motor.angle()? - self.left_start_angle);
        let right_deg = I32F32::from_num(self.right_motor.angle()? - self.right_start_angle);

        let wheel_circ = I32F32::PI * self.wheel_diameter;
        let left_mm = wheel_circ * left_deg / 360;
        let right_mm = wheel_circ * right_deg / 360;
        let arc_diff = left_mm - right_mm;
        let turn_rad = arc_diff / self.axle_track;
        let raw_heading = turn_rad * 180 / I32F32::PI;

        let alpha = I32F32::from_num(0.7);
        let filtered =
            alpha * raw_heading + (I32F32::from_num(1) - alpha) * self.prev_encoder_heading.get();
        self.prev_encoder_heading.set(filtered);
        Ok(filtered)
    }
}
