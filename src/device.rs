/// An interface to the stepper motor.
pub trait Device {
    /// The type of error that may be encountered when taking a step.
    ///
    /// Use `!` (or `void::Void` on stable) if stepping can never fail.
    type Error;

    fn step(&mut self, position: i64) -> Result<(), Self::Error>;
}

/// A [`Device`] which will call one function to make a forwards step, and
/// another to step backwards.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FunctionalDevice<Forwards, Backwards> {
    forwards: Forwards,
    backwards: Backwards,
}

impl<Forwards, Backwards> FunctionalDevice<Forwards, Backwards> {
    pub fn new(forwards: Forwards, backwards: Backwards) -> Self {
        FunctionalDevice {
            forwards,
            backwards,
        }
    }
}

impl<Forwards, Backwards, E> Device for FunctionalDevice<Forwards, Backwards>
where
    Forwards: FnMut() -> Result<(), E>,
    Backwards: FnMut() -> Result<(), E>,
{
    type Error = E;

    #[inline]
    fn step(&mut self, position: i64) -> Result<(), Self::Error> {
        if position >= 0 {
            (self.forwards)()
        } else {
            (self.backwards)()
        }
    }
}
