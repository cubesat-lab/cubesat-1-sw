#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use fugit::HertzU32;
use panic_halt as _;
use rtic::app;
use rtic_monotonics::{systick::Systick, Monotonic};

#[cfg(feature = "nucleo-f767zi-board")]
mod nucleo_f767zi_board {
    use super::*;
    use cc1101_wrapper::{Cc1101Wrapper, Cc1101WrapperError};
    use nucleo_f767zi::{
        button::{Button, ButtonParameters},
        led::{LedBlue, LedGreen, LedParameters, LedRed},
        serial::{SerialParameters, SerialUartUsb},
        spi::SpiMaster3,
    };
    use stm32f7xx_hal::{gpio::Edge, pac, prelude::*};

    #[app(device = pac, dispatchers = [TIM2, TIM3])]
    mod app {
        use super::*;

        type SPI = stm32f7xx_hal::spi::Spi<
            stm32f7xx_hal::pac::SPI3,
            (
                stm32f7xx_hal::gpio::Pin<'C', 10, stm32f7xx_hal::gpio::Alternate<6>>,
                stm32f7xx_hal::gpio::Pin<'C', 11, stm32f7xx_hal::gpio::Alternate<6>>,
                stm32f7xx_hal::gpio::Pin<'C', 12, stm32f7xx_hal::gpio::Alternate<6>>,
            ),
            stm32f7xx_hal::spi::Enabled<u8>,
        >;
        type CS = stm32f7xx_hal::gpio::Pin<'C', 9, stm32f7xx_hal::gpio::Output>;

        #[derive(PartialEq)]
        enum TestDeviceRf {
            Transmitter,
            Receiver,
        }

        const THIS_TEST_DEVICE_RF: TestDeviceRf = TestDeviceRf::Receiver;

        #[shared]
        struct Shared {
            serial: SerialUartUsb,
            button_signal: bool,
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
            let mut syscfg = dp.SYSCFG;

            // Initialize GPIO Ports
            let gpiob = dp.GPIOB.split();
            let gpioc = dp.GPIOC.split();
            let gpiod = dp.GPIOD.split();

            // Initialize systick
            let sysclk = (216.MHz() as HertzU32).to_Hz();
            let systick_token = rtic_monotonics::create_systick_token!();
            Systick::start(cp.SYST, sysclk, systick_token);

            // Initialize LEDs
            let led_green = LedGreen::new(LedParameters { pin: gpiob.pb0 });
            let led_blue = LedBlue::new(LedParameters { pin: gpiob.pb7 });
            let led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

            // Initialize UART for serial communication through USB
            let mut serial = SerialUartUsb::new(SerialParameters {
                uart: dp.USART3,
                clocks: &clocks,
                pin_tx: gpiod.pd8,
                pin_rx: gpiod.pd9,
            });
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
            let mut exti = dp.EXTI;
            let button = Button::new(ButtonParameters {
                pin: gpioc.pc13,
                edge: Edge::Rising,
                syscfg: &mut syscfg,
                exti: &mut exti,
                apb: &mut rcc.apb2,
            });

            // Initialize CC1101 Wrapper - RF Transceiver
            let cc1101_wrp = Cc1101Wrapper::new(spi_3.spi, spi_3.cs);

            // Spawn tasks
            task_10ms::spawn().ok();
            task_rf_com::spawn().ok();

