/// Error type for AS5047D operations
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// Communication error with the sensor
    Communication(E),
    /// Parity error in received data
    ParityError,
    /// Error flag set by the sensor (invalid command or parity error)
    SensorError,
}
