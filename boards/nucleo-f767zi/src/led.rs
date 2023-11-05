use embedded_hal::digital::v2::PinState;
use stm32f7xx_hal::gpio::{Output, Pin};

pub struct Led<const P: char, const N: u8> {
    led: Pin<P, N, Output>,
}

impl<const P: char, const N: u8> Led<P, N> {
    pub fn new(pin: Pin<P, N>) -> Self {
        let led = pin.into_push_pull_output();
        Self { led }
    }

    pub fn set_state(&mut self, state: PinState) {
        self.led.set_state(state);
    }

    pub fn toggle(&mut self) {
        self.led.toggle();
    }
}

pub type LedGreen = Led<'B', 0>;
pub type LedBlue = Led<'B', 7>;
pub type LedRed = Led<'B', 14>;
