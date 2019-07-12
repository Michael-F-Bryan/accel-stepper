/// An interface to the stepper motor.
pub trait Device {
    fn step(&mut self, position: i64);
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

impl<Forwards, Backwards> Device for FunctionalDevice<Forwards, Backwards>
where
    Forwards: FnMut(),
    Backwards: FnMut(),
{
    fn step(&mut self, position: i64) {
        if position >= 0 {
            (self.forwards)();
        } else {
            (self.backwards)();
        }
    }
}
