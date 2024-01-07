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

#[app(device = stm32f7xx_hal::pac, dispatchers = [TIM2, TIM3])]
mod app {
    use super::*;

    type SPI = stm32f7xx_hal::spi::Spi<stm32f7xx_hal::pac::SPI3, (stm32f7xx_hal::gpio::Pin<'C', 10, stm32f7xx_hal::gpio::Alternate<6>>, stm32f7xx_hal::gpio::Pin<'C', 11, stm32f7xx_hal::gpio::Alternate<6>>, stm32f7xx_hal::gpio::Pin<'C', 12, stm32f7xx_hal::gpio::Alternate<6>>), stm32f7xx_hal::spi::Enabled<u8>>;
    type CS = stm32f7xx_hal::gpio::Pin<'C', 9, stm32f7xx_hal::gpio::Output>;

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
        cc1101_wrp: Cc1101Wrapper<SPI, CS>,
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

        // Initialize CC1101 Wrapper - RF Transceiver
        let cc1101_wrp = Cc1101Wrapper::new(spi_3.spi, spi_3.cs);

        // Spawn tasks
        task_10ms::spawn().ok();
        task_rf_test::spawn().ok();

        (
            Shared { serial },
            Local {
                button,
                led_green,
                led_blue,
                led_red,
                cc1101_wrp,
            },
        )
    }

    #[task(priority = 1, shared = [serial])]
    async fn task_10ms(mut ctx: task_10ms::Context) {
        loop {
            let mut instant = Systick::now();
            instant += 10.millis();

            let _task_10ms = {
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

    #[task(priority = 2, local = [cc1101_wrp], shared = [])]
    async fn task_rf_test(ctx: task_rf_test::Context) {

        ctx.local.cc1101_wrp.configure_radio().unwrap();
        Systick::delay(10.millis().into()).await;
        ctx.local.cc1101_wrp.sandbox().await.unwrap();
        Systick::delay(100.millis().into()).await;

        loop {
            let _task_rf_test = {
                // Do nothing
            };

            Systick::delay(100.millis().into()).await;
        }
    }

    #[idle(shared = [serial])]
    fn idle(mut _ctx: idle::Context) -> ! {
        loop {
            let _idle = {
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
