use stm32f1xx_hal::{
    afio::Parts,
    gpio::{Edge, ExtiPin, Input, Pin, PullDown, HL},
    pac::EXTI,
};

pub struct ButtonParameters<'a> {
    pub pin: Pin<'A', 0>,
    pub edge: Edge,
    pub exti: &'a mut EXTI,
    pub afio: &'a mut Parts,
    pub cr: &'a mut <Pin<'A', 0> as HL>::Cr,
}

pub struct Button {
    btn: Pin<'A', 0, Input<PullDown>>,
}

impl Button {
    pub fn new(button_parameters: ButtonParameters) -> Self {
        let mut button = button_parameters
            .pin
            .into_pull_down_input(button_parameters.cr);

        // Enable external interrupt on PA0
        button.make_interrupt_source(button_parameters.afio);
        button.trigger_on_edge(button_parameters.exti, button_parameters.edge);
        button.enable_interrupt(button_parameters.exti);

        Self { btn: button }
    }

    pub fn clear_interrupt_pending_bit(&mut self) {
        self.btn.clear_interrupt_pending_bit();
    }
}
