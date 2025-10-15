//! Asynchronous driver for AS5048A magnetic position sensor

use embedded_hal_async::spi::SpiDevice;

use crate::{diagnostics::Diagnostics, error::Error, register::Register, utils};

const READ_BIT: u16 = 0x4000;
const PARITY_BIT: u16 = 0x8000;
const ERROR_FLAG: u16 = 0x4000;
const DATA_MASK: u16 = 0x3FFF;
const NOP_COMMAND: u16 = 0x0000;

/// Maximum angle value (14-bit: 0-16383, representing 0-360°)
pub const ANGLE_MAX: u16 = 0x3FFF;

/// AS5048A driver instance (asynchronous)
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct As5048a<SPI> {
    spi: SPI,
}

impl<SPI, E> As5048a<SPI>
where
    SPI: SpiDevice<u8, Error = E>,
{
    /// Create a new AS5048A driver instance
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Release the SPI bus, consuming the driver
    pub fn release(self) -> SPI {
        self.spi
    }

    /// Read a register from the AS5048A
    ///
    /// This follows the command-response protocol:
    /// - Transaction 1: Send read command, ignore response
    /// - Transaction 2: Send NOP, receive actual data
    async fn read_register(&mut self, register: Register) -> Result<u16, Error<E>> {
        let address = u16::from(register);

        let command = READ_BIT | address;

        let command = if utils::calculate_parity(command) {
            PARITY_BIT | command
        } else {
            command
        };

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "Reading register 0x{:04X}, command: 0x{:04X}",
            address,
            command
        );

        let tx_cmd = command.to_be_bytes();
        let mut rx_cmd = [0u8; 2];
        self.spi
            .transfer(&mut rx_cmd, &tx_cmd)
            .await
            .map_err(Error::Communication)?;

        let tx_nop = NOP_COMMAND.to_be_bytes();
        let mut rx_data = [0u8; 2];
        self.spi
            .transfer(&mut rx_data, &tx_nop)
            .await
            .map_err(Error::Communication)?;

        let response = u16::from_be_bytes(rx_data);

        #[cfg(feature = "defmt")]
        defmt::trace!("Received response: 0x{:04X}", response);

        if !utils::verify_parity(response) {
            #[cfg(feature = "defmt")]
            defmt::warn!("Parity error in response: 0x{:04X}", response);
            return Err(Error::ParityError);
        }

        if response & ERROR_FLAG != 0 {
            #[cfg(feature = "defmt")]
            defmt::warn!("Sensor error flag set in response");
            return Err(Error::SensorError);
        }

        let data = response & DATA_MASK;
        #[cfg(feature = "defmt")]
        defmt::debug!("Register 0x{:04X} value: 0x{:04X}", address, data);

        Ok(data)
    }

    /// Write a register to the AS5048A
    ///
    /// This follows the write protocol:
    /// - Transaction 1: Send write command
    /// - Transaction 2: Send data frame
    /// - Transaction 3: Send NOP to verify write
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - SPI communication fails
    /// - Parity check fails on the response
    /// - The sensor reports an error
    #[allow(dead_code)]
    async fn write_register(&mut self, register: Register, data: u16) -> Result<(), Error<E>> {
        let address = u16::from(register);

        #[cfg(feature = "defmt")]
        defmt::debug!("Writing 0x{:04X} to register 0x{:04X}", data, address);

        let command = address;

        let command = if utils::calculate_parity(command) {
            PARITY_BIT | command
        } else {
            command
        };

        let tx_cmd = command.to_be_bytes();
        let mut rx_cmd = [0u8; 2];
        self.spi
            .transfer(&mut rx_cmd, &tx_cmd)
            .await
            .map_err(Error::Communication)?;

        let data_frame = data & DATA_MASK;
        let data_frame = if utils::calculate_parity(data_frame) {
            PARITY_BIT | data_frame
        } else {
            data_frame
        };

        let tx_data = data_frame.to_be_bytes();
        let mut rx_old = [0u8; 2];
        self.spi
            .transfer(&mut rx_old, &tx_data)
            .await
            .map_err(Error::Communication)?;

        let tx_nop = NOP_COMMAND.to_be_bytes();
        let mut rx_verify = [0u8; 2];
        self.spi
            .transfer(&mut rx_verify, &tx_nop)
            .await
            .map_err(Error::Communication)?;

        let response = u16::from_be_bytes(rx_verify);

        if !utils::verify_parity(response) {
            #[cfg(feature = "defmt")]
            defmt::warn!("Parity error in write verification: 0x{:04X}", response);
            return Err(Error::ParityError);
        }

        if response & ERROR_FLAG != 0 {
            #[cfg(feature = "defmt")]
            defmt::warn!("Sensor error flag set during write");
            return Err(Error::SensorError);
        }

        #[cfg(feature = "defmt")]
        defmt::trace!("Write to register 0x{:04X} successful", address);

        Ok(())
    }

    /// Get the 14-bit corrected angular position
    ///
    /// Value ranges from 0 to 16383 (0° to 359.978°)
    /// Use [`ANGLE_MAX`] constant for conversion calculations
    ///
    /// For integer degree conversion, use [`Self::angle_degrees`]
    ///
    /// # Errors
    ///
    /// Returns an error if SPI communication fails, parity check fails, or the sensor reports an error
    pub async fn angle(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::Angle).await
    }

    /// Get the angular position in degrees (0-359)
    ///
    /// This method converts the raw 14-bit angle value to degrees using
    /// integer arithmetic with saturation. The result is rounded down
    ///
    /// # Errors
    ///
    /// Returns an error if SPI communication fails, parity check fails, or the sensor reports an error
    pub async fn angle_degrees(&mut self) -> Result<u16, Error<E>> {
        let angle = self.angle().await?;
        let degrees = (u32::from(angle).saturating_mul(360)) / u32::from(ANGLE_MAX);
        #[allow(clippy::cast_possible_truncation)]
        Ok(degrees as u16)
    }

    /// Get the 14-bit magnitude value from CORDIC
    ///
    /// Useful for checking magnet presence and strength
    ///
    /// # Errors
    ///
    /// Returns an error if SPI communication fails, parity check fails, or the sensor reports an error
    pub async fn magnitude(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::Magnitude).await
    }

    /// Get the diagnostics and AGC register
    ///
    /// Returns a `Diagnostics` struct with helper methods to check:
    /// - Magnetic field strength (`COMP_HIGH`, `COMP_LOW`)
    /// - CORDIC overflow
    /// - Offset compensation status
    /// - AGC value
    ///
    /// # Errors
    ///
    /// Returns an error if SPI communication fails, parity check fails, or the sensor reports an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use as5048a_async::As5048a;
    /// # async fn example<SPI, E>(mut sensor: As5048a<SPI>) -> Result<(), as5048a_async::Error<E>>
    /// # where SPI: embedded_hal_async::spi::SpiDevice<u8, Error = E>
    /// # {
    /// let diag = sensor.diagnostics().await?;
    ///
    /// if diag.is_valid() {
    ///     println!("AGC value: {}", diag.agc_value());
    /// } else if diag.cordic_overflow() {
    ///     println!("CORDIC overflow - data invalid!");
    /// } else if diag.comp_high() {
    ///     println!("Magnet too close");
    /// } else if diag.comp_low() {
    ///     println!("Magnet too far");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn diagnostics(&mut self) -> Result<Diagnostics, Error<E>> {
        self.read_register(Register::DiagAgc)
            .await
            .map(Diagnostics::new)
    }

    /// Clear the error flag by reading the clear error flag register
    ///
    /// # Errors
    ///
    /// Returns an error if SPI communication fails, parity check fails, or the sensor reports an error
    pub async fn clear_error_flag(&mut self) -> Result<(), Error<E>> {
        self.read_register(Register::ClearErrorFlag).await?;
        Ok(())
    }
}
