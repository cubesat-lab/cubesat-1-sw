#![no_main]
#![no_std]

use embedded_hal_1::digital::PinState;
use fugit::HertzU32;
use nucleo_f446re::{
    button::{Button, ButtonParameters, Edge},
    led::{LedGreen, LedParameters},
    serial::{SerialParameters, SerialUartUsb},
};
use panic_halt as _;
use rtic::app;
use rtic_monotonics_2::systick::prelude::*;
use stm32f4xx_hal::prelude::*;

#[app(device = stm32f4xx_hal::pac)]
mod app {
    use super::*;

    systick_monotonic!(Mono, 1_000);

    #[shared]
    struct Shared {
        serial: SerialUartUsb,
    }

    #[local]
    struct Local {
        button: Button,
        led_green: LedGreen,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Take the core and device peripherals
        let cp = ctx.core;
        let dp = ctx.device;

        // Initialize systick
        Mono::start(cp.SYST, (180.MHz() as HertzU32).to_Hz());

        // Set up the system clock. We want to run at 180 MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(180.MHz()).freeze();

        // Initialize GPIO Ports
        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        // Initialize LED
        let mut led_green = LedGreen::new(LedParameters { pin: gpioa.pa5 });

        // Initialize Led state
        led_green.set_state(PinState::Low);

        // Initialize UART for serial communication through USB
        let serial_parameters = SerialParameters {
            uart: dp.USART2,
            clocks: &clocks,
            pin_tx: gpioa.pa2.into_alternate(),
            pin_rx: gpioa.pa3.into_alternate(),
        };
        let mut serial = SerialUartUsb::new(serial_parameters);
        serial.println("Hello RTIC!");

        // Initialize User Button
        let mut syscfg = dp.SYSCFG.constrain();
        let mut exti = dp.EXTI;
        let button = Button::new(ButtonParameters {
            pin: gpioc.pc13,
            edge: Edge::Falling,
            syscfg: &mut syscfg,
            exti: &mut exti,
        });

        (Shared { serial }, Local { button, led_green })
    }

    #[idle(shared = [serial])]
    fn idle(mut ctx: idle::Context) -> ! {
        loop {
            let _idle_task = {
                ctx.shared.serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "[idle] time: {}",
                        Mono::now().duration_since_epoch().to_millis()
                    ));
                });
            };

            rtic::export::wfi();
        }
    }

    #[task(binds = EXTI15_10, local = [button, led_green], shared=[])]
    fn button_pressed(ctx: button_pressed::Context) {
        // Obtain access to LED Peripheral and Perform actions on the Button push event
        ctx.local.led_green.toggle();

        // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
        ctx.local.button.clear_interrupt_pending_bit();
    }
}
