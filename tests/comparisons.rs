use AccelStepper_sys::AccelStepper;
use accel_stepper::{Driver, Device, SystemClock};
use quickcheck_macros::quickcheck;
use quickcheck::{Gen, Arbitrary, TestResult};
use rand::Rng;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[quickcheck]
fn both_versions_are_identical(input: Input) -> TestResult {
    let rust = Counter::default();
    let mut rust_driver = Driver::new(rust.clone());

    // make sure the counter is zeroed out at the start of the run
    ORIGINAL.forward.store(0, Ordering::SeqCst);
    ORIGINAL.back.store(0, Ordering::SeqCst);

    let mut original_driver = unsafe { AccelStepper::new1(Some(forward), Some(back)) };

    // initialize the Rust driver with our motion parameters
    rust_driver.set_speed(input.speed);
    rust_driver.set_acceleration(input.max_acceleration);
    rust_driver.move_to(input.target_location);

    // and initialize the AccelStepper driver
    unsafe {
        original_driver.setSpeed(input.speed);
        original_driver.setAcceleration(input.max_acceleration);
        original_driver.moveTo(input.target_location);
    }

    for i in 0..input.iterations {
        unsafe {
            // update the "time"
            MICROS = i * 100;

            rust_driver.poll(&MicrosClock).unwrap();
            original_driver.run(); 
        }

        assert_eq!(*rust.0, ORIGINAL, "{:?}                                  {:?}", rust_driver, original_driver);
    }

    TestResult::from_bool(true)
}

extern "C" {
    static mut MICROS: u64;
}

/// Inject our own `micros` function in at link-time so we can mock out the
/// Arduino's timing functionality.
#[no_mangle]
pub unsafe extern "C" fn micros() -> u64 {
    MICROS
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
static ORIGINAL: Inner = Inner { forward: AtomicU32::new(0), back: AtomicU32::new(0) };

unsafe extern "C" fn forward() {
    ORIGINAL.forward.fetch_add(1, Ordering::SeqCst);
}

unsafe extern "C" fn back() {
    ORIGINAL.back.fetch_add(1, Ordering::SeqCst);
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Input {
    speed: f32,
    max_acceleration: f32,
    target_location: i64,
    iterations: u64,
}

impl Arbitrary for Input {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Input {
            speed: g.gen_range(0.0, 10000.0),
            max_acceleration: g.gen_range(0.0, 5000.0),
            target_location: g.gen_range(-500, 500),
            iterations: g.gen_range(0, 1000),
        }
    }
}


#[derive(Debug, Default)]
struct Inner {
    forward: AtomicU32,
    back: AtomicU32,
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
