// use fugit::{Duration, Instant};
use sys_time::prelude::*;
pub use stm32f4xx_hal::gpio::Edge;
use stm32f4xx_hal::{
    gpio::{ExtiPin, Input, Pin, Pull},
    pac::EXTI,
    syscfg::SysCfg,
};

pub struct ButtonParameters<'a> {
    pub pin: Pin<'C', 13>,
    pub edge: Edge,
    pub syscfg: &'a mut SysCfg,
    pub exti: &'a mut EXTI,
    // pub debounce_period: Duration<u64, 1, 1000>,
}

pub struct Button {
    btn: Pin<'C', 13, Input>,
    // debounce_period: Duration<u64, 1, 1000>,
    // pub debounce_instant: Instant<u64, 1, 1000>,
}

impl Button {
    pub fn new(button_parameters: ButtonParameters) -> Self {
        let mut button = Input::new(button_parameters.pin, Pull::Up);

        // Enable external interrupt on PC13
        button.make_interrupt_source(button_parameters.syscfg);
        button.trigger_on_edge(button_parameters.exti, button_parameters.edge);
        button.enable_interrupt(button_parameters.exti);

        Self {
            btn: button,
            // debounce_period: button_parameters.debounce_period,
            // debounce_instant: SysTime::now(),
        }
    }

    pub fn clear_interrupt_pending_bit(&mut self) {
        self.btn.clear_interrupt_pending_bit();
    }

    // pub fn get_debounce_period(&mut self) -> Duration<u64, 1, 1000> {
    //     self.debounce_period
    // }
}
