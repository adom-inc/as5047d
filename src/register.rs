//! Register addresses for AS5048A sensor.

/// Register addresses for AS5048A
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
#[repr(u16)]
pub enum Register {
    /// NOP command (no operation)
    Nop = 0x0000,
    /// Clear error flag.
    ClearErrorFlag = 0x0001,
    /// Diagnostics and AGC register
    DiagAgc = 0x3FFD,
    /// Magnitude register (14-bit)
    Magnitude = 0x3FFE,
    /// Angle register (14-bit corrected position)
    Angle = 0x3FFF,
}

impl From<Register> for u16 {
    fn from(reg: Register) -> u16 {
        reg as u16
    }
}
