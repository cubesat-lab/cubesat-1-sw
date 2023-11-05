use stm32f7xx_hal::{
    gpio::{Edge, ExtiPin, Input, Pin},
    pac::{EXTI, SYSCFG},
    rcc::APB2,
};

pub struct Button {
    btn: Pin<'C', 13, Input>,
}

impl Button {
    pub fn new(pin: Pin<'C', 13>) -> Self {
        Self {
            btn: pin.into_floating_input(),
        }
    }

    pub fn enable_interrupt(
        &mut self,
        edge: Edge,
        syscfg: &mut SYSCFG,
        exti: &mut EXTI,
        apb: &mut APB2,
    ) {
        // Enable external interrupt on PC13
        self.btn.make_interrupt_source(syscfg, apb);
        self.btn.trigger_on_edge(exti, edge);
        self.btn.enable_interrupt(exti);
    }

    pub fn clear_interrupt_pending_bit(&mut self) {
        self.btn.clear_interrupt_pending_bit();
    }
}
