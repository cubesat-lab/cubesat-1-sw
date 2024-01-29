#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use fugit::HertzU32;
use nucleo_f767zi::{
    button::{Button, ButtonParameters},
    led::{LedBlue, LedGreen, LedParameters, LedRed},
    serial::{SerialParameters, SerialUartUsb},
};
use panic_halt as _;
use rtic::app;
use rtic_monotonics::systick::Systick;
use rtic_monotonics::Monotonic;
use stm32f7xx_hal::{
    gpio::{Edge, PinState},
    prelude::*,
};

#[app(device = stm32f7xx_hal::pac)]
mod app {
    use super::*;

    // Led States
    pub enum LedState {
        Red,
        Blue,
        Green,
    }

    #[shared]
    struct Shared {
        serial: SerialUartUsb,
    }

    #[local]
    struct Local {
        button: Button,
        led_state: LedState,
        led_green: LedGreen,
        led_blue: LedBlue,
        led_red: LedRed,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Take the core and device peripherals
        let cp = ctx.core;
        let dp = ctx.device;

        // Set up the system clock. We want to run at 216MHz for this one.
        let mut rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

        // Initialize GPIO Ports
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();
        let gpiod = dp.GPIOD.split();

        // Initialize systick
        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cp.SYST, (216.MHz() as HertzU32).to_Hz(), systick_token);

        // Initialize LEDs
        let mut led_green = LedGreen::new(LedParameters { pin: gpiob.pb0 });
        let mut led_blue = LedBlue::new(LedParameters { pin: gpiob.pb7 });
        let mut led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

        // Initialize Led state
        let led_state = LedState::Red;
        led_green.set_state(PinState::Low);
        led_blue.set_state(PinState::Low);
        led_red.set_state(PinState::High);

        // Initialize UART for serial communication through USB
        let serial_parameters = SerialParameters {
            uart: dp.USART3,
            clocks: &clocks,
            pin_tx: gpiod.pd8,
            pin_rx: gpiod.pd9,
        };
        let mut serial = SerialUartUsb::new(serial_parameters);
        serial.println("Hello RTIC!");

        // Initialize User Button
        let mut syscfg = dp.SYSCFG;
        let mut exti = dp.EXTI;
        let button = Button::new(ButtonParameters {
            pin: gpioc.pc13,
            edge: Edge::Rising,
            syscfg: &mut syscfg,
            exti: &mut exti,
            apb: &mut rcc.apb2,
        });

        (
            Shared { serial },
            Local {
                button,
                led_state,
                led_green,
                led_blue,
                led_red,
            },
        )
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

            rtic::export::wfi();
        }
    }

    #[task(binds = EXTI15_10, local = [button, led_state, led_green, led_blue, led_red], shared=[])]
    fn button_pressed(ctx: button_pressed::Context) {
        // Obtain access to LEDs Peripheral
        let led_state = ctx.local.led_state;
        let led_green = ctx.local.led_green;
        let led_blue = ctx.local.led_blue;
        let led_red = ctx.local.led_red;

        // Perform actions on the Button push event
        match led_state {
            LedState::Red => {
                *led_state = LedState::Blue;
                led_red.toggle();
                led_blue.toggle();
            }
            LedState::Blue => {
                *led_state = LedState::Green;
                led_blue.toggle();
                led_green.toggle();
            }
            LedState::Green => {
                *led_state = LedState::Red;
                led_green.toggle();
                led_red.toggle();
            }
        };

        // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
        ctx.local.button.clear_interrupt_pending_bit();
    }
}
