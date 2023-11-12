#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use cc1101_wrapper::Cc1101Wrapper;
use fugit::HertzU32;
use nucleo_f767zi::{
    button::Button,
    led::{LedBlue, LedGreen, LedRed},
    serial::SerialUartUsb,
    spi::SpiMaster3,
};
use panic_halt as _;
use rtic::app;
use rtic_monotonics::systick::Systick;
use rtic_monotonics::Monotonic;
use stm32f7xx_hal::{gpio::Edge, prelude::*};

#[app(device = stm32f7xx_hal::pac, dispatchers = [TIM2])]
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
        let led_green = LedGreen::new(gpiob.pb0);
        let led_blue = LedBlue::new(gpiob.pb7);
        let led_red = LedRed::new(gpiob.pb14);

        // Initialize UART for serial communication through USB
        let mut serial = SerialUartUsb::new(dp.USART3, &clocks, gpiod.pd8, gpiod.pd9);

        serial.println("Hello RTIC!");

        // Initialize SPI3
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
        let mut syscfg = dp.SYSCFG;
        let mut exti = dp.EXTI;
        let mut button = Button::new(gpioc.pc13);
        button.enable_interrupt(Edge::Rising, &mut syscfg, &mut exti, &mut rcc.apb2);

        // Initialize CC1101 Wrapper - RF Device 1
        let mut cc1101_wrp_1 = Cc1101Wrapper::new(spi_3.spi, spi_3.cs);
        cc1101_wrp_1.configure_radio().unwrap();

        // Spawn tasks
        task_10ms::spawn().ok();

        (
            Shared { serial },
            Local {
                button,
                led_green,
                led_blue,
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

    #[task(binds = EXTI15_10, local = [button, led_green, led_blue, led_red], shared=[])]
    fn button_pressed(ctx: button_pressed::Context) {
        // Obtain access to LEDs Peripheral and toggle them
        ctx.local.led_green.toggle();
        ctx.local.led_blue.toggle();
        ctx.local.led_red.toggle();

        // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
        ctx.local.button.clear_interrupt_pending_bit();
    }
}
