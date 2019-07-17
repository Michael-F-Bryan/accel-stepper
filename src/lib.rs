//! A Rust port of the popular [`AccelStepper`][original] Arduino stepper
//! library.
//! 
//! # Basic Usage
//! 
//! The most common way of using this crate is by driving an axis to a
//! particular location.
//! 
//! ```rust
//! use accel_stepper::{Driver, SystemClock};
//! # use core::time::Duration;
//! # use core::cell::RefCell;
//! 
//! let mut axis = Driver::new();
//! // Make sure to set your device's motion parameters
//! axis.set_max_speed(500.0);
//! axis.set_acceleration(100.0);
//! 
//! // The axis needs a clock for timing purposes. This could be an
//! // `accel_stepper::OperatingSystemClock` when compiled with the `std`
//! // feature, or your device's external oscillator
//! 
//! #[derive(Debug, Default)]
//! struct TickingClock(RefCell<Duration>);
//! 
//! impl SystemClock for TickingClock {
//!     fn elapsed(&self) -> Duration {
//!         let mut ticks = self.0.borrow_mut();
//!         *ticks = *ticks + Duration::from_millis(10);
//!         ticks.clone()
//!     }
//! }
//! 
//! let clock = TickingClock::default();
//! 
//! let mut forward = 0;
//! let mut back = 0;
//! 
//! {
//!     // for testing purposes, we'll create a Device which counts the number
//!     // of forward/backward steps
//!     let mut dev = accel_stepper::func_device(|| forward += 1, || back += 1);
//! 
//!     // set the desired location
//!     axis.move_to(17);
//! 
//!     // keep polling the axis until it reaches that location
//!     while axis.is_running() {
//!         axis.poll(&mut dev, &clock)?;
//!     }
//! }
//! 
//! // we should have arrived at our destination
//! assert_eq!(17, axis.current_position());
//! 
//! // it takes 17 steps forward to reach position 17
//! assert_eq!(17, forward);
//! assert_eq!(0, back);
//! # Result::<(), Box<dyn std::error::Error>>::Ok(())
//! ```
//!
//! # Cargo Features
//!
//! To minimise compile time and code size, this crate uses cargo features.
//!
//! - `std` - Enable functionality which depends on the standard library (e.g.
//!   the OS clock)
//! - `hal` - Enable functionality which implements [`Device`] on top of traits
//!   from the [`embedded-hal`][hal] crate.
//!
//! [original]: http://www.airspayce.com/mikem/arduino/AccelStepper/index.html
//! [hal]: https://crates.io/crates/embedded-hal

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), test))]
#[macro_use]
extern crate std;

mod clock;
mod device;
mod driver;
#[cfg(feature = "hal")]
mod hal_devices;
mod multi_driver;
mod utils;

pub use crate::{
    clock::SystemClock,
    device::{fallible_func_device, func_device, Device, StepContext},
    driver::Driver,
    multi_driver::MultiDriver,
    utils::CummulativeSteps,
};

#[cfg(feature = "std")]
pub use crate::clock::OperatingSystemClock;

#[cfg(feature = "hal")]
pub use crate::hal_devices::*;
