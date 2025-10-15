/// Error type for AS5048A operations
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// Communication error with the sensor
    Communication(E),
    /// Parity error in received data
    ParityError,
    /// Error flag set by the sensor (invalid command or parity error)
    SensorError,
}

impl<E: Clone> Clone for Error<E> {
    fn clone(&self) -> Self {
        match self {
            Error::Communication(e) => Error::Communication(e.clone()),
            Error::ParityError => Error::ParityError,
            Error::SensorError => Error::SensorError,
        }
    }
}
