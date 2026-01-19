#![no_std]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod driver;
mod error;
mod register;
mod utils;

pub use driver::{ANGLE_MAX, As5047d};
pub use error::Error;
pub use register::Register;
