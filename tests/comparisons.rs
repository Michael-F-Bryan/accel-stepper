use AccelStepper_sys::AccelStepper;
use accel_stepper::{Driver, Device, SystemClock};
use quickcheck_macros::quickcheck;
use quickcheck::{Gen, Arbitrary, TestResult};
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[quickcheck]
#[ignore]
fn both_versions_are_identical(input: Input) -> TestResult {
    let rust = Counter::default();
    let mut rust_driver = Driver::new(rust.clone());

    // make sure the counter is zeroed out at the start of the run
    ORIGINAL.forward.store(0, Ordering::SeqCst);
    ORIGINAL.back.store(0, Ordering::SeqCst);

    let mut original_driver = unsafe { AccelStepper::new1(Some(forward), Some(back)) };

    // initialize the Rust driver with our motion parameters
    rust_driver.set_speed(input.speed);
    rust_driver.set_max_speed(input.max_speed);
    rust_driver.set_acceleration(input.max_acceleration);
    rust_driver.move_to(input.target_location);

    // and initialize the AccelStepper driver
    unsafe {
        original_driver.setSpeed(input.speed);
        original_driver.setMaxSpeed(input.max_speed);
        original_driver.setAcceleration(input.max_acceleration);
        original_driver.moveTo(input.target_location);
    }

    assert_eq!(rust_driver.target_position(), unsafe { original_driver.targetPosition() });
    assert_eq!(rust_driver.acceleration(), original_driver._acceleration);

    for i in 0..input.iterations {
        unsafe {
            // update the "time"
            MICROS = i * 1000;

            rust_driver.poll(&MicrosClock).unwrap();
            original_driver.run(); 
        }
    }

    assert_eq!(rust_driver.speed().round(), unsafe {original_driver.speed().round() }, "{:#?}\n\n{:#?}\n", rust_driver, original_driver);
    assert_eq!(*rust.0, ORIGINAL, "{:#?}\n\n{:#?}\n", rust_driver, original_driver);

    TestResult::from_bool(true)
}

extern "C" {
    /// This is defined in the `AccelStepper-sys` crate's `Arduino.h` header
    /// file.
    static mut MICROS: u64;
}

struct MicrosClock;

impl SystemClock for MicrosClock {
    fn elapsed(&self) -> Duration {
        unsafe {
            Duration::from_micros(MICROS)
        }
    }
}

/// Note: quickcheck doesn't use multi-threading, so this static will only
/// be used by one thing at a time
static ORIGINAL: Inner = Inner { forward: AtomicUsize::new(0), back: AtomicUsize::new(0) };

unsafe extern "C" fn forward() {
    ORIGINAL.forward.fetch_add(1, Ordering::SeqCst);
}

unsafe extern "C" fn back() {
    ORIGINAL.back.fetch_add(1, Ordering::SeqCst);
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Input {
    speed: f32,
    max_speed: f32,
    max_acceleration: f32,
    target_location: i64,
    iterations: u64,
}

impl Arbitrary for Input {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let speed = g.gen_range(0.0, 10000.0);
        Input {
            speed,
            max_speed: g.gen_range(speed, 15000.0),
            max_acceleration: g.gen_range(0.0, 5000.0),
            target_location: g.gen_range(-500, 500),
            iterations: g.gen_range(0, 1000),
        }
    }
}


#[derive(Debug, Default)]
struct Inner {
    forward: AtomicUsize,
    back: AtomicUsize,
}

impl PartialEq for Inner {
    fn eq(&self, other: &Inner) -> bool {
        self.forward.load(Ordering::SeqCst) == other.forward.load(Ordering::SeqCst) && 
        self.back.load(Ordering::SeqCst) == other.back.load(Ordering::SeqCst)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Counter(Arc<Inner>);

impl Device for Counter {
    type Error = void::Void;

    fn step(&mut self, position: i64) -> Result<(), Self::Error> {
        if position > 0 {
            self.0.forward.fetch_add(1, Ordering::SeqCst);
        } else if position < 0  {
            self.0.back.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }
}
