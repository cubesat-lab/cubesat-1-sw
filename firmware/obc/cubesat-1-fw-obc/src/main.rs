#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

#[cfg(feature = "nucleo-f767zi-board")]
use cc1101_wrapper::Cc1101Wrapper;
#[cfg(feature = "nucleo-f767zi-board")]
use nucleo_f767zi::{
    button::{Button, ButtonParameters},
    led::{LedBlue, LedGreen, LedParameters, LedRed},
    serial::{SerialParameters, SerialUartUsb},
    spi::SpiMaster3,
};
#[cfg(feature = "stm32vldiscovery-board")]
use stm32f1xx_hal::{gpio::Edge, pac, prelude::*};
#[cfg(feature = "nucleo-f767zi-board")]
use stm32f7xx_hal::{gpio::Edge, pac, prelude::*};
#[cfg(feature = "stm32vldiscovery-board")]
use stm32vldiscovery::{
    button::{Button, ButtonParameters},
    led::{LedBlue, LedGreen, LedParameters},
    serial::{SerialParameters, SerialUartUsb},
};

use fugit::HertzU32;
use panic_halt as _;
use rtic::app;
use rtic_monotonics::systick::Systick;
use rtic_monotonics::Monotonic;

#[app(device = pac, dispatchers = [TIM2])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        serial: SerialUartUsb,
    }

    #[local]
    struct Local {
        button: Button,
        led_green: LedGreen,
        led_blue: LedBlue,
        #[cfg(feature = "nucleo-f767zi-board")]
        led_red: LedRed,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Take the core and device peripherals
        let cp = ctx.core;
        let dp = ctx.device;

        // Set up the system clock. We want to run at 216MHz for this one.
        #[cfg(feature = "nucleo-f767zi-board")]
        let mut rcc = dp.RCC.constrain();
        #[cfg(feature = "stm32vldiscovery-board")]
        let rcc = dp.RCC.constrain();
        #[cfg(feature = "nucleo-f767zi-board")]
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();
        #[cfg(feature = "stm32vldiscovery-board")]
        let mut flash = dp.FLASH.constrain();
        #[cfg(feature = "stm32vldiscovery-board")]
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        #[cfg(feature = "stm32vldiscovery-board")]
        let mut afio = dp.AFIO.constrain();
        #[cfg(feature = "nucleo-f767zi-board")]
        let mut syscfg = dp.SYSCFG;

        // Initialize GPIO Ports
        #[cfg(feature = "nucleo-f767zi-board")]
        let gpiob = dp.GPIOB.split();
        #[cfg(feature = "nucleo-f767zi-board")]
        let gpioc = dp.GPIOC.split();
        #[cfg(feature = "nucleo-f767zi-board")]
        let gpiod = dp.GPIOD.split();

        #[cfg(feature = "stm32vldiscovery-board")]
        let mut gpioa = dp.GPIOA.split();
        #[cfg(feature = "stm32vldiscovery-board")]
        let mut gpioc = dp.GPIOC.split();

        // Initialize systick
        #[cfg(feature = "nucleo-f767zi-board")]
        let sysclk = (216.MHz() as HertzU32).to_Hz();
        #[cfg(feature = "stm32vldiscovery-board")]
        let sysclk = (24.MHz() as HertzU32).to_Hz();
        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cp.SYST, sysclk, systick_token);

        // Initialize LEDs
        #[cfg(feature = "nucleo-f767zi-board")]
        let led_green_parameters = LedParameters { pin: gpiob.pb0 };
        #[cfg(feature = "stm32vldiscovery-board")]
        let led_green_parameters = LedParameters {
            pin: gpioc.pc9,
            cr: &mut gpioc.crh,
        };
        let led_green = LedGreen::new(led_green_parameters);

        #[cfg(feature = "nucleo-f767zi-board")]
        let led_blue_parameters = LedParameters { pin: gpiob.pb7 };
        #[cfg(feature = "stm32vldiscovery-board")]
        let led_blue_parameters = LedParameters {
            pin: gpioc.pc8,
            cr: &mut gpioc.crh,
        };
        let led_blue = LedBlue::new(led_blue_parameters);

        #[cfg(feature = "nucleo-f767zi-board")]
        let led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

        // Initialize UART for serial communication through USB
        #[cfg(feature = "stm32vldiscovery-board")]
        let serial_parameters = SerialParameters {
            uart: dp.USART1,
            clocks: &clocks,
            pin_tx: gpioa.pa9,
            pin_rx: gpioa.pa10,
            afio: &mut afio,
            cr: &mut gpioa.crh,
        };
        #[cfg(feature = "nucleo-f767zi-board")]
        let serial_parameters = SerialParameters {
            uart: dp.USART3,
            clocks: &clocks,
            pin_tx: gpiod.pd8,
            pin_rx: gpiod.pd9,
        };
        let mut serial = SerialUartUsb::new(serial_parameters);

        serial.println("Hello RTIC!");

        // Initialize SPI3
        #[cfg(feature = "nucleo-f767zi-board")]
        let spi_3 = SpiMaster3::new(
            dp.SPI3,
            &clocks,
            &mut rcc.apb1,
            gpioc.pc9,
            gpioc.pc10,
            gpioc.pc11,
            gpioc.pc12,
        );

        // Initialize User Button
        let mut exti = dp.EXTI;

        #[cfg(feature = "nucleo-f767zi-board")]
        let button_parameters = ButtonParameters {
            pin: gpioc.pc13,
            edge: Edge::Rising,
            syscfg: &mut syscfg,
            exti: &mut exti,
            apb: &mut rcc.apb2,
        };
        #[cfg(feature = "stm32vldiscovery-board")]
        let button_parameters = ButtonParameters {
            pin: gpioa.pa0,
            edge: Edge::Falling,
            afio: &mut afio,
            exti: &mut exti,
            cr: &mut gpioa.crl,
        };
        let button = Button::new(button_parameters);

        // Initialize CC1101 Wrapper - RF Device 1
        #[cfg(feature = "nucleo-f767zi-board")]
        let mut cc1101_wrp_1 = Cc1101Wrapper::new(spi_3.spi, spi_3.cs);
        #[cfg(feature = "nucleo-f767zi-board")]
        cc1101_wrp_1.configure_radio().unwrap();

        // Spawn tasks
        task_10ms::spawn().ok();

        (
            Shared { serial },
            Local {
                button,
                led_green,
                led_blue,
                #[cfg(feature = "nucleo-f767zi-board")]
                led_red,
            },
        )
    }

    #[task(priority = 1, shared = [serial])]
    async fn task_10ms(mut ctx: task_10ms::Context) {
        loop {
            let mut instant = Systick::now();
            instant += 10.millis();

            let _10ms_task = {
                // Lock shared "serial" resource. Use it in the critical section
                ctx.shared.serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "[task_10ms] time: {}",
                        Systick::now().duration_since_epoch()
                    ));
                });
            };

            Systick::delay_until(instant).await;
        }
    }

    #[idle(shared = [serial])]
    fn idle(mut _ctx: idle::Context) -> ! {
        loop {
            let _idle_task = {
                // Do nothing
            };

            rtic::export::wfi();
        }
    }

    #[task(binds = EXTI15_10, local = [button, led_green, led_blue, led_red], shared=[serial])]
    fn button_pressed(mut ctx: button_pressed::Context) {
        // Obtain access to LEDs Peripheral and toggle them
        ctx.local.led_green.toggle();
        ctx.local.led_blue.toggle();
        #[cfg(feature = "nucleo-f767zi-board")]
        ctx.local.led_red.toggle();

        // Lock shared "serial" resource. Use it in the critical section
        ctx.shared.serial.lock(|serial| {
            serial.formatln(format_args!(
                "[button_pressed] time: {}",
                Systick::now().duration_since_epoch()
            ));
        });

        // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
        ctx.local.button.clear_interrupt_pending_bit();
    }
}
