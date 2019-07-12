use void::Void;

/// An interface to the stepper motor.
pub trait Device {
    /// The type of error that may be encountered when taking a step.
    ///
    /// Use `!` (or `void::Void` on stable) if stepping can never fail.
    type Error;

    fn step(&mut self, position: i64) -> Result<(), Self::Error>;
}

/// A [`Device`] which will call one function for a forward step, and another
/// for a backward one.
///
/// See [`fallible_func_device()`] for a version which accepts fallible
/// callbacks.
pub fn func_device<F, B, T>(forward: F, backward: B) -> impl Device<Error = Void>
where
    F: FnMut() -> T,
    B: FnMut() -> T,
{
    Infallible {
        forward,
        backward,
        previous_position: 0,
    }
}

struct Infallible<F, B> {
    previous_position: i64,
    forward: F,
    backward: B,
}

impl<F, B, T> Device for Infallible<F, B>
where
    F: FnMut() -> T,
    B: FnMut() -> T,
{
    type Error = Void;

    #[inline]
    fn step(&mut self, position: i64) -> Result<(), Self::Error> {
        let diff = position - self.previous_position;

        if diff > 0 {
            (self.forward)();
        } else if diff < 0 {
            (self.backward)();
        }

        self.previous_position = position;
        Ok(())
    }
}

/// A device which uses callbacks which may fail.
///
/// See [`func_device()`] for a version which uses infallible callbacks.
pub fn fallible_func_device<F, B, T, E>(forward: F, backward: B) -> impl Device<Error = E>
where
    F: FnMut() -> Result<T, E>,
    B: FnMut() -> Result<T, E>,
{
    Fallible {
        forward,
        backward,
        previous_position: 0,
    }
}

struct Fallible<F, B> {
    previous_position: i64,
    forward: F,
    backward: B,
}

impl<F, B, T, E> Device for Fallible<F, B>
where
    F: FnMut() -> Result<T, E>,
    B: FnMut() -> Result<T, E>,
{
    type Error = E;

    #[inline]
    fn step(&mut self, position: i64) -> Result<(), Self::Error> {
        let diff = position - self.previous_position;

        if diff > 0 {
            (self.forward)()?;
        } else if diff < 0 {
            (self.backward)()?;
        }

        self.previous_position = position;
        Ok(())
    }
}
