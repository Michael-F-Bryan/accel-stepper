#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), test))]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libm::F32Ext;

use core::time::Duration;

/// A stepper motor driver.
#[derive(Debug, PartialEq)]
pub struct Driver<D> {
    device: D,
    max_speed: f32,
    acceleration: f32,
    current_position: i64,
    step_interval: Duration,
    speed: f32,
    target_position: i64,
    n: f32,
    c0: f32,
}

impl<D> Driver<D> {
    pub fn new(device: D) -> Driver<D> {
        Driver {
            device,
            max_speed: 1.0,
            acceleration: 10.0,
            current_position: 0,
            step_interval: Duration::default(),
            speed: 0.0,
            target_position: 0,
            n: 0.0,
            c0: 0.0,
        }
    }

    pub fn inner(&mut self) -> &mut D {
        &mut self.device
    }

    pub fn into_inner(self) -> D {
        self.device
    }

    /// Move to the specified location relative to the zero point (typically
    /// set when calibrating using [`Driver::set_current_position()`]).
    pub fn move_to(&mut self, location: i64) {
        if self.target_position() != location {
            self.target_position = location;
            self.compute_new_speed();
        }
    }

    /// Move forward by the specified number of steps.
    pub fn move_by(&mut self, delta: i64) {
        self.move_to(self.current_position() + delta);
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
    pub fn set_acceleration(&mut self, acceleration: f32) {
        if acceleration == 0.0 {
            return;
        }

        let acceleration = acceleration.abs();

        if self.acceleration != acceleration {
            // Recompute _n per Equation 17
            self.n = self.n * (self.acceleration / acceleration);
            // New c0 per Equation 7, with correction per Equation 15
            self.c0 = 0.676 * (2.0 / acceleration).sqrt() * 1e6;
            self.acceleration = acceleration;
            self.compute_new_speed();
        }
    }

    /// Get the acceleration/deceleration rate.
    pub fn acceleration(&self) -> f32 {
        self.acceleration
    }

    /// Set the desired constant speed in `steps/sec`.
    pub fn set_speed(&mut self, speed: f32) {
        if speed == self.speed {
            return;
        }

        let speed = speed.constrain(-self.max_speed, self.max_speed);

        if speed == 0.0 || !speed.is_finite() {
            self.step_interval = Duration::default();
        } else {
            let duration_nanos = (1e9 / speed).abs().round();
            self.step_interval = Duration::from_nanos(duration_nanos as u64);
        }

        self.speed = speed;
    }

    /// Get the most recently set speed.
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Get the number of steps to go until reaching the target position.
    pub fn distance_to_go(&self) -> i64 {
        self.target_position() - self.current_position()
    }

    /// Get the most recently set target position.
    pub fn target_position(&self) -> i64 {
        self.target_position
    }

    /// Reset the current motor position so the current location is considered
    /// the new `0` position.
    ///
    ///  Useful for setting a zero position on a stepper after an initial
    /// hardware positioning move.
    pub fn set_current_position(&mut self, position: i64) {
        self.current_position = position;
        self.target_position = position;
        self.step_interval = Duration::default();
        self.speed = 0.0;
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
        if self.speed == 0.0 {
            return;
        }

        let stopping_distance = (self.speed * self.speed) / (2.0 * self.acceleration);
        let steps_to_stop = stopping_distance.round() as i64 + 1;

        if self.speed > 0.0 {
            self.move_by(steps_to_stop);
        } else {
            self.move_by(-steps_to_stop);
        }
    }

    /// Checks to see if the motor is currently running to a target.
    pub fn is_running(&self) -> bool {
        self.speed == 0.0 && self.target_position() == self.current_position()
    }

    fn compute_new_speed(&mut self) {
        unimplemented!()
    }
}

impl<D: Device> Driver<D> {
    /// Poll the driver and step it if a step is due.
    ///
    /// This function must called as frequently as possoble, but at least once
    /// per minimum step time interval, preferably as part of the main loop.
    ///
    /// Note that each call to [`Driver::poll()`] will make at most one step, and
    /// then only when a step is due, based on the current speed and the time
    /// since the last step.
    ///
    /// # Warning
    ///
    /// For correctness, the same [`SystemClock`] should be used every time
    /// [`Driver::poll()`] is called. Failing to do so may mess up internal
    /// timing calculations.
    pub fn poll<C: SystemClock>(&mut self, _clock: C) {
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

/// Something which records the elapsed real time.
///
/// This uses shared references because it may be shared between multiple
/// components at any one time.
pub trait SystemClock {
    /// The amount of time that has passed since a clock-specific reference
    /// point (e.g. device startup or the unix epoch).
    fn elapsed(&self) -> Duration;
}

impl<F> SystemClock for F
where
    F: Fn() -> Duration,
{
    fn elapsed(&self) -> Duration {
        self()
    }
}

/// A monotonically non-decreasing clock backed by the operating system.
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq)]
pub struct OperatingSystemClock {
    created_at: std::time::Instant,
}

#[cfg(feature = "std")]
impl OperatingSystemClock {
    pub fn new() -> OperatingSystemClock {
        OperatingSystemClock::default()
    }
}

#[cfg(feature = "std")]
impl SystemClock for OperatingSystemClock {
    fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }
}

#[cfg(feature = "std")]
impl Default for OperatingSystemClock {
    fn default() -> OperatingSystemClock {
        OperatingSystemClock {
            created_at: std::time::Instant::now(),
        }
    }
}

trait FloatHelpers {
    fn constrain(self, lower: Self, upper: Self) -> Self;
}

impl FloatHelpers for f32 {
    fn constrain(self, lower: Self, upper: Self) -> Self {
        if self < lower {
            lower
        } else if upper < self {
            upper
        } else {
            self
        }
    }
}
