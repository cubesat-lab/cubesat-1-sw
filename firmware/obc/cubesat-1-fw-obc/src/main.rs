#![no_main]
#![no_std]

use panic_halt as _;
use rtic::{app, Mutex};
use sys_time::prelude::*;

#[cfg(any(feature = "nucleo-f446re-board", feature = "nucleo-f767zi-board"))]
mod nucleo_fxxxxx_board {
    use super::*;
    use cc1101_wrapper::{Cc1101Wrapper, PACKET_LENGTH};

    #[cfg(feature = "nucleo-f446re-board")]
    use nucleo_f446re::{
        button::{Button, ButtonParameters},
        event_pin::{EventPinCc1101Gdo2, EventPinParameters},
        led::{LedGreen, LedParameters},
        serial::{SerialParameters, SerialUartUsb},
        spi::{SpiMaster, SpiParameters},
    };
    #[cfg(feature = "nucleo-f446re-board")]
    use stm32f4xx_hal::{
        gpio::{Edge, Pull},
        pac::{self, SPI2},
        prelude::*,
    };

    #[cfg(feature = "nucleo-f767zi-board")]
    use nucleo_f767zi::{
        button::{Button, ButtonParameters},
        event_pin::{EventPinCc1101Gdo2, EventPinParameters},
        led::{LedBlue, LedGreen, LedParameters, LedRed},
        serial::{SerialParameters, SerialUartUsb},
        spi::SpiMaster3,
        spi_adapter::SpiAdapter,
    };
    #[cfg(feature = "nucleo-f767zi-board")]
    use stm32f7xx_hal::{
        gpio::{Alternate, Edge, Output, Pin},
        pac::{self, SPI3},
        prelude::*,
        spi::{Enabled, Spi},
    };

    #[app(device = pac, dispatchers = [TIM2, TIM3])]
    mod app {
        use super::*;
        use shared_resources::{
            button_int_signal_that_needs_to_be_locked, button_that_needs_to_be_locked,
            cc1101_int_signal_that_needs_to_be_locked, cc1101_int_that_needs_to_be_locked,
            led_blue_that_needs_to_be_locked, led_green_that_needs_to_be_locked,
            led_red_that_needs_to_be_locked, serial_that_needs_to_be_locked,
        };

        #[cfg(feature = "nucleo-f446re-board")]
        type LedBlue = ();
        #[cfg(feature = "nucleo-f446re-board")]
        type LedRed = ();

        #[cfg(feature = "nucleo-f767zi-board")]
        type SPI = Spi<
            SPI3,
            (
                Pin<'C', 10, Alternate<6>>,
                Pin<'C', 11, Alternate<6>>,
                Pin<'C', 12, Alternate<6>>,
            ),
            Enabled<u8>,
        >;
        #[cfg(feature = "nucleo-f767zi-board")]
        type CS = Pin<'C', 9, Output>;

        #[cfg(feature = "nucleo-f446re-board")]
        type Cc1101GenericSpi = SpiMaster<SPI2>;

        #[cfg(feature = "nucleo-f767zi-board")]
        type Cc1101GenericSpi = SpiAdapter<SPI, CS>;

        #[cfg(feature = "nucleo-f446re-board")]
        const SYS_CLK: FreqSize = FreqSize::MHz(180);
        #[cfg(feature = "nucleo-f767zi-board")]
        const SYS_CLK: FreqSize = FreqSize::MHz(216);

        #[shared]
        struct Shared {
            serial: SerialUartUsb,
            button_int_signal: bool,
            cc1101_int_signal: bool,
            button: Button,
            led_green: LedGreen,
            led_blue: LedBlue,
            led_red: LedRed,
            cc1101_int: EventPinCc1101Gdo2,
        }

        #[local]
        struct Local {
            cc1101_wrp: Cc1101Wrapper<Cc1101GenericSpi>,
        }

