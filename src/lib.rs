#![no_std]

/// A high-level stepper motor driver.
#[derive(Debug, PartialEq)]
pub struct Driver<D> {
    device: D,
    max_speed: f32,
    acceleration: f32,
    current_position: i64,
}

impl<D> Driver<D> {
    pub fn new(device: D) -> Driver<D> {
        Driver {
            device,
            max_speed: 1.0,
            acceleration: 10.0,
            current_position: 0,
        }
    }

    pub fn inner(&mut self) -> &mut D {
        &mut self.device
    }

    pub fn into_inner(self) -> D {
        self.device
    }

    pub fn move_to(&mut self, _location: i64) {
        unimplemented!()
    }

    /// Move forward by the specified number of steps.
    pub fn move_by(&mut self, _delta: i64) {
        unimplemented!()
    }

    /// Set the maximum permitted speed in `steps/second`.
    ///
    /// # Caution
    ///
    /// the maximum speed achievable depends on your processor and clock speed.
    /// The default max speed is `1.0` step per second.
    pub fn set_max_speed(&mut self, steps_per_second: f32) {
        debug_assert!(steps_per_second > 0.0);

        self.max_speed = steps_per_second;
    }

    /// Get the maximum speed.
    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }

    /// Set the acceleration/deceleration rate (in `steps/sec/sec`).
    pub fn set_acceleration(&mut self, _acceleration: f32) {
        unimplemented!()
    }

    /// Get the acceleration/deceleration rate.
    pub fn acceleration(&self) -> f32 {
        unimplemented!()
    }

    /// Set the desired constant speed in `steps/sec`.
    pub fn set_speed(&mut self, _speed: f32) {
        unimplemented!()
    }

    /// Get the most recently set speed.
    pub fn speed(&self) -> f32 {
        unimplemented!()
    }

    /// Get the number of steps to go until reaching the target position.
    pub fn distance_to_go(&self) -> i64 {
        unimplemented!()
    }

    /// Get the most recently set target position.
    pub fn target_position(&self) -> i64 {
        unimplemented!()
    }

    /// Reset the current motor position so the current location is considered
    /// the new 0` position.
    ///
    ///  Useful for setting a zero position on a stepper after an initial
    /// hardware positioning move.
    pub fn set_current_position(&mut self, position: i64) {
        self.current_position = position;
    }

    /// Get the current motor position, as measured by counting the number of
    /// pulses emitted.
    ///
    /// # Note
    ///
    /// Stepper motors are an open-loop system, so there's no guarantee the
    /// motor will *actually* be at that position.
    pub fn current_position(&self) -> i64 {
        self.current_position
    }

    /// Sets a new target position that causes the stepper to stop as quickly as
    /// possible, using the current speed and acceleration parameters.
    pub fn stop(&mut self) {
        unimplemented!()
    }

    /// Checks to see if the motor is currently running to a target.
    pub fn is_running(&self) -> bool {
        unimplemented!()
    }
}

impl<D: Device> Driver<D> {
    /// Poll the driver and step it if a step is due.
    ///
    /// This function must called as frequently as possoble, but at least once
    /// per minimum step time interval, preferably as part of the main loop.
    ///
    /// Note that each call to `poll()` will make at most one step, and then only
    /// when a step is due, based on the current speed and the time since the
    /// last step.
    pub fn poll(&mut self) {
        unimplemented!()
    }
}

/// An interface to the stepper motor.
pub trait Device {
    /// Take one step forwards.
    fn forward(&mut self);
    /// Take one step backwards.
    fn backward(&mut self);
}
