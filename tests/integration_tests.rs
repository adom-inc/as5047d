//! Integration tests for AS5048A driver using mocked SPI.

use as5048a_async::{As5048a, Error};
use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

/// Helper to calculate even parity for a 16-bit value.
fn calculate_parity(value: u16) -> u16 {
    let bits = value & 0x7FFF;
    if bits.count_ones() % 2 == 1 {
        0x8000 | value
    } else {
        value
    }
}

/// Helper to create a read command frame with parity.
fn read_command(address: u16) -> [u8; 2] {
    let cmd = 0x4000 | address;
    calculate_parity(cmd).to_be_bytes()
}

/// Helper to create a response frame with parity.
fn response_frame(data: u16, error_flag: bool) -> [u8; 2] {
    let frame = if error_flag {
        0x4000 | (data & 0x3FFF)
    } else {
        data & 0x3FFF
    };
    calculate_parity(frame).to_be_bytes()
}

#[tokio::test]
async fn reads_angle_register() {
    let expectations = [
        // Transaction 1: Send read command for ANGLE register (0x3FFF)
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            read_command(0x3FFF).to_vec(),
            vec![0x00, 0x00], // Response is ignored in first transaction
        ),
        SpiTransaction::transaction_end(),
        // Transaction 2: Send NOP, receive angle data
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0x00, 0x00], // NOP command
            response_frame(0x1234, false).to_vec(), // Angle value 0x1234
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    let angle = sensor.angle().await.unwrap();
    assert_eq!(angle, 0x1234);

    sensor.release().done();
}

#[tokio::test]
async fn reads_magnitude_register() {
    let expectations = [
        // Transaction 1: Send read command for MAGNITUDE register (0x3FFE)
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            read_command(0x3FFE).to_vec(),
            vec![0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // Transaction 2: Send NOP, receive magnitude data
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0x00, 0x00],
            response_frame(0x0ABC, false).to_vec(),
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    let magnitude = sensor.magnitude().await.unwrap();
    assert_eq!(magnitude, 0x0ABC);

    sensor.release().done();
}

#[tokio::test]
async fn reads_diagnostics_register() {
    let expectations = [
        // Transaction 1: Send read command for DIAGNOSTICS register (0x3FFD)
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            read_command(0x3FFD).to_vec(),
            vec![0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // Transaction 2: Send NOP, receive diagnostics data
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0x00, 0x00],
            response_frame(0x0480, false).to_vec(), // OCF bit set (bit 10), AGC = 128
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    let diag = sensor.diagnostics().await.unwrap();
    assert_eq!(diag.raw(), 0x0480);
    assert!(diag.offset_comp_finished());
    assert_eq!(diag.agc_value(), 128);
    assert!(diag.is_valid());

    sensor.release().done();
}

#[tokio::test]
async fn detects_parity_error() {
    // Create a response with incorrect parity (odd number of bits)
    let bad_response = [0xC0, 0x01]; // This has odd parity

    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            read_command(0x3FFF).to_vec(),
            vec![0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0x00, 0x00],
            bad_response.to_vec(),
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    let result = sensor.angle().await;
    assert!(matches!(result, Err(Error::ParityError)));

    sensor.release().done();
}

#[tokio::test]
async fn detects_sensor_error_flag() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            read_command(0x3FFF).to_vec(),
            vec![0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0x00, 0x00],
            response_frame(0x1234, true).to_vec(), // Error flag set
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    let result = sensor.angle().await;
    assert!(matches!(result, Err(Error::SensorError)));

    sensor.release().done();
}

#[tokio::test]
async fn clears_error_flag() {
    let expectations = [
        // Transaction 1: Send read command for CLEAR_ERROR_FLAG register (0x0001)
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            read_command(0x0001).to_vec(),
            vec![0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // Transaction 2: Send NOP, receive error register content
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0x00, 0x00],
            response_frame(0x0002, false).to_vec(), // Error bits
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    sensor.clear_error_flag().await.unwrap();

    sensor.release().done();
}

#[tokio::test]
async fn reads_multiple_angles_sequentially() {
    let angles = [0x0000, 0x1000, 0x2000, 0x3000];
    let mut expectations = Vec::new();

    for &angle_value in &angles {
        expectations.extend_from_slice(&[
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(
                read_command(0x3FFF).to_vec(),
                vec![0x00, 0x00],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(
                vec![0x00, 0x00],
                response_frame(angle_value, false).to_vec(),
            ),
            SpiTransaction::transaction_end(),
        ]);
    }

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    for &expected_angle in &angles {
        let angle = sensor.angle().await.unwrap();
        assert_eq!(angle, expected_angle);
    }

    sensor.release().done();
}

#[tokio::test]
async fn masks_data_to_14_bits() {
    // Send data with upper bits set (should be masked out)
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            read_command(0x3FFF).to_vec(),
            vec![0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0x00, 0x00],
            response_frame(0x3FFF, false).to_vec(), // Maximum 14-bit value
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let mut sensor = As5048a::new(spi);

    let angle = sensor.angle().await.unwrap();
    assert_eq!(angle, 0x3FFF); // Should be exactly 14 bits (16383)
    assert!(angle <= 0x3FFF); // Verify it's within 14-bit range

    sensor.release().done();
}
