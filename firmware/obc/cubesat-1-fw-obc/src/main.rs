#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use fugit::HertzU32;
use fugit::ExtU64;
use panic_halt as _;
use rtic::app;
use sys_time::prelude::*;

#[cfg(feature = "nucleo-f767zi-board")]
mod nucleo_f767zi_board {
    use super::*;
    use cc1101_wrapper::{Cc1101Wrapper, PACKET_LENGTH};
    use nucleo_f767zi::{
        button::{Button, ButtonParameters},
        event_pin::{EventPinCc1101Gdo2, EventPinParameters},
        led::{LedBlue, LedGreen, LedParameters, LedRed},
        serial::{SerialParameters, SerialUartUsb},
        spi::SpiMaster3,
        spi_adapter::SpiAdapter,
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
        type Cc1101SpiAdapter = SpiAdapter<SPI, CS>;

        #[shared]
        struct Shared {
            serial: SerialUartUsb,
            button_int_signal: bool,
            cc1101_int_signal: bool,
        }

        #[local]
        struct Local {
            button: Button,
            led_green: LedGreen,
            led_blue: LedBlue,
            led_red: LedRed,
            cc1101_int: EventPinCc1101Gdo2,
            cc1101_wrp: Cc1101Wrapper<Cc1101SpiAdapter>,
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
            let mut exti = dp.EXTI;

            // Initialize GPIO Ports
            let gpiob = dp.GPIOB.split();
            let gpioc = dp.GPIOC.split();
            let gpiod = dp.GPIOD.split();

            // Initialize SysTime
            let sysclk = (216.MHz() as HertzU32).to_Hz();
            SysTime::start(cp.SYST, sysclk);

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
            let button = Button::new(ButtonParameters {
                pin: gpioc.pc13,
                edge: Edge::Rising,
                syscfg: &mut syscfg,
                exti: &mut exti,
                apb: &mut rcc.apb2,
                debounce_period: ExtU64::millis(150),
            });

            // Initialize CC1101 interrupt
            let cc1101_int = EventPinCc1101Gdo2::new(EventPinParameters {
                pin: gpiod.pd2,
                edge: Edge::Falling,
                syscfg: &mut syscfg,
                exti: &mut exti,
                apb: &mut rcc.apb2,
            });

            // Initialize CC1101 Wrapper - RF Transceiver
            let cc1101_wrp = Cc1101Wrapper::new(SpiAdapter::new(spi_3.spi, spi_3.cs));

            // Spawn tasks
            task_10ms::spawn().ok();
            task_rf_com::spawn().ok();

            // Return
            (
                Shared {
                    serial,
                    button_int_signal: false,
                    cc1101_int_signal: false,
                },
                Local {
                    button,
                    led_green,
                    led_blue,
                    led_red,
                    cc1101_int,
                    cc1101_wrp,
                },
            )
        }

        #[allow(unused_variables, unused_mut)]
        #[task(priority = 1, shared = [serial])]
        async fn task_10ms(mut ctx: task_10ms::Context) {
            loop {
                let mut instant = SysTime::now();
                instant += ExtU64::millis(10);

                #[cfg(feature = "task_10ms")]
                let _task_10ms = {
                    // Lock shared "serial" resource. Use it in the critical section
                    ctx.shared.serial.lock(|serial| {
                        serial.formatln(format_args!(
                            "[task_10ms] time: {}",
                            SysTime::now().duration_since_epoch()
                        ));
                    });
                };

                SysTime::delay_until(instant).await;
            }
        }

        #[task(priority = 2, local = [cc1101_wrp], shared = [button_int_signal, cc1101_int_signal, serial])]
        async fn task_rf_com(mut ctx: task_rf_com::Context) {
            ctx.local.cc1101_wrp.init_config().unwrap();

            SysTime::delay(ExtU64::millis(100)).await;

            loop {
                let _task_rf_com = {
                    let mut button_int_flag = false;
                    let mut cc1101_int_flag = false;
                    let mut data_rx: [u8; PACKET_LENGTH as usize] = [0; PACKET_LENGTH as usize];
                    let mut data_tx: [u8; PACKET_LENGTH as usize] = [0; PACKET_LENGTH as usize];
                    let mut rssi: i16 = 0;
                    let mut lqi: u8 = 0;

                    // Prepare Tx data
                    let _setup_data_tx = {
                        for (index, element) in data_tx.iter_mut().enumerate() {
                            *element = index as u8;
                        }
                    };

                    // Lock shared "button_int_signal" resource. Use it in the critical section
                    ctx.shared.button_int_signal.lock(|signal| {
                        button_int_flag = *signal;
                        *signal = false;
                    });

                    // Lock shared "cc1101_int_signal" resource. Use it in the critical section
                    ctx.shared.cc1101_int_signal.lock(|signal| {
                        cc1101_int_flag = *signal;
                        *signal = false;
                    });

                    // Test Code: Generate Tx data
                    if button_int_flag {
                        let _ = ctx
                            .local
                            .cc1101_wrp
                            .write_data(&data_tx[0..(PACKET_LENGTH as usize)]);
                    }

                    // Handle Rx interrupt for CC1101
                    if cc1101_int_flag {
                        ctx.local.cc1101_wrp.signal_rx_int();
                    }

                    // Process RF
                    ctx.local.cc1101_wrp.main().await;

                    if ctx.local.cc1101_wrp.is_data_received() {
                        ctx.local
                            .cc1101_wrp
                            .read_data(&mut data_rx, &mut rssi, &mut lqi)
                            .unwrap();

                        // Test Code: Consume Rx data
                        // Lock shared "serial" resource. Use it in the critical section
                        ctx.shared.serial.lock(|serial| {
                            serial.formatln(format_args!(
                                "[task_rf_com] Rx (len: {}, rssi: {}, lqi: {}): {:02X?}",
                                PACKET_LENGTH,
                                rssi,
                                lqi,
                                &data_rx[0..(PACKET_LENGTH as usize)]
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
                    SysTime::delay(ExtU64::millis(10)).await;
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

        #[task(binds = EXTI15_10, local = [button, led_green, led_blue, led_red], shared=[button_int_signal, serial])]
        fn button_isr(mut ctx: button_isr::Context) {
            let instant = SysTime::now();
            let debounced: bool = (instant - ctx.local.button.debounce_instant)
                > ctx.local.button.get_debounce_period();

            // Check if debounce time elapsed
            if debounced {
                ctx.local.button.debounce_instant = instant;

                // Lock shared "button_int_signal" resource. Use it in the critical section
                ctx.shared.button_int_signal.lock(|signal| {
                    *signal = true;
                });

                // Obtain access to LEDs Peripheral and toggle them
                ctx.local.led_green.toggle();
                ctx.local.led_blue.toggle();
                ctx.local.led_red.toggle();

                // Lock shared "serial" resource. Use it in the critical section
                ctx.shared.serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "[button_isr] time: {}",
                        SysTime::now().duration_since_epoch()
                    ));
                });
            }

            // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
            ctx.local.button.clear_interrupt_pending_bit();
        }

        #[task(binds = EXTI2, local = [cc1101_int], shared=[cc1101_int_signal, serial])]
        fn cc1101_isr(mut ctx: cc1101_isr::Context) {
            // Lock shared "cc1101_int_signal" resource. Use it in the critical section
            ctx.shared.cc1101_int_signal.lock(|signal| {
                *signal = true;
            });

            // Lock shared "serial" resource. Use it in the critical section
            ctx.shared.serial.lock(|serial| {
                serial.formatln(format_args!(
                    "[cc1101_isr] time: {}",
                    SysTime::now().duration_since_epoch()
                ));
            });

            // Obtain access to CC1101 Interrupt Pin and Clear Interrupt Pending Flag
            ctx.local.cc1101_int.clear_interrupt_pending_bit();
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

            // Initialize SysTime
            let sysclk = (24.MHz() as HertzU32).to_Hz();
            SysTime::start(cp.SYST, sysclk);

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
                let mut instant = SysTime::now();
                instant += ExtU64::millis(10);

                let _10ms_task = {
                    // Lock shared "serial" resource. Use it in the critical section
                    ctx.shared.serial.lock(|serial| {
                        serial.formatln(format_args!(
                            "[task_10ms] time: {}",
                            SysTime::now().duration_since_epoch()
                        ));
                    });
                };

                SysTime::delay_until(instant).await;
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
        fn button_isr(mut ctx: button_isr::Context) {
            // Obtain access to LEDs Peripheral and toggle them
            ctx.local.led_green.toggle();
            ctx.local.led_blue.toggle();

            // Lock shared "serial" resource. Use it in the critical section
            ctx.shared.serial.lock(|serial| {
                serial.formatln(format_args!(
                    "[button_isr] time: {}",
                    SysTime::now().duration_since_epoch()
                ));
            });

            // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
            ctx.local.button.clear_interrupt_pending_bit();
        }
    }
}
