use stm32f7xx_hal::{
    gpio::{Edge, ExtiPin, Input, Pin},
    pac::{EXTI, SYSCFG},
    rcc::APB2,
};

pub struct EventPinParameters<'a, const P: char, const N: u8> {
    pub pin: Pin<P, N>,
    pub edge: Edge,
    pub syscfg: &'a mut SYSCFG,
    pub exti: &'a mut EXTI,
    pub apb: &'a mut APB2,
}

pub struct EventPin<const P: char, const N: u8> {
    pin: Pin<P, N, Input>,
}

impl<const P: char, const N: u8> EventPin<P, N> {
    pub fn new(event_pin_parameters: EventPinParameters<P, N>) -> Self {
        let mut pin = event_pin_parameters.pin.into_floating_input();

        // Enable external interrupt on the pin
        pin.make_interrupt_source(event_pin_parameters.syscfg, event_pin_parameters.apb);
        pin.trigger_on_edge(event_pin_parameters.exti, event_pin_parameters.edge);
        pin.enable_interrupt(event_pin_parameters.exti);

        Self { pin }
    }

    pub fn clear_interrupt_pending_bit(&mut self) {
        self.pin.clear_interrupt_pending_bit();
    }
}

pub type EventPinCc1101Gdo2 = EventPin<'D', 2>;
