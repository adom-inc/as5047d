//! Register addresses for AS5047D sensor.

/// Register addresses for AS5047D
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
#[repr(u16)]
pub enum Register {
    /// No operation
    Nop = 0x0000,
    /// Clear error flag.
    ErrFl = 0x0001,
    /// Programming register
    Prog = 0x0003,

    /// Zero position MSB
    ZPosM = 0x0016,
    /// Zero position LSB/MAG diagnostic
    ZPosL = 0x0017,
    /// Custom setting register 1
    Settings1 = 0x0018,
    /// Custom setting register 2
    Settings2 = 0x0019,

    /// Diagnostic and AGC
    DiaAgc = 0x3FFC,
    /// CORDIC magnitude (14-bit)
    Mag = 0x3FFD,
    /// Measured angle without dynamic angle error compensation (14-bit)
    AngleUnc = 0x3FFE,
    /// Measured angle with dynamic angle error compensation (14-bit)
    AngleCom = 0x3FFF,
}

impl From<Register> for u16 {
    fn from(reg: Register) -> u16 {
        reg as u16
    }
}

bitfield::bitfield! {
    /// ERRFL
    ///
    /// Reading the ERRFL register automatically clears its contents
    // (ERRFL=0x0000)
    pub struct ErrorFlagRegister(u16);
    impl Debug;
    u8;
    /// Parity error
    pub parerr, _: 2;
    /// Invalid command error: set to 1 by reading or writing an invalid
    /// register address
    pub invcomm, _: 1;
    /// Framing error: is set to 1 when a non-compliant SPI frame is detected
    pub frerr, _: 0;
}

bitfield::bitfield! {
    /// PROG
    ///
    /// The PROG register is used for programming the OTP memory
    pub struct ProgrammingRegister(u16);
    impl Debug;
    u8;
    /// Program verify: must be set to 1 for verifying the correctness of the
    /// OTP programming
    pub progver, set_progver: 6;
    /// Start OTP programming cycle
    pub progotp, set_progotp: 3;
    /// Refreshes the non-volatile memory content with the OTP programmed
    /// content
    pub otpref, set_otpref: 2;
    /// Program OTP enable: enables programming the entire OTP memory
    pub progen, set_progen: 0;
}

bitfield::bitfield! {
    /// DIAAGC
    pub struct DiagnosticsAgcRegister(u16);
    impl Debug;
    u8;
    /// Magnetic field strength too low; AGC=0xFF
    pub magl, _: 11;
    /// Magnetic field strength too high; AGC=0x00
    pub magh, _: 10;
    /// CORDIC overflow
    pub cof, _: 9;
    /// Offset compensation
    ///
    /// - `0` = internal offset loops not ready regulated
    /// - `1` = internal offset loop finished
    pub lf, _: 8;
    /// Automatic gain control value
    pub agc, _: 7, 0;
}

impl DiagnosticsAgcRegister {
    /// Check if the magnetic field strength is within acceptable range
    #[must_use]
    #[inline(always)]
    pub fn magnetic_field_ok(&self) -> bool {
        !self.magh() && !self.magl()
    }

    /// Check if data is valid
    #[must_use]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        !self.cof() && self.magnetic_field_ok()
    }
}

bitfield::bitfield! {
    /// MAG
    pub struct CordicMagnitudeRegister(u16);
    impl Debug;
    /// CORDIC magnitude information
    pub cmag, _: 13, 0;
}

bitfield::bitfield! {
    /// ANGLEUNC
    pub struct AngleUncompensatedRegister(u16);
    impl Debug;
    /// Angle information without dynamic angle error compensation
    pub cordicang, _: 13, 0;
}

bitfield::bitfield! {
    /// ANGLECOM
    pub struct AngleCompensatedRegister(u16);
    impl Debug;
    /// Angle information with dynamic angle error compensation
    pub daecang, _: 13, 0;
}

bitfield::bitfield! {
    /// ZPOSM
    pub struct ZeroPositionMsbRegister(u16);
    impl Debug;
    u8;
    /// 8 most significant bits of the zero position
    pub zposm, set_zposm: 7, 0;
}

bitfield::bitfield! {
    /// ZPOSL
    pub struct ZeroPositionLsbRegister(u16);
    impl Debug;
    u8;
    // TODO: test these registers because the descript in the datasheet is in
    // conflict with the names of the registers (Figure 27)

    /// This bit enables the contribution of MAGL (magnetic field strength too
    /// low) to the error flag
    pub comp_h_error_en, set_comp_h_error_en: 7;
    /// This bit enables the contribution of MAGH (magnetic field strength too
    /// high) to the error flag
    pub comp_l_error_en, set_comp_l_error_en: 6;
    /// 6 least significant bits of the zero position
    pub zposl, set_zposl: 5, 0;
}

bitfield::bitfield! {
    /// SETTINGS1
    pub struct Settings1Register(u16);
    impl Debug;
    u8;
    /// Enables PWM (setting of UVW_ABI Bit necessary)
    pub pwmon, set_pwmon: 7;
    /// This bit defines which data can be read form address 0x3FFF.
    ///
    /// - `0` = DAECANG
    /// - `1` = CORDICANG
    pub dataselect, set_dataselect: 6;
    /// ABI decimal or binary selection of the ABI pulses per revolution
    pub abibin, set_abibin: 5;
    /// Disable Dynamic Angle Error Compensation
    ///
    /// - `0` = DAE compensation ON
    /// - `1` = DAE compensation OFF
    pub daecdis, set_daecdis: 4;
    /// Defines the PWM Output
    ///
    /// - `0` = ABI is operating, W is used as PWM
    /// - `1` = UVW is operating, I is used as PWM
    pub uvw_abi, set_uvw_abi: 3;
    /// Rotation direction
    pub dir, set_dir: 2;
}

bitfield::bitfield! {
    /// SETTINGS2
    pub struct Settings2Register(u16);
    impl Debug;
    u8;
    /// Resolution of ABI (See Figure 30)
    pub abires, set_abires: 7, 5;
    /// Hysteresis setting (See Figure 34)
    pub hys, set_hys: 4, 3;
    /// UVW number of pole pairs
    ///
    /// - `000` = 1
    /// - `001` = 2
    /// - `010` = 3
    /// - `011` = 4
    /// - `100` = 5
    /// - `101` = 6
    /// - `110` = 7
    /// - `111` = 7
    pub uvwpp, set_uvwpp: 2, 0;
}
