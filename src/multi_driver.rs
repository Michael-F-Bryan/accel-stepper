use crate::{utils::DurationHelpers, Device, Driver, SystemClock};
#[allow(unused_imports)]
use arrayvec::ArrayVec;
use core::time::Duration;

/// Controller for moving multiple axes in a coordinated fashion.
pub struct MultiDriver {
    #[cfg(feature = "std")]
    drivers: Vec<Driver>,
    #[cfg(not(feature = "std"))]
    drivers: ArrayVec<[Driver; MultiDriver::MAX_DRIVERS]>,
}

impl MultiDriver {
    /// The maximum number of [`Driver`]s that a [`MultiDriver`] can manage when
    /// compiled without the `std` feature.
    pub const MAX_DRIVERS: usize = 10;

    pub fn new() -> MultiDriver {
        MultiDriver {
            drivers: Default::default(),
        }
    }

    /// Add a new [`Driver`] to the list of synchronised axes managed by the
    /// [`MultiDriver`].
    ///
    /// # Panics
    ///
    /// When compiling without the `std` feature flag, the [`MultiDriver`] can
    /// only manage up to [`MultiDriver::MAX_DRIVERS`] drivers.
    pub fn push_driver(&mut self, driver: Driver) { self.drivers.push(driver); }

    pub fn drivers(&self) -> &[Driver] { &self.drivers }

    pub fn drivers_mut(&mut self) -> &mut [Driver] { &mut self.drivers }

    /// Set the target positions of all managed steppers.
    ///
    /// # Panics
    ///
    /// The number of managed steppers should be the same as the number of
    /// positions.
    pub fn move_to(&mut self, positions: &[i64]) {
        assert_eq!(positions.len(), self.drivers.len());

        if self.drivers.is_empty() {
            return;
        }

        // first find the stepper that will take the longest time to move
        let longest_time = self
            .drivers
            .iter()
            .zip(positions)
            .map(|(d, p)| time_to_move(d, *p))
            .max()
            .expect("There is always a least one time");

        if longest_time == Duration::new(0, 0) {
            // nothing else needs to be done
            return;
        }

        let longest_time = longest_time.as_secs_f32_2();

        // Now work out a new max speed for each stepper so they will all
        // arrived at the same time of longestTime
        for (i, driver) in self.drivers.iter_mut().enumerate() {
            let distance = positions[i] - driver.current_position();

            driver.move_to(positions[i]);
            driver.set_speed(distance as f32 / longest_time);
        }
    }

    /// Poll the underlying [`Driver`]s, emitting steps to the provided
    /// [`Device`]s when necessary.
    ///
    /// # Panics
    ///
    /// The number of managed steppers should be the same as the number of
    /// devices.
    pub fn poll<D, C>(
        &mut self,
        devices: &mut [D],
        clock: &C,
    ) -> Result<(), D::Error>
    where
        D: Device,
        C: SystemClock,
    {
        assert_eq!(devices.len(), self.drivers.len());

        for (driver, dev) in self.drivers.iter_mut().zip(devices.iter_mut()) {
            driver.poll(dev, clock)?;
        }

        Ok(())
    }

    /// Are any of the managed steppers still running?
    pub fn is_running(&self) -> bool {
        self.drivers.iter().any(|d| d.is_running())
    }
}

fn time_to_move(driver: &Driver, pos: i64) -> Duration {
    let distance = driver.current_position() - pos;

    Duration::from_secs_f32_2(distance.abs() as f32 / driver.max_speed())
}
