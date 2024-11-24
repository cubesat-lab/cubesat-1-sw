pub use stm32f4xx_hal::gpio::Edge;
use stm32f4xx_hal::{
    gpio::{ExtiPin, Input, Pin, Pull},
    pac::EXTI,
    syscfg::SysCfg,
};

pub struct EventPinParameters<'a, const P: char, const N: u8> {
    pub pin: Pin<P, N>,
    pub edge: Edge,
    pub pull: Pull,
    pub syscfg: &'a mut SysCfg,
    pub exti: &'a mut EXTI,
}

pub struct EventPin<const P: char, const N: u8> {
    pin: Pin<P, N, Input>,
}

impl<const P: char, const N: u8> EventPin<P, N> {
    pub fn new(event_pin_parameters: EventPinParameters<P, N>) -> Self {
        let mut pin = Input::new(event_pin_parameters.pin, event_pin_parameters.pull);

        // Enable external interrupt on the pin
        pin.make_interrupt_source(event_pin_parameters.syscfg);
        pin.trigger_on_edge(event_pin_parameters.exti, event_pin_parameters.edge);
        pin.enable_interrupt(event_pin_parameters.exti);

        Self { pin }
    }

    pub fn clear_interrupt_pending_bit(&mut self) {
        self.pin.clear_interrupt_pending_bit();
    }

    pub fn check_interrupt(&mut self) -> bool {
        self.pin.check_interrupt()
    }
}

pub type EventPinCc1101Gdo2 = EventPin<'B', 5>;
