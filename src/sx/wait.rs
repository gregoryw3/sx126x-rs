use embedded_hal_async::digital::Wait;
use embedded_hal::digital::InputPin;

/*
 * A wrapper allowing both polling and async GPIO implementations
 * of embedded-hal to be used in the library.
 */
pub trait AnyWait {
    type Error;
    async fn anywait_for_high(&mut self) -> Result<(), Self::Error>;
    async fn anywait_for_low(&mut self) -> Result<(), Self::Error>;
}

impl <T:Wait> AnyWait for T {
    type Error = T::Error;

    async fn anywait_for_high(&mut self) -> Result<(), Self::Error> {
        self.wait_for_high().await
    }

    async fn anywait_for_low(&mut self) -> Result<(), Self::Error> {
        self.wait_for_low().await
    }
}

pub struct PollingInputPin<T: InputPin>(T);

impl <T:InputPin> AnyWait for PollingInputPin<T> {
    type Error = T::Error;

    async fn anywait_for_high(&mut self) -> Result<(), Self::Error> {
        while let Ok(false) = self.0.is_high() {
            // Busy loop
        }
        Ok(())
    }

    async fn anywait_for_low(&mut self) -> Result<(), Self::Error> {
        while let Ok(false) = self.0.is_low() {
            // Busy loop
        }
        Ok(())
    }
}

impl <T:InputPin> From<T> for PollingInputPin<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
