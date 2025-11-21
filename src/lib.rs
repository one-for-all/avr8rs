use std::f64;

pub mod clock;
pub mod cpu;
pub mod instruction;
pub mod interrupt;
pub mod plot;
pub mod port;
pub mod program;
pub mod runner;
pub mod stepper;
pub mod timer;
pub mod usart;
pub mod util;

pub type Float = f64;
pub const PI: Float = f64::consts::PI;
