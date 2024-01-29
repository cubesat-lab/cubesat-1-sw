use embedded_hal::digital::v2::PinState;
use stm32f1xx_hal::gpio::{Output, Pin, PinState as BasePinState, HL};

pub struct LedParameters<'a, const P: char, const N: u8>
where
    Pin<P, N>: HL,
{
    pub pin: Pin<P, N>,
    pub cr: &'a mut <Pin<P, N> as HL>::Cr,
}

pub struct Led<const P: char, const N: u8> {
    led: Pin<P, N, Output>,
}

impl<const P: char, const N: u8> Led<P, N>
where
    Pin<P, N>: HL,
{
    pub fn new(led_parameters: LedParameters<P, N>) -> Self {
        let led = led_parameters.pin.into_push_pull_output(led_parameters.cr);
        Self { led }
    }

    pub fn set_state(&mut self, state: PinState) {
        let base_state = match state {
            PinState::Low => BasePinState::Low,
            PinState::High => BasePinState::High,
        };

        self.led.set_state(base_state);
    }

    pub fn toggle(&mut self) {
        self.led.toggle();
    }
}

pub type LedGreen = Led<'C', 9>;
pub type LedBlue = Led<'C', 8>;
