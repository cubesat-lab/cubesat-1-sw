use embedded_hal::digital::PinState;
use stm32f4xx_hal::gpio::{Output, Pin, PinState as Stm32F4PinState};

struct _PinState(PinState);

impl From<_PinState> for Stm32F4PinState {
    fn from(value: _PinState) -> Self {
        match value.0 {
            PinState::Low => Stm32F4PinState::Low,
            PinState::High => Stm32F4PinState::High,
        }
    }
}

pub struct LedParameters<const P: char, const N: u8> {
    pub pin: Pin<P, N>,
}

pub struct Led<const P: char, const N: u8> {
    led: Pin<P, N, Output>,
}

impl<const P: char, const N: u8> Led<P, N> {
    pub fn new(led_parameters: LedParameters<P, N>) -> Self {
        let led = led_parameters.pin.into_push_pull_output();
        Self { led }
    }

    pub fn set_state(&mut self, state: PinState) {
        self.led.set_state(_PinState(state).into());
    }

    pub fn toggle(&mut self) {
        self.led.toggle();
    }
}

pub type LedGreen = Led<'A', 5>;
