/// Calculate even parity bit for the lower 15 bits of a 16-bit value
pub fn calculate_parity(value: u16) -> bool {
    let bits = value & 0x7FFF;
    bits.count_ones() % 2 == 1
}

/// Verify even parity of a 16-bit frame
pub fn verify_parity(frame: u16) -> bool {
    frame.count_ones().is_multiple_of(2)
}
