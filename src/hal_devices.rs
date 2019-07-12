use crate::Device;
use embedded_hal::digital::v2::OutputPin;

/// A [`Device`] which has step and direction pins.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct StepAndDirection<Step, Direction> {
    step: Step, 
    direction: Direction,
}

impl<Step, Direction> StepAndDirection<Step, Direction> {
    pub fn new(step: Step, direction: Direction) -> Self {
        StepAndDirection {step, direction }
    }

    pub fn into_inner(self) -> (Step, Direction) {
        (self.step, self.direction)
    }
}

fn set_output<P: OutputPin>(pin: &mut P, mask: u8) -> Result<(), P::Error> {
    if mask != 0 {
        pin.set_high()
    } else {
        pin.set_low()
    }
}

impl<Step, Direction, E> StepAndDirection<Step, Direction>
where
    Step: OutputPin<Error = E>,
    Direction: OutputPin<Error = E>,
{
    fn set_output(&mut self, mask: u8) -> Result<(), E> {
        set_output(&mut self.step, mask & 0b01)?;
        set_output(&mut self.direction, mask & 0b10)?;

        Ok(())
    }
}

impl<Step, Direction, E> Device for StepAndDirection<Step, Direction>
where
    Step: OutputPin<Error = E>,
    Direction: OutputPin<Error = E>,
{
    type Error = E;

    #[inline]
    fn step(&mut self, position: i64) -> Result<(), Self::Error> {
        // copied straight from AccelStepper::step2()
        match position & 0x03 {
            0 => self.set_output(0b10),
            1 => self.set_output(0b11),
            2 => self.set_output(0b01),
            3 => self.set_output(0b00),
            _ => unreachable!(),
        }
    }
}
