use core::time::Duration;

pub(crate) trait Clamp {
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
pub(crate) trait DurationHelpers {
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
        (self.as_secs() as f32)
            + (self.subsec_nanos() as f32) / (NANOS_PER_SEC as f32)
    }
}
