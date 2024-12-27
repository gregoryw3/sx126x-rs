#[derive(Debug)]
pub enum SpiError {
    Write,
    Transfer,
    /// SPI bus error.
    BusError,
    /// SPI device error.
    DeviceError,
}

pub enum PinError {
    Input,
    Output,
}
