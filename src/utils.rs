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

/// A helper type for determining the number of steps to take when moving a
/// distance in the real world, taking rounding errors into consideration.
#[derive(Debug, Clone, PartialEq)]
pub struct CummulativeSteps {
    steps_per_unit: f32,
    steps: f32,
}

impl CummulativeSteps {
    /// Create a new [`CummulativeSteps`] which will use the provided ratio as
    /// the number of steps to take per "real" unit.
    pub const fn new(steps_per_unit: f32) -> CummulativeSteps {
        CummulativeSteps {
            steps: 0.0,
            steps_per_unit,
        }
    }

    /// The current location in "real" units.
    pub const fn real_location(&self) -> f32 { self.steps }

    /// Get the number of steps travelled per "real" unit.
    pub const fn steps_per_unit(&self) -> f32 { self.steps_per_unit }

    pub const fn with_steps_per_unit(
        &self,
        steps_per_unit: f32,
    ) -> CummulativeSteps {
        CummulativeSteps {
            steps_per_unit,
            steps: self.steps,
        }
    }

    /// Execute a relative movement, retrieving the number of steps to take.
    pub fn move_by(&mut self, delta: f32) -> i64 {
        let previous_steps = self.steps.round();

        self.steps += delta * self.steps_per_unit;
        let rounded_steps = (self.steps - previous_steps).round();

        rounded_steps as i64
    }
}
