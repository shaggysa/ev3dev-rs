use std::cell::Cell;
use std::cmp::min;

use fixed::{traits::ToFixed, types::I32F32};

pub(crate) struct Pid {
    kp: Cell<I32F32>,
    ki: Cell<I32F32>,
    kd: Cell<I32F32>,
    integral_deadzone: Cell<I32F32>,
    integral_rate: Cell<I32F32>,
    integral_term: Cell<I32F32>,
    prev_measurement: Cell<Option<I32F32>>,
}

impl Pid {
    pub(crate) fn new<Number>(
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) -> Self
    where
        Number: ToFixed,
    {
        Self {
            kp: Cell::new(I32F32::from_num(kp)),
            ki: Cell::new(I32F32::from_num(ki)),
            kd: Cell::new(I32F32::from_num(kd)),
            integral_deadzone: Cell::new(I32F32::from_num(integral_deadzone)),
            integral_rate: Cell::new(I32F32::from_num(integral_rate)),
            integral_term: Cell::new(I32F32::ZERO),
            prev_measurement: Cell::new(None),
        }
    }

    /// Sets the PID settings.
    ///
    /// Floating point numbers or integers can be used.
    pub fn settings<Number>(
        &self,
        kp: Number,
        ki: Number,
        kd: Number,
        integral_deadzone: Number,
        integral_rate: Number,
    ) where
        Number: ToFixed,
    {
        self.kp.set(I32F32::from_num(kp));
        self.ki.set(I32F32::from_num(ki));
        self.kd.set(I32F32::from_num(kd));
        self.integral_deadzone
            .set(I32F32::from_num(integral_deadzone));
        self.integral_rate.set(I32F32::from_num(integral_rate));
    }

    /// Resets the stored PID values without changing the settings.
    pub fn reset(&self) {
        self.integral_term.set(I32F32::ZERO);
        self.prev_measurement.set(None);
    }

    /// Calculates the next PID output based on the current measurement.
    ///
    /// A target measurement of zero and a constant polling rate are assumed.
    pub(crate) fn next<Number>(&self, measurement: Number) -> I32F32
    where
        Number: ToFixed,
    {
        let fixed_point_measurement = I32F32::from_num(measurement);

        let error = -fixed_point_measurement;
        let p = self.kp.get() * error;

        if error.abs() > self.integral_deadzone.get() {
            self.integral_term.set(
                self.integral_term.get() + min(error * self.ki.get(), self.integral_rate.get()),
            );
        }

        let d = -match self.prev_measurement.get() {
            Some(prev_measurement) => fixed_point_measurement - prev_measurement,
            None => I32F32::ZERO,
        } * self.kd.get();

        self.prev_measurement.set(Some(fixed_point_measurement));

        p + self.integral_term.get() + d
    }
}
