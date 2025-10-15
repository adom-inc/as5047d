# AS5048A Example

This folder contains an example for using the `as5048a-async` driver with Embassy on STM32. View an example SPI implementation at: https://github.com/embassy-rs/embassy/blob/main/examples/stm32h7/src/bin/spi_dma.rs. This example is for reference only and is not compiled by default.

## STM32 Example (`stm32_basic.rs`)
Basic example for STM32 microcontrollers using Embassy.
- Target: STM32 (tested on STM32F4/L4)
- SPI: SPI1 with shared bus support
- Shows: Basic angle reading with diagnostic error handling

## Important Notes

### SPI Mode Configuration
The example uses **SPI Mode 1** (CPOL=0, CPHA=1), which is required by the AS5048A. Using Mode 0 will cause errors.

### Shared Bus Pattern
The example uses the HALs `SpiDevice` but by default you get the entire bus in embassy so using the `embassy-embedded-hal` is required to correctly handle the CS pin.
