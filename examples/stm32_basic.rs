//! Basic example for STM32 with Embassy
//!
//! This example demonstrates how to use the AS5048A sensor with
//! Embassy on an STM32 microcontroller.
//!
//! Hardware setup:
//! - AS5048A connected via SPI1
//! - SPI pins: SCK=PA5, MOSI=PA7, MISO=PA6, CS=PA4
//! - SPI Mode 1 (CPOL=0, CPHA=1)
//! - 3 MHz clock frequency

#![no_std]
#![no_main]

use as5048a_async::As5048a;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    spi,
    time::Hertz,
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Embassy shared bus support
use embassy_embedded_hal::shared_bus::spi::SpiDevice;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

// Static storage for shared SPI bus
static SPI_BUS: StaticCell<Mutex<NoopRawMutex, spi::Spi<'static, spi::SPI1>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut spi_config = spi::Config::default();
    spi_config.mode = spi::MODE_1;
    spi_config.frequency = Hertz(3_000_000);

    let spi = spi::Spi::new(
        p.SPI1,
        p.PA5,
        p.PA7,
        p.PA6,
        p.DMA1_CH1,
        p.DMA1_CH2,
        spi_config,
    );

    let spi_bus = Mutex::new(spi);
    let spi_bus = SPI_BUS.init(spi_bus);

    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);
    let spi_device = SpiDevice::new(spi_bus, cs);

    let mut sensor = As5048a::new(spi_device);

    info!("AS5048A driver initialized");

    loop {
        match sensor.angle_degrees().await {
            Ok(degrees) => {
                info!("Angle: {}°", degrees);
            }
            Err(e) => {
                error!("Sensor error: {:?}", e);

                if let Ok(diag) = sensor.diagnostics().await {
                    if diag.comp_high() {
                        warn!("Magnet too close");
                    } else if diag.comp_low() {
                        warn!("Magnet too far");
                    } else if diag.cordic_overflow() {
                        warn!("CORDIC overflow");
                    }
                    info!("AGC value: {}", diag.agc_value());
                }
            }
        }

        Timer::after_millis(100).await;
    }
}
