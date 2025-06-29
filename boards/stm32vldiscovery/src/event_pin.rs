use stm32f1xx_hal::{
    afio::Parts,
    gpio::{Edge, ExtiPin, Floating, Input, Pin},
    pac::EXTI,
};

pub struct EventPinParameters<'a, const P: char, const N: u8> {
    pub pin: Pin<P, N, Input<Floating>>,
    pub edge: Edge,
    pub afio: &'a mut Parts,
    pub exti: &'a mut EXTI,
}

pub struct EventPin<const P: char, const N: u8> {
    pin: Pin<P, N, Input<Floating>>,
}

impl<const P: char, const N: u8> EventPin<P, N> {
    pub fn new(event_pin_parameters: EventPinParameters<P, N>) -> Self {
        let mut pin = event_pin_parameters.pin;

        // Enable external interrupt on the pin
        pin.make_interrupt_source(event_pin_parameters.afio);
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
