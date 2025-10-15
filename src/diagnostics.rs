//! Diagnostics registers for AS5048A

/// Diagnostics flags from the `DIAG_AGC` register (0x3FFD)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Diagnostics {
    raw: u16,
}

impl Diagnostics {
    /// Create diagnostics from raw register value
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self { raw }
    }

    /// Get the raw register value
    #[must_use]
    pub const fn raw(&self) -> u16 {
        self.raw
    }

    /// `COMP_HIGH`: Magnetic field too strong
    ///
    /// Set when the magnetic field is above the recommended range
    /// The sensor may still work but accuracy could be affected
    #[must_use]
    pub const fn comp_high(&self) -> bool {
        self.raw & 0x2000 != 0
    }

    /// `COMP_LOW`: Magnetic field too weak
    ///
    /// Set when the magnetic field is below the recommended range
    /// The sensor may still work but accuracy could be affected
    #[must_use]
    pub const fn comp_low(&self) -> bool {
        self.raw & 0x1000 != 0
    }

    /// COF: CORDIC overflow
    ///
    /// Set when an overflow occurred in the CORDIC calculation
    /// When this bit is set, angle and magnitude data is invalid
    #[must_use]
    pub const fn cordic_overflow(&self) -> bool {
        self.raw & 0x0800 != 0
    }

    /// OCF: Offset compensation finished
    ///
    /// This flag is set to 1 after power-up when the offset compensation
    /// algorithm has finished. After power-up, the flag remains at 1
    #[must_use]
    pub const fn offset_comp_finished(&self) -> bool {
        self.raw & 0x0400 != 0
    }

    /// Get the Automatic Gain Control (AGC) value
    ///
    /// Returns an 8-bit value where:
    /// - 0 = high magnetic field (close to sensor)
    /// - 255 = low magnetic field (far from sensor)
    ///
    /// Typical values are between 60-200. Values outside this range
    /// may indicate the magnet is too close or too far
    #[must_use]
    pub const fn agc_value(&self) -> u8 {
        (self.raw & 0x00FF) as u8
    }

    /// Check if the magnetic field strength is within acceptable range
    ///
    /// Returns `true` if neither `COMP_HIGH` nor `COMP_LOW` is set
    #[must_use]
    pub const fn magnetic_field_ok(&self) -> bool {
        !self.comp_high() && !self.comp_low()
    }

    /// Check if data is valid
    ///
    /// Returns `true` if there's no CORDIC overflow and the magnetic
    /// field is within acceptable range
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        !self.cordic_overflow() && self.magnetic_field_ok()
    }
}

impl From<u16> for Diagnostics {
    fn from(raw: u16) -> Self {
        Self::new(raw)
    }
}