        #[init]
        fn init(ctx: init::Context) -> (Shared, Local) {
            // Take the core and device peripherals
            let cp = ctx.core;
            let dp = ctx.device;

            // Set up the system clock. We want to run at 216MHz for this one.
            #[cfg(feature = "nucleo-f446re-board")]
            let rcc = dp.RCC.constrain();
            #[cfg(feature = "nucleo-f767zi-board")]
            let mut rcc = dp.RCC.constrain();
            let clocks = rcc.cfgr.sysclk(SYS_CLK).freeze();
            #[cfg(feature = "nucleo-f446re-board")]
            let mut syscfg = dp.SYSCFG.constrain();
            #[cfg(feature = "nucleo-f767zi-board")]
            let mut syscfg = dp.SYSCFG;
            let mut exti = dp.EXTI;

            // Initialize GPIO Ports
            #[cfg(feature = "nucleo-f446re-board")]
            let gpioa = dp.GPIOA.split();
            let gpiob = dp.GPIOB.split();
            let gpioc = dp.GPIOC.split();
            #[cfg(feature = "nucleo-f767zi-board")]
            let gpiod = dp.GPIOD.split();

            // Initialize SysTime
            let sysclk = SYS_CLK.to_Hz();
            SysTime::start(cp.SYST, sysclk);

            // Initialize LEDs
            #[cfg(feature = "nucleo-f446re-board")]
            let pin_led_green = gpioa.pa5;
            #[cfg(feature = "nucleo-f767zi-board")]
            let pin_led_green = gpiob.pb0;
            let led_green = LedGreen::new(LedParameters { pin: pin_led_green });
            #[cfg(feature = "nucleo-f446re-board")]
            let led_blue = LedBlue::default();
            #[cfg(feature = "nucleo-f767zi-board")]
            let led_blue = LedBlue::new(LedParameters { pin: gpiob.pb7 });
            #[cfg(feature = "nucleo-f446re-board")]
            let led_red = LedRed::default();
            #[cfg(feature = "nucleo-f767zi-board")]
            let led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

            // Initialize UART for serial communication through USB
            #[cfg(feature = "nucleo-f446re-board")]
            let serial_param = SerialParameters {
                uart: dp.USART2,
                clocks: &clocks,
                pin_tx: gpioa.pa2.into_alternate(),
                pin_rx: gpioa.pa3.into_alternate(),
            };
            #[cfg(feature = "nucleo-f767zi-board")]
            let serial_param = SerialParameters {
                uart: dp.USART3,
                clocks: &clocks,
                pin_tx: gpiod.pd8,
                pin_rx: gpiod.pd9,
            };
            let mut serial = SerialUartUsb::new(serial_param);
            serial.println("Hello RTIC!");

            // Initialize SPI3
            // #[cfg(feature = "nucleo-f446re-board")]
            // let spi_2 = SpiMaster2::new(
            //     dp.SPI3, &clocks, gpioc.pc9, gpioc.pc10, gpioc.pc11, gpioc.pc12,
            // );
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
            #[cfg(feature = "nucleo-f446re-board")]
            let button_param = ButtonParameters {
                pin: gpioc.pc13,
                edge: Edge::Rising,
                syscfg: &mut syscfg,
                exti: &mut exti,
                debounce_period: TimeSize::millis(150),
            };
            #[cfg(feature = "nucleo-f767zi-board")]
            let button_param = ButtonParameters {
                pin: gpioc.pc13,
                edge: Edge::Rising,
                syscfg: &mut syscfg,
                exti: &mut exti,
                apb: &mut rcc.apb2,
                debounce_period: TimeSize::millis(150),
            };
            let button = Button::new(button_param);

            // Initialize CC1101 interrupt
            #[cfg(feature = "nucleo-f446re-board")]
            let event_pin_gdo_2 = EventPinParameters {
                pin: gpiob.pb5,
                edge: Edge::Falling,
                pull: Pull::Up,
                syscfg: &mut syscfg,
                exti: &mut exti,
            };
            #[cfg(feature = "nucleo-f767zi-board")]
            let event_pin_gdo_2 = EventPinParameters {
                pin: gpiod.pd2,
                edge: Edge::Falling,
                syscfg: &mut syscfg,
                exti: &mut exti,
                apb: &mut rcc.apb2,
            };
            let cc1101_int = EventPinCc1101Gdo2::new(event_pin_gdo_2);

            // Initialize CC1101 Wrapper - RF Transceiver
            #[cfg(feature = "nucleo-f446re-board")]
            let spi_param = SpiParameters {
                spi: dp.SPI2,
                clocks: &clocks,
                freq: FreqSize::kHz(250),
                pin_cs: gpiob.pb4,
                pin_sck: gpiob.pb13,
                pin_miso: gpiob.pb14,
                pin_mosi: gpiob.pb15,
            };
            #[cfg(feature = "nucleo-f446re-board")]
            let spi_master_2 = SpiMaster::new(spi_param);
            #[cfg(feature = "nucleo-f446re-board")]
            let cc1101_wrp = Cc1101Wrapper::new(spi_master_2);
            #[cfg(feature = "nucleo-f767zi-board")]
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
                    button,
                    led_green,
                    led_blue,
                    led_red,
                    cc1101_int,
                },
                Local { cc1101_wrp },
            )
        }

        #[allow(unused_variables, unused_mut)]
        #[task(priority = 1, shared = [serial])]
        async fn task_10ms(mut ctx: task_10ms::Context) {
            loop {
                let mut instant = SysTime::now();
                instant += TimeSize::millis(10);

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

            SysTime::delay(TimeSize::millis(100)).await;

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
                    SysTime::delay(TimeSize::millis(10)).await;
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

        fn interrupt_router(
            mut button: button_that_needs_to_be_locked,
            mut button_int_signal: button_int_signal_that_needs_to_be_locked,
            mut cc1101_int: cc1101_int_that_needs_to_be_locked,
            mut cc1101_int_signal: cc1101_int_signal_that_needs_to_be_locked,
            mut led_green: led_green_that_needs_to_be_locked,
            mut led_blue: led_blue_that_needs_to_be_locked,
            mut led_red: led_red_that_needs_to_be_locked,
            mut serial: serial_that_needs_to_be_locked,
        ) {
            let mut known_interrupt_event: bool = false;

            // Check if interrupt occured due to Button
            button.lock(|button| {
                if button.check_interrupt() {
                    let instant = SysTime::now();
                    let debounced: bool =
                        (instant - button.debounce_instant) > button.get_debounce_period();
                    known_interrupt_event = true;

                    // Check if debounce time elapsed
                    if debounced {
                        button.debounce_instant = instant;

                        // Lock shared "button_int_signal" resource. Use it in the critical section
                        button_int_signal.lock(|signal| {
                            *signal = true;
                        });

                        // Obtain access to LEDs Peripheral and toggle them
                        led_green.lock(|led_green| {
                            led_green.toggle();
                        });
                        led_blue.lock(|led_blue| {
                            #[cfg(feature = "nucleo-f446re-board")]
                            let _ = led_blue;
                            #[cfg(feature = "nucleo-f767zi-board")]
                            led_blue.toggle();
                        });
                        led_red.lock(|led_red| {
                            #[cfg(feature = "nucleo-f446re-board")]
                            let _ = led_red;
                            #[cfg(feature = "nucleo-f767zi-board")]
                            led_red.toggle();
                        });

                        // Lock shared "serial" resource. Use it in the critical section
                        serial.lock(|serial| {
                            serial.formatln(format_args!(
                                "[button_isr] time: {}",
                                SysTime::now().duration_since_epoch()
                            ));
                        });
                    }

                    // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
                    button.clear_interrupt_pending_bit();
                }
            });

            // Check if interrupt occured due to EventPin
            cc1101_int.lock(|cc1101_int| {
                if cc1101_int.check_interrupt() {
                    known_interrupt_event = true;

                    // Lock shared "cc1101_int_signal" resource. Use it in the critical section
                    cc1101_int_signal.lock(|signal| {
                        *signal = true;
                    });

                    // Lock shared "serial" resource. Use it in the critical section
                    serial.lock(|serial| {
                        serial.formatln(format_args!(
                            "[cc1101_isr] time: {}",
                            SysTime::now().duration_since_epoch()
                        ));
                    });

                    // Obtain access to CC1101 Interrupt Pin and Clear Interrupt Pending Flag
                    cc1101_int.clear_interrupt_pending_bit();
                }
            });

            if !known_interrupt_event {
                serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "[interrupt_router] time: {} (unknown event)",
                        SysTime::now().duration_since_epoch()
                    ));
                });
            }
        }

        #[task(binds = EXTI0, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti0_isr(ctx: exti0_isr::Context) {
            interrupt_router(
                ctx.shared.button,
                ctx.shared.button_int_signal,
                ctx.shared.cc1101_int,
                ctx.shared.cc1101_int_signal,
                ctx.shared.led_green,
                ctx.shared.led_blue,
                ctx.shared.led_red,
                ctx.shared.serial,
            );
        }

        #[task(binds = EXTI1, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti1_isr(ctx: exti1_isr::Context) {
            interrupt_router(
                ctx.shared.button,
                ctx.shared.button_int_signal,
                ctx.shared.cc1101_int,
                ctx.shared.cc1101_int_signal,
                ctx.shared.led_green,
                ctx.shared.led_blue,
                ctx.shared.led_red,
                ctx.shared.serial,
            );
        }

        #[task(binds = EXTI2, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti2_isr(ctx: exti2_isr::Context) {
            interrupt_router(
                ctx.shared.button,
                ctx.shared.button_int_signal,
                ctx.shared.cc1101_int,
                ctx.shared.cc1101_int_signal,
                ctx.shared.led_green,
                ctx.shared.led_blue,
                ctx.shared.led_red,
                ctx.shared.serial,
            );
        }

        #[task(binds = EXTI3, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti3_isr(ctx: exti3_isr::Context) {
            interrupt_router(
                ctx.shared.button,
                ctx.shared.button_int_signal,
                ctx.shared.cc1101_int,
                ctx.shared.cc1101_int_signal,
                ctx.shared.led_green,
                ctx.shared.led_blue,
                ctx.shared.led_red,
                ctx.shared.serial,
            );
        }

        #[task(binds = EXTI4, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti4_isr(ctx: exti4_isr::Context) {
            interrupt_router(
                ctx.shared.button,
                ctx.shared.button_int_signal,
                ctx.shared.cc1101_int,
                ctx.shared.cc1101_int_signal,
                ctx.shared.led_green,
                ctx.shared.led_blue,
                ctx.shared.led_red,
                ctx.shared.serial,
            );
        }

        #[task(binds = EXTI9_5, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti9_5_isr(ctx: exti9_5_isr::Context) {
            interrupt_router(
                ctx.shared.button,
                ctx.shared.button_int_signal,
                ctx.shared.cc1101_int,
                ctx.shared.cc1101_int_signal,
                ctx.shared.led_green,
                ctx.shared.led_blue,
                ctx.shared.led_red,
                ctx.shared.serial,
            );
        }

        #[task(binds = EXTI15_10, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti15_10_isr(ctx: exti15_10_isr::Context) {
            interrupt_router(
                ctx.shared.button,
                ctx.shared.button_int_signal,
                ctx.shared.cc1101_int,
                ctx.shared.cc1101_int_signal,
                ctx.shared.led_green,
                ctx.shared.led_blue,
                ctx.shared.led_red,
                ctx.shared.serial,
            );
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
            let sysclk = FreqSize::MHz(24).to_Hz();
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
                instant += TimeSize::millis(10);

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
