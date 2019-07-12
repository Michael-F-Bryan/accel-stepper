#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), test))]
#[macro_use]
extern crate std;

mod clock;
mod driver;
mod device;

pub use crate::driver::Driver;
pub use crate::clock::SystemClock;
pub use crate::device::{Device, FunctionalDevice};

#[cfg(feature = "std")]
pub use crate::clock::OperatingSystemClock;