            // Return
            (
                Shared {
                    serial,
                    button_signal: false,
                },
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

                #[cfg(feature = "task_10ms")]
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

        #[task(priority = 2, local = [cc1101_wrp], shared = [button_signal, serial])]
        async fn task_rf_com(mut ctx: task_rf_com::Context) {
            ctx.local.cc1101_wrp.init_config().unwrap();
            Systick::delay(10.millis().into()).await;

            loop {
                let _task_rf_com = {
                    let mut signal_received = false;
                    let mut data_rx: [u8; 64] = [0; 64];
                    let mut data_tx: [u8; 16] = [0xAA; 16];
                    data_tx[0] = 10;
                    data_tx[1] = 10;
                    let mut length = 0;

                    // Lock shared "button_signal" resource. Use it in the critical section
                    ctx.shared.button_signal.lock(|signal| {
                        signal_received = *signal;
                        *signal = false;
                    });

                    // Test Code: Generate Tx data
                    if signal_received {
                        let _ = ctx.local.cc1101_wrp.write_data(&mut data_tx);
                    }

                    // Process RF
                    ctx.local.cc1101_wrp.main().await;

                    if ctx.local.cc1101_wrp.is_data_received() {
                        ctx.local
                            .cc1101_wrp
                            .read_data(&mut data_rx, &mut length)
                            .unwrap();

                        // Test Code: Consume Rx data
                        // Lock shared "serial" resource. Use it in the critical section
                        ctx.shared.serial.lock(|serial| {
                            serial.formatln(format_args!(
                                "[task_rf_com] Rx ({}): {:02X?}",
                                length,
                                &data_rx[0..(length as usize)]
                            ));
                        });
                    }

                    // Test Code: Consume last error
                    let (error_option, error_count) = ctx.local.cc1101_wrp.read_last_error();
                    if let Some(error) = error_option {
                        // Lock shared "serial" resource. Use it in the critical section
                        ctx.shared.serial.lock(|serial| {
                            serial.formatln(format_args!(
                                "[task_rf_com] Error: {:?}, {}",
                                error, error_count
                            ));
                        });
                    }

                    // Test Code: Simulate other activity
                    Systick::delay(10.millis().into()).await;
                };
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

        #[task(binds = EXTI15_10, local = [button, led_green, led_blue, led_red], shared=[button_signal, serial])]
        fn button_pressed(mut ctx: button_pressed::Context) {
            // Lock shared "button_signal" resource. Use it in the critical section
            ctx.shared.button_signal.lock(|signal| {
                *signal = true;
            });

            // Obtain access to LEDs Peripheral and toggle them
            ctx.local.led_green.toggle();
            ctx.local.led_blue.toggle();
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
}

#[cfg(feature = "stm32vldiscovery-board")]
mod stm32vldiscovery_board {
    use super::*;
    use stm32f1xx_hal::{gpio::Edge, pac, prelude::*};
    use stm32vldiscovery::{
        button::{Button, ButtonParameters},
        led::{LedBlue, LedGreen, LedParameters},
        serial::{SerialParameters, SerialUartUsb},
    };

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
        }

        #[init]
        fn init(ctx: init::Context) -> (Shared, Local) {
            // Take the core and device peripherals
            let cp = ctx.core;
            let dp = ctx.device;

            // Set up the system clock. We want to run at 216MHz for this one.
            let rcc = dp.RCC.constrain();
            let mut flash = dp.FLASH.constrain();
            let clocks = rcc.cfgr.freeze(&mut flash.acr);
            let mut afio = dp.AFIO.constrain();

            // Initialize GPIO Ports
            let mut gpioa = dp.GPIOA.split();
            let mut gpioc = dp.GPIOC.split();

            // Initialize systick
            let sysclk = (24.MHz() as HertzU32).to_Hz();
            let systick_token = rtic_monotonics::create_systick_token!();
            Systick::start(cp.SYST, sysclk, systick_token);

            // Initialize LEDs
            let led_green = LedGreen::new(LedParameters {
                pin: gpioc.pc9,
                cr: &mut gpioc.crh,
            });
            let led_blue = LedBlue::new(LedParameters {
                pin: gpioc.pc8,
                cr: &mut gpioc.crh,
            });

            // Initialize UART for serial communication through USB
            let mut serial = SerialUartUsb::new(SerialParameters {
                uart: dp.USART1,
                clocks: &clocks,
                pin_tx: gpioa.pa9,
                pin_rx: gpioa.pa10,
                afio: &mut afio,
                cr: &mut gpioa.crh,
            });
            serial.println("Hello RTIC!");

            // Initialize SPI3
            // #[cfg(feature = "nucleo-f767zi-board")]
            // let spi_3 = SpiMaster3::new(
            //     dp.SPI3,
            //     &clocks,
            //     &mut rcc.apb1,
            //     gpioc.pc9,
            //     gpioc.pc10,
            //     gpioc.pc11,
            //     gpioc.pc12,
            // );

            // Initialize User Button
            let mut exti = dp.EXTI;
            let button = Button::new(ButtonParameters {
                pin: gpioa.pa0,
                edge: Edge::Falling,
                afio: &mut afio,
                exti: &mut exti,
                cr: &mut gpioa.crl,
            });

            // Initialize CC1101 Wrapper - RF Device 1
            // #[cfg(feature = "nucleo-f767zi-board")]
            // let mut cc1101_wrp_1 = Cc1101Wrapper::new(spi_3.spi, spi_3.cs);
            // #[cfg(feature = "nucleo-f767zi-board")]
            // cc1101_wrp_1.configure_radio().unwrap();

            // Spawn tasks
            task_10ms::spawn().ok();

            // Return
            (
                Shared { serial },
                Local {
                    button,
                    led_green,
                    led_blue,
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

        #[task(binds = EXTI0, local = [button, led_green, led_blue], shared=[serial])]
        fn button_pressed(mut ctx: button_pressed::Context) {
            // Obtain access to LEDs Peripheral and toggle them
            ctx.local.led_green.toggle();
            ctx.local.led_blue.toggle();

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
}
