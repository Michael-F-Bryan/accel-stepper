#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use libm::F32Ext;

use crate::{Device, StepContext, SystemClock};
use core::f32::EPSILON;
use core::time::Duration;

/// A stepper motor driver.
#[derive(Debug, PartialEq)]
pub struct Driver {
    max_speed: f32,
    acceleration: f32,
    current_position: i64,
    step_interval: Duration,
    speed: f32,
    target_position: i64,
    last_step_time: Duration,

    /// The step counter for speed calculations
    step_counter: i64,
    initial_step_size: Duration,
    last_step_size: Duration,
    /// Min step size based on `max_speed`.
    min_step_size: Duration,
}

impl Driver {
    pub fn new() -> Driver {
        Driver {
            max_speed: 1.0,
            acceleration: 10.0,
            current_position: 0,
            step_interval: Duration::default(),
            speed: 0.0,
            target_position: 0,
            step_counter: 0,
            initial_step_size: Duration::default(),
            min_step_size: Duration::default(),
            last_step_size: Duration::default(),
            last_step_time: Duration::default(),
        }
    }

    /// Move to the specified location relative to the zero point (typically
    /// set when calibrating using [`Driver::set_current_position()`]).
    #[inline]
    pub fn move_to(&mut self, location: i64) {
        if self.target_position() != location {
            self.target_position = location;
            self.compute_new_speed();
        }
    }

    /// Move forward by the specified number of steps.
    #[inline]
    pub fn move_by(&mut self, delta: i64) {
        self.move_to(self.current_position() + delta);
    }

    /// Set the maximum permitted speed in `steps/second`.
    ///
    /// # Caution
    ///
    /// the maximum speed achievable depends on your processor and clock speed.
    /// The default max speed is `1.0` step per second.
    #[inline]
    pub fn set_max_speed(&mut self, steps_per_second: f32) {
        debug_assert!(steps_per_second > 0.0);

        self.max_speed = steps_per_second;
        self.min_step_size = Duration::from_secs_f32_2(steps_per_second.recip());
    }

