#![allow(non_snake_case)]
use std::f64;

pub mod atmega328p;
pub mod clock;
pub mod cpu;
pub mod encoder;
pub mod instruction;
pub mod interrupt;
pub mod peripheral;
pub mod program;
pub mod runner;
pub mod stepper;
pub mod util;

#[cfg(not(target_arch = "wasm32"))]
pub mod plot;

pub type Float = f64;
pub const PI: Float = f64::consts::PI;
