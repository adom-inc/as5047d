# as5048a-async

[![Crates.io](https://img.shields.io/crates/v/as5048a-async.svg)](https://crates.io/crates/as5048a-async)
[![Documentation](https://img.shields.io/docsrs/as5048a-async)](https://docs.rs/as5048a-async)
[![License](https://img.shields.io/crates/l/as5048a-async)]


Async Rust driver for the **AS5048A** 14-bit magnetic rotary position sensor using the SPI interface.

This driver is designed for embedded systems using [`embedded-hal-async`](https://github.com/rust-embedded/embedded-hal) and works seamlessly with async runtimes like [Embassy](https://embassy.dev/).

- Fully async using `embedded-hal-async` traits
- 14-bit angular position reading (0.022° resolution)
- Optional `defmt` logging for debugging
- `no_std` compatible
- `forbid(unsafe_code)` - 100% safe Rust

## AS5048A Requirements

The AS5048A is a 14-bit rotary position sensor with SPI interface:
- **Resolution:** 14-bit (16384 positions, 0.022°/LSB)
- **SPI Mode:** Mode 1 (CPOL=0, CPHA=1)
- **Max Clock:** 10 MHz (recommend 1-3 MHz for initial testing)
- **Supply:** 3.3V or 5V

For more details read the full [AS5048A Datasheet](https://look.ams-osram.com/m/287d7ad97d1ca22e/original/AS5048-DS000298.pdf).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
as5048a-async = "0.1"
embedded-hal-async = "1.0"
```

## Usage

### Quick Start

```rust
use as5048a_async::As5048a;

// Create sensor with an SpiDevice (bus + CS pin)
let mut sensor = As5048a::new(spi_device);

// Read angle (0-16383, representing 0-360°)
let angle = sensor.angle().await?;
let degrees = (angle as f32) * 360.0 / 16384.0;
```

For complete examples see the [examples](examples/) folder.

### Diagnostics and Error Handling

```rust
// Check magnetic field strength and sensor status
let diag = sensor.diagnostics().await?;
if diag.is_valid() {
    // Read AGC value, check COMP flags, etc.
}

// Clear error flags
sensor.clear_error_flag().await?;
```

## API Reference

See the [API documentation](https://docs.rs/as5048a-async) for complete details on available methods and types.

Main methods:
- `angle()` - Read 14-bit angle (0-16383)
- `angle_degrees()` - Read angle in degrees (0-359) using integer math
- `magnitude()` - Read CORDIC magnitude
- `diagnostics()` - Read diagnostics with AGC value and status flags
- `clear_error_flag()` - Clear error flag

Constants:
- `ANGLE_MAX` - Maximum angle value (16384) for custom conversions

## MSRV (Minimum Supported Rust Version)

This crate requires Rust 1.87 or later.

## Resources

- [API Documentation](https://docs.rs/as5048a-async)
- [AS5048A Datasheet](https://look.ams-osram.com/m/287d7ad97d1ca22e/original/AS5048-DS000298.pdf)

## Inspired by

 - https://github.com/uwearzt/as5048a
 - https://github.com/barafael/as5600-rs

## Contributing

Contributions are welcome! Please run `cargo test` and `cargo clippy` before submitting.
