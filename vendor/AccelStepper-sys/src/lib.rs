//! A quick'n'dirty interface to `AccelStepper` so we can use it when checking
//! our Rust implementation.

#![allow(
    non_snake_case,
    deprecated,
    non_upper_case_globals,
    non_camel_case_types,
    intra_doc_link_resolution_failure
)]

// bindgen --output src/bindings.rs
//         --whitelist-type AccelStepper
//         ../AccelStepper/AccelStepper.cpp
//         --
//         -DARDUINO=100 -I src
include!("bindings.rs");