    /// Get the maximum speed.
    #[inline]
    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }

    /// Set the acceleration/deceleration rate (in `steps/sec/sec`).
    #[inline]
    pub fn set_acceleration(&mut self, acceleration: f32) {
        if acceleration == 0.0 {
            return;
        }

        let acceleration = acceleration.abs();

        if (self.acceleration - acceleration).abs() > EPSILON {
            // Recompute step_counter per Equation 17
            self.step_counter =
                (self.step_counter as f32 * self.acceleration / acceleration) as i64;
            // New initial_step_size per Equation 7, with correction per Equation 15
            let initial_step_size = 0.676 * (2.0 / acceleration).sqrt();
            self.initial_step_size = Duration::from_secs_f32_2(initial_step_size);
            self.acceleration = acceleration;
            self.compute_new_speed();
        }
    }

    /// Get the acceleration/deceleration rate.
    #[inline]
    pub fn acceleration(&self) -> f32 {
        self.acceleration
    }

    /// Set the desired constant speed in `steps/sec`.
    ///
    /// Speeds of more than 1000 steps per second are unreliable. Very slow
    /// speeds may be set (eg 0.00027777 for once per hour, approximately). Speed
    /// accuracy depends on the system's clock. Jitter depends on how frequently
    /// you call the [`Driver::poll_speed()`] method. The speed will be limited
    /// by the current value of [`Driver::max_speed()`].
    pub fn set_speed(&mut self, speed: f32) {
        if (speed - self.speed).abs() < EPSILON {
            return;
        }

        let speed = Clamp::clamp(speed, -self.max_speed, self.max_speed);

        if speed == 0.0 || !speed.is_finite() {
            self.step_interval = Duration::default();
        } else {
            let duration_nanos = (1e9 / speed).abs().round();
            self.step_interval = Duration::from_nanos(duration_nanos as u64);
        }

        self.speed = speed;
    }

    /// Get the most recently set speed.
    #[inline]
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Get the number of steps to go until reaching the target position.
    #[inline]
    pub fn distance_to_go(&self) -> i64 {
        self.target_position() - self.current_position()
    }

    /// Get the most recently set target position.
    #[inline]
    pub fn target_position(&self) -> i64 {
        self.target_position
    }

    /// Reset the current motor position so the current location is considered
    /// the new `0` position.
    ///
    ///  Useful for setting a zero position on a stepper after an initial
    /// hardware positioning move.
    #[inline]
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
    #[inline]
    pub fn current_position(&self) -> i64 {
        self.current_position
    }

    /// Sets a new target position that causes the stepper to stop as quickly as
    /// possible, using the current speed and acceleration parameters.
    #[inline]
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
    #[inline]
    pub fn is_running(&self) -> bool {
        self.speed == 0.0 && self.target_position() == self.current_position()
    }

    fn compute_new_speed(&mut self) {
        let distance_to = self.distance_to_go();
        let distance_to_stop = (self.speed() * self.speed()) / (2.0 * self.acceleration());
        let steps_to_stop = distance_to_stop.round() as i64;

        if distance_to == 0 && steps_to_stop <= 1 {
            // We are at the target and its time to stop
            self.step_interval = Duration::default();
            self.speed = 0.0;
            self.step_counter = 0;
            return;
        }

        if distance_to > 0 {
            // the target is in front of us
            // We need to go forwards, maybe decelerate now?
            if self.step_counter > 0 {
                // Currently accelerating, need to decel now? Or maybe going the wrong way?
                if steps_to_stop >= distance_to || distance_to < 0 {
                    self.step_counter = -steps_to_stop; // start decelerating
                }
            } else if self.step_counter < 0 {
                // Currently decelerating, need to accel again?
                if steps_to_stop < distance_to && distance_to > 0 {
                    self.step_counter = -self.step_counter; // start accelerating
                }
            }
        } else if distance_to < 0 {
            // we've gone past the target and need to go backwards. Maybe
            // decelerating.
            if self.step_counter > 0 {
                // Currently accelerating, need to decel now? Or maybe going the wrong way?
                if steps_to_stop >= -distance_to || distance_to > 0 {
                    self.step_counter = -steps_to_stop;
                }
            } else if self.step_counter < 0 {
                // currently decelerating, need to accel again?
                if steps_to_stop < -distance_to && distance_to < 0 {
                    self.step_counter = -self.step_counter;
                }
            }
        }

        if self.step_counter == 0 {
            // This is the first step after having stopped
            self.last_step_size = self.initial_step_size;
        } else {
            // Subsequent step. Works for accel (n is +_ve) and decel (n is -ve).
            let last_step_size = self.last_step_size.as_secs_f32_2();
            let last_step_size =
                last_step_size - last_step_size * 2.0 / ((4.0 * self.step_counter as f32) + 1.0);
            self.last_step_size = Duration::from_secs_f32_2(last_step_size);
            if self.last_step_size < self.min_step_size {
                self.last_step_size = self.min_step_size;
            }
        }

        self.step_counter += 1;
        self.step_interval = self.last_step_size;
        self.speed = self.last_step_size.as_secs_f32_2().recip();

        if distance_to < 0 {
            self.speed *= -1.0;
        }
    }

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
    #[inline]
    pub fn poll<C, D>(&mut self, device: D, clock: &C) -> Result<(), D::Error> 
        where C: SystemClock,
            D: Device,
    {
        if self.poll_speed(device, clock)? {
            self.compute_new_speed();
        }

        Ok(())
    }

    /// Poll the motor and step it if a step is due, implementing a constant
    /// speed as set by the most recent call to [`Driver::set_speed()`].
    ///
    /// You must call this as frequently as possible, but at least once per step
    /// interval, returns true if the motor was stepped.
    pub fn poll_speed<C, D>(&mut self, mut device: D, clock: &C) -> Result<bool, D::Error> 
        where C: SystemClock,
            D: Device,
    {
        // Dont do anything unless we actually have a step interval
        if self.step_interval == Duration::default() {
            return Ok(false);
        }

        let now = clock.elapsed();

        if now - self.last_step_time >= self.step_interval {
            // we need to take a step

            // Note: we can't assign to current_position directly because we
            // a failed step shouldn't update any internal state
            let new_position = if self.distance_to_go() > 0 {
                self.current_position + 1
            } else {
                self.current_position - 1
            };

            let ctx = StepContext {
                position: new_position,
                step_time: now,
            };
            device.step(&ctx)?;

            self.current_position = new_position;
            self.last_step_time = now; // Caution: does not account for costs in step()

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for Driver {
    fn default() -> Driver {
        Driver::new()
    }
}

trait Clamp {
    fn clamp(self, lower: Self, upper: Self) -> Self;
}

impl<C: PartialOrd> Clamp for C {
    fn clamp(self, lower: Self, upper: Self) -> Self {
        if self < lower {
            lower
        } else if upper < self {
            upper
        } else {
            self
        }
    }
}

/// Workarounds because working with `Duration` and `f32` requires nightly (see
/// the `duration_float` feature).
trait DurationHelpers {
    fn from_secs_f32_2(secs: f32) -> Self;

    fn as_secs_f32_2(&self) -> f32;
}

const NANOS_PER_SEC: u32 = 1_000_000_000;

impl DurationHelpers for Duration {
    fn from_secs_f32_2(secs: f32) -> Self {
        // copied straight from libcore/time.rs

        let nanos = secs * (NANOS_PER_SEC as f32);
        assert!(nanos.is_finite());

        let nanos = nanos as u128;
        Duration::new(
            (nanos / u128::from(NANOS_PER_SEC)) as u64,
            (nanos % u128::from(NANOS_PER_SEC)) as u32,
        )
    }

    fn as_secs_f32_2(&self) -> f32 {
        (self.as_secs() as f32) + (self.subsec_nanos() as f32) / (NANOS_PER_SEC as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug, Copy, Clone, PartialEq, Default)]
    struct NopDevice;

    impl Device for NopDevice {
        type Error = ();

        fn step(&mut self, _ctx: &StepContext) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct DummyClock {
        ticks: Cell<u32>,
    }

    impl SystemClock for DummyClock {
        fn elapsed(&self) -> Duration {
            let ticks = self.ticks.get();
            self.ticks.set(ticks + 1);

            Duration::new(ticks as u64, 0)
        }
    }

    #[test]
    fn compute_new_speeds_when_already_at_target() {
        let mut driver = Driver::default();
        driver.target_position = driver.current_position;

        driver.compute_new_speed();

        assert_eq!(driver.speed(), 0.0);
        assert_eq!(driver.step_interval, Duration::new(0, 0));
    }

    #[test]
    fn dont_step_when_already_at_target() {
        let mut forward = 0;
        let mut back = 0;
        let clock = DummyClock::default();

        {
            let mut dev = crate::func_device(|| forward += 1, || back += 1);
            let mut driver = Driver::new();
            driver.target_position = driver.current_position;

            for _ in 0..100 {
                driver.poll(&mut dev, &clock).unwrap();
            }
        }

        assert_eq!(forward, 0);
        assert_eq!(back, 0);
    }
}
