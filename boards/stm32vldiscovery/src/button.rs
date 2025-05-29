use stm32f1xx_hal::{
    afio::Parts,
    gpio::{Edge, ExtiPin, Input, Pin, PullDown, HL},
    pac::EXTI,
};
use sys_time::prelude::*;

pub struct ButtonParameters<'a> {
    pub pin: Pin<'A', 0>,
    pub edge: Edge,
    // pub syscfg: &'a mut SYSCFG,
    pub exti: &'a mut EXTI,
    pub afio: &'a mut Parts,
    pub cr: &'a mut <Pin<'A', 0> as HL>::Cr,
    pub debounce_period: TimeDuration,
}

pub struct Button {
    btn: Pin<'A', 0, Input<PullDown>>,
    debounce_period: TimeDuration,
    pub debounce_instant: TimeInstant,
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

        Self {
            btn: button,
            debounce_period: button_parameters.debounce_period,
            debounce_instant: SysTime::now(),
        }
    }

    pub fn clear_interrupt_pending_bit(&mut self) {
        self.btn.clear_interrupt_pending_bit();
    }

    pub fn check_interrupt(&mut self) -> bool {
        self.btn.check_interrupt()
    }

    pub fn get_debounce_period(&mut self) -> TimeDuration {
        self.debounce_period
    }
}
