//! Asynchronous driver for AS5048A magnetic position sensor
//!
//! This driver implements an async interface for the AS5048A 14-bit magnetic rotary
//! position sensor using the SPI interface. It is compatible with embedded-hal-async
//! and works with embassy and other async embedded frameworks
//!
//! # Features
//!
//! - Async SPI communication using `embedded-hal-async`
//! - Read 14-bit angular position
//! - Read magnitude and diagnostics
//! - Automatic parity checking
//! - Error flag handling
//!
//! # Example
//!
//! ```no_run
//! use as5048a_async::As5048a;
//! use embedded_hal_async::spi::SpiDevice;
//!
//! async fn read_angle<SPI, E>(spi: SPI) -> Result<(), as5048a_async::Error<E>>
//! where
//!     SPI: SpiDevice<u8, Error = E>,
//! {
//!     let mut sensor = As5048a::new(spi);
//!
//!     // Read angle in degrees (0-359)
//!     match sensor.angle_degrees().await {
//!         Ok(degrees) => {
//!             // degrees is 0-359
//!         }
//!         Err(e) => {
//!             // Handle error
//!         }
//!     }
//!
//!     // Or read raw 14-bit value (0-16383)
//!     let raw_angle = sensor.angle().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Important Timing Requirements
//!
//! The AS5048A requires a minimum CS (Chip Select) high time of **350ns** between
//! SPI transactions. Ensure your `SpiDevice` implementation provides adequate
//! inter-frame delay to meet this requirement. The maximum SPI clock frequency is 10 MHz
//!
//! # Error Handling
//!
//! When `Error::SensorError` occurs, the sensor's internal error flag is set
//! To recover:
//! 1. Call `clear_error_flag()` to reset the error state
//! 2. Retry the operation
//!
//! Example:
//! ```no_run
//! # use as5048a_async::{As5048a, Error};
//! # async fn example<SPI, E>(mut sensor: As5048a<SPI>) -> Result<u16, Error<E>>
//! # where SPI: embedded_hal_async::spi::SpiDevice<u8, Error = E>
//! # {
//! match sensor.angle().await {
//!     Err(Error::SensorError) => {
//!         sensor.clear_error_flag().await?;
//!         sensor.angle().await // Retry
//!     }
//!     result => result
//! }
//! # }
//! ```

#![no_std]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

mod diagnostics;
mod driver;
mod error;
mod register;
mod utils;

pub use diagnostics::Diagnostics;
pub use driver::{As5048a, ANGLE_MAX};
pub use error::Error;
pub use register::Register;
