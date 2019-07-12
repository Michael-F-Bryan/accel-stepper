use core::time::Duration;

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
