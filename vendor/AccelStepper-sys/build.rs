use cc::Build;
use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor = manifest_dir.parent().unwrap();
    let accel_stepper = vendor.join("AccelStepper");

    Build::new()
        .file(accel_stepper.join("AccelStepper.cpp"))
        .include(&accel_stepper)
        .include(manifest_dir.join("src"))
        .define("ARDUINO", "100")
        .warnings(false)
        .cpp(true)
        .compile("AccelStepper");
}
