#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use fugit::HertzU32;
use nucleo_f767zi::serial::{SerialParameters, SerialUartUsb};
use panic_halt as _;
use rtic::app;
use rtic_monotonics::systick::Systick;
use rtic_monotonics::Monotonic;
use stm32f7xx_hal::prelude::*;

#[app(device = stm32f7xx_hal::pac, dispatchers = [TIM2, TIM3])]
mod app {
    use super::*;

    const TAB: &str = "    ";

    #[shared]
    struct Shared {
        serial: SerialUartUsb,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Take the core and device peripherals
        let cp = ctx.core;
        let dp = ctx.device;

        // Set up the system clock. We want to run at 216MHz for this one.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

        // Initialize GPIO Ports
        let gpiod = dp.GPIOD.split();

        // Initialize systick
        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cp.SYST, (216.MHz() as HertzU32).to_Hz(), systick_token);

        // Initialize UART for serial communication through USB
        let serial_parameters = SerialParameters {
            uart: dp.USART3,
            clocks: &clocks,
            pin_tx: gpiod.pd8,
            pin_rx: gpiod.pd9,
        };
        let mut serial = SerialUartUsb::new(serial_parameters);
        serial.println("Hello RTIC!");

        // Spawn tasks
        task_20ms::spawn().ok();
        task_100ms::spawn().ok();

        (Shared { serial }, Local {})
    }

    #[task(priority = 2, shared = [serial])]
    async fn task_20ms(mut ctx: task_20ms::Context) {
        loop {
            let mut instant = Systick::now();
            instant += 20.millis();

            let _10ms_task = {
                // Lock shared "serial" resource. Use it in the critical section
                ctx.shared.serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "{}[task_20ms] time: {}",
                        TAB,
                        Systick::now().duration_since_epoch()
                    ));
                });
            };

            Systick::delay_until(instant).await;
        }
    }

    #[task(priority = 1, shared = [serial])]
    async fn task_100ms(mut ctx: task_100ms::Context) {
        loop {
            let mut instant = Systick::now();
            instant += 100.millis();

            let _100ms_task = {
                // Lock shared "serial" resource. Use it in the critical section
                ctx.shared.serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "{}{}[task_100ms] time: {}",
                        TAB,
                        TAB,
                        Systick::now().duration_since_epoch()
                    ));
                });
            };

            Systick::delay_until(instant).await;
        }
    }

    #[idle(shared = [serial])]
    fn idle(mut ctx: idle::Context) -> ! {
        loop {
            let _idle_task = {
                ctx.shared.serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "[idle] time: {}",
                        Systick::now().duration_since_epoch()
                    ));
                });
            };

            // Perform a primitive delay (async delay is not permitted in idle task)
            for _ in 0..1_000 {
                rtic::export::nop();
            }

            rtic::export::wfi();
        }
    }
}
