//! A Rust port of the popular [`AccelStepper`][original] Arduino stepper
//! library.
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
