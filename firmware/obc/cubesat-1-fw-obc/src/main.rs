#![no_main]
#![no_std]

use panic_halt as _;
use rtic::app;
use sys_time::prelude::*;

#[cfg(any(
    feature = "nucleo-f446re-board",
    feature = "nucleo-f767zi-board",
    feature = "stm32vldiscovery-board"
))]
mod stm32_dev_board {
    use super::*;
    use cc1101_wrapper::{Cc1101Wrapper, PACKET_LENGTH};
    use rtic::Mutex;

    // Import HAL based on the selected board
    #[cfg(feature = "stm32vldiscovery-board")]
    use stm32f1xx_hal as hal;
    #[cfg(feature = "nucleo-f446re-board")]
    use stm32f4xx_hal as hal;
    #[cfg(feature = "nucleo-f767zi-board")]
    use stm32f7xx_hal as hal;

    use hal::{
        gpio::Edge,
        pac::{self},
        prelude::*,
    };

    #[cfg(feature = "nucleo-f446re-board")]
    use hal::gpio::Pull;
    #[cfg(feature = "nucleo-f767zi-board")]
    use hal::rcc::PLL48CLK;

    // Import Board crate based on the selected board
    #[cfg(feature = "nucleo-f446re-board")]
    use nucleo_f446re as board;
    #[cfg(feature = "nucleo-f767zi-board")]
    use nucleo_f767zi as board;
    #[cfg(feature = "stm32vldiscovery-board")]
    use stm32vldiscovery as board;

    use board::{
        button::{Button, ButtonParameters},
        event_pin::{EventPinCc1101Gdo2, EventPinParameters},
        led::{LedGreen, LedParameters},
        serial::{SerialParameters, SerialUartUsb},
        spi::SpiParameters,
    };

    #[cfg(feature = "nucleo-f446re-board")]
    use board::spi::SpiMaster2 as SpiCc1101;
    #[cfg(feature = "stm32vldiscovery-board")]
    use board::{led::LedBlue, spi::SpiMaster1 as SpiCc1101};
    #[cfg(feature = "nucleo-f767zi-board")]
    use board::{
        led::{LedBlue, LedRed},
        spi::SpiMaster3 as SpiCc1101,
        usb::{Usb, UsbParameters},
    };

    #[app(device = pac, dispatchers = [TIM2, TIM3, TIM4])]
    mod app {
        use super::*;
        use shared_resources::{
            button_int_signal_that_needs_to_be_locked, button_that_needs_to_be_locked,
            cc1101_int_signal_that_needs_to_be_locked, cc1101_int_that_needs_to_be_locked,
            led_blue_that_needs_to_be_locked, led_green_that_needs_to_be_locked,
            led_red_that_needs_to_be_locked, serial_that_needs_to_be_locked,
        };

        struct InterruptParameters<'a> {
            button: button_that_needs_to_be_locked<'a>,
            button_int_signal: button_int_signal_that_needs_to_be_locked<'a>,
            cc1101_int: cc1101_int_that_needs_to_be_locked<'a>,
            cc1101_int_signal: cc1101_int_signal_that_needs_to_be_locked<'a>,
            led_green: led_green_that_needs_to_be_locked<'a>,
            led_blue: led_blue_that_needs_to_be_locked<'a>,
            led_red: led_red_that_needs_to_be_locked<'a>,
            serial: serial_that_needs_to_be_locked<'a>,
        }

        #[cfg(feature = "nucleo-f446re-board")]
        type LedBlue = ();
        #[cfg(any(feature = "nucleo-f446re-board", feature = "stm32vldiscovery-board"))]
        type LedRed = ();
        #[cfg(any(feature = "nucleo-f446re-board", feature = "stm32vldiscovery-board"))]
        type Usb<'a> = ();

        #[cfg(feature = "nucleo-f446re-board")]
        const SYS_CLK: FreqSize = FreqSize::MHz(180);
        #[cfg(feature = "nucleo-f767zi-board")]
        const SYS_CLK: FreqSize = FreqSize::MHz(216);
        #[cfg(feature = "stm32vldiscovery-board")]
        const SYS_CLK: FreqSize = FreqSize::MHz(24);

        #[shared]
        struct Shared {
            button: Button,
            button_int_signal: bool,
            cc1101_int: EventPinCc1101Gdo2,
            cc1101_int_signal: bool,
            led_green: LedGreen,
            led_blue: LedBlue,
            led_red: LedRed,
            serial: SerialUartUsb,
        }

        #[local]
        struct Local {
            cc1101_wrp: Cc1101Wrapper<SpiCc1101>,
            usb: Usb<'static>,
        }

        #[init]
        fn init(ctx: init::Context) -> (Shared, Local) {
            // Take the core and device peripherals
            let cp = ctx.core;
            let dp = ctx.device;

            // Set up the system clock
            #[cfg(any(feature = "nucleo-f446re-board", feature = "stm32vldiscovery-board"))]
            let rcc = dp.RCC.constrain();
            #[cfg(feature = "nucleo-f767zi-board")]
            let mut rcc = dp.RCC.constrain();
            #[cfg(feature = "stm32vldiscovery-board")]
            let mut flash = dp.FLASH.constrain();
            #[cfg(feature = "stm32vldiscovery-board")]
            let mut afio = dp.AFIO.constrain();
            #[cfg(feature = "nucleo-f767zi-board")]
            let clocks = rcc
                .cfgr
                .use_pll()
                .use_pll48clk(PLL48CLK::Pllq)
                .sysclk(SYS_CLK)
                .freeze();
            #[cfg(feature = "nucleo-f446re-board")]
            let clocks = rcc.cfgr.sysclk(SYS_CLK).freeze();
            #[cfg(feature = "stm32vldiscovery-board")]
            let clocks = rcc.cfgr.sysclk(SYS_CLK).freeze(&mut flash.acr);
            #[cfg(feature = "nucleo-f446re-board")]
            let mut syscfg = dp.SYSCFG.constrain();
            #[cfg(feature = "nucleo-f767zi-board")]
            let mut syscfg = dp.SYSCFG;
            let mut exti = dp.EXTI;

            // Initialize GPIO Ports
            #[cfg(any(feature = "nucleo-f446re-board", feature = "nucleo-f767zi-board"))]
            let gpioa = dp.GPIOA.split();
            #[cfg(feature = "stm32vldiscovery-board")]
            let mut gpioa = dp.GPIOA.split();
            let gpiob = dp.GPIOB.split();
            #[cfg(any(feature = "nucleo-f446re-board", feature = "nucleo-f767zi-board"))]
            let gpioc = dp.GPIOC.split();
            #[cfg(feature = "stm32vldiscovery-board")]
            let mut gpioc = dp.GPIOC.split();
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
            #[cfg(feature = "stm32vldiscovery-board")]
            let pin_led_green = gpioc.pc9;
            #[cfg(feature = "nucleo-f446re-board")]
            let led_green = LedGreen::new(LedParameters { pin: pin_led_green });
            #[cfg(feature = "nucleo-f767zi-board")]
            let led_green = LedGreen::new(LedParameters { pin: pin_led_green });
            #[cfg(feature = "stm32vldiscovery-board")]
            let led_green = LedGreen::new(LedParameters {
                pin: pin_led_green,
                cr: &mut gpioc.crh,
            });
            #[cfg(feature = "nucleo-f446re-board")]
            #[allow(clippy::let_unit_value)]
            let led_blue = LedBlue::default();
            #[cfg(feature = "nucleo-f767zi-board")]
            let led_blue = LedBlue::new(LedParameters { pin: gpiob.pb7 });
            #[cfg(feature = "stm32vldiscovery-board")]
            let led_blue = LedBlue::new(LedParameters {
                pin: gpioc.pc8,
                cr: &mut gpioc.crh,
            });
            #[cfg(any(feature = "nucleo-f446re-board", feature = "stm32vldiscovery-board"))]
            #[allow(clippy::let_unit_value)]
            let led_red = LedRed::default();
            #[cfg(feature = "nucleo-f767zi-board")]
            let led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

            // Initialize UART for serial communication through USB Debug port
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
            #[cfg(feature = "stm32vldiscovery-board")]
            let serial_param = SerialParameters {
                uart: dp.USART1,
                clocks: &clocks,
                pin_tx: gpioa.pa9,
                pin_rx: gpioa.pa10,
                afio: &mut afio,
                cr: &mut gpioa.crh,
            };
            let mut serial = SerialUartUsb::new(serial_param);
            serial.println("Hello RTIC!");

            // Initialize USB
            #[cfg(feature = "nucleo-f767zi-board")]
            let usb_param = UsbParameters {
                global: dp.OTG_FS_GLOBAL,
                device: dp.OTG_FS_DEVICE,
                pwrclk: dp.OTG_FS_PWRCLK,
                clocks: &clocks,
                pin_dm: gpioa.pa11,
                pin_dp: gpioa.pa12,
            };
            #[cfg(feature = "nucleo-f767zi-board")]
            let usb = Usb::new(usb_param);
            #[cfg(any(feature = "nucleo-f446re-board", feature = "stm32vldiscovery-board"))]
            #[allow(clippy::let_unit_value)]
            let usb = Usb::default();

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
            #[cfg(feature = "stm32vldiscovery-board")]
            let button_param = ButtonParameters {
                pin: gpioa.pa0,
                edge: Edge::Falling,
                exti: &mut exti,
                afio: &mut afio,
                cr: &mut gpioa.crl,
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
            #[cfg(feature = "stm32vldiscovery-board")]
            let event_pin_gdo_2 = EventPinParameters {
                pin: gpiob.pb5,
                edge: Edge::Falling,
                afio: &mut afio,
                exti: &mut exti,
            };
            let cc1101_int = EventPinCc1101Gdo2::new(event_pin_gdo_2);

            // Initialize Spi
            #[cfg(feature = "nucleo-f767zi-board")]
            let spi_param = SpiParameters {
                spi: dp.SPI3,
                clocks: &clocks,
                freq: FreqSize::kHz(250),
                apb: &mut rcc.apb1,
                pin_cs: gpioc.pc9,
                pin_sck: gpioc.pc10,
                pin_miso: gpioc.pc11,
                pin_mosi: gpioc.pc12,
            };
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
            #[cfg(feature = "stm32vldiscovery-board")]
            let spi_param = SpiParameters {
                spi: dp.SPI1,
                clocks: &clocks,
                afio: &mut afio,
                freq: FreqSize::kHz(250),
                pin_cs: gpioa.pa4,
                pin_sck: gpioa.pa5,
                pin_miso: gpioa.pa6,
                pin_mosi: gpioa.pa7,
                cr: &mut gpioa.crl,
            };
            let spi_cc1101 = SpiCc1101::new(spi_param);

            // Initialize CC1101 Wrapper - RF Transceiver
            let cc1101_wrp = Cc1101Wrapper::new(spi_cc1101);

            // Spawn tasks
            task_1ms::spawn().ok();
            task_10ms::spawn().ok();
            task_rf_com::spawn().ok();

            // Return
            (
                Shared {
                    button,
                    button_int_signal: false,
                    cc1101_int,
                    cc1101_int_signal: false,
                    led_green,
                    led_blue,
                    led_red,
                    serial,
                },
                Local { cc1101_wrp, usb },
            )
        }

        #[allow(unused_variables, unused_mut)]
        #[allow(clippy::let_unit_value)]
        #[task(priority = 1, local = [usb])]
        async fn task_1ms(mut ctx: task_1ms::Context) {
            loop {
                let mut instant = SysTime::now();
                instant += TimeSize::millis(1);

                let _task_1ms = {
                    let mut usb_buffer = [0u8; 64];

                    // Perform USB packet loopback

                    #[cfg(feature = "nucleo-f767zi-board")]
                    while ctx.local.usb.poll() {
                        // Read data from USB
                        if let Ok(len) = ctx.local.usb.read(&mut usb_buffer) {
                            // Write data to USB
                            let _ = ctx.local.usb.write(&usb_buffer[0..len]);
                        }
                    }
                };

                SysTime::delay_until(instant).await;
            }
        }

        #[allow(unused_variables, unused_mut)]
        #[task(priority = 2, shared = [serial])]
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

        #[task(priority = 3, local = [cc1101_wrp], shared = [button_int_signal, cc1101_int_signal, serial])]
        async fn task_rf_com(mut ctx: task_rf_com::Context) {
            ctx.local.cc1101_wrp.init_config().unwrap();

            SysTime::delay(TimeSize::millis(100)).await;

            loop {
                // RF Communication Task
                {
                    let mut button_int_flag = false;
                    let mut cc1101_int_flag = false;
                    let mut data_rx: [u8; PACKET_LENGTH as usize] = [0; PACKET_LENGTH as usize];
                    let mut data_tx: [u8; PACKET_LENGTH as usize] = [0; PACKET_LENGTH as usize];
                    let mut rssi: i16 = 0;
                    let mut lqi: u8 = 0;

                    // Prepare Tx data
                    for (index, element) in data_tx.iter_mut().enumerate() {
                        *element = index as u8;
                    }

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
                // Idle Task
                {
                    // Do nothing
                };

                rtic::export::wfi();
            }
        }

        fn interrupt_router(mut int_params: InterruptParameters) {
            let mut known_interrupt_event: bool = false;

            // Check if interrupt occured due to Button
            int_params.button.lock(|button| {
                if button.check_interrupt() {
                    let instant = SysTime::now();
                    let debounced: bool =
                        (instant - button.debounce_instant) > button.get_debounce_period();
                    known_interrupt_event = true;

                    // Check if debounce time elapsed
                    if debounced {
                        button.debounce_instant = instant;

                        // Lock shared "button_int_signal" resource. Use it in the critical section
                        int_params.button_int_signal.lock(|signal| {
                            *signal = true;
                        });

                        // Obtain access to LEDs Peripheral and toggle them
                        int_params.led_green.lock(|led_green| {
                            led_green.toggle();
                        });
                        int_params.led_blue.lock(|led_blue| {
                            #[cfg(any(
                                feature = "nucleo-f767zi-board",
                                feature = "stm32vldiscovery-board"
                            ))]
                            led_blue.toggle();
                            #[cfg(feature = "nucleo-f446re-board")]
                            let _ = led_blue;
                        });
                        int_params.led_red.lock(|led_red| {
                            #[cfg(feature = "nucleo-f767zi-board")]
                            led_red.toggle();
                            #[cfg(any(
                                feature = "nucleo-f446re-board",
                                feature = "stm32vldiscovery-board"
                            ))]
                            let _ = led_red;
                        });

                        // Lock shared "serial" resource. Use it in the critical section
                        int_params.serial.lock(|serial| {
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
            int_params.cc1101_int.lock(|cc1101_int| {
                if cc1101_int.check_interrupt() {
                    known_interrupt_event = true;

                    // Lock shared "cc1101_int_signal" resource. Use it in the critical section
                    int_params.cc1101_int_signal.lock(|signal| {
                        *signal = true;
                    });

                    // Lock shared "serial" resource. Use it in the critical section
                    int_params.serial.lock(|serial| {
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
                int_params.serial.lock(|serial| {
                    serial.formatln(format_args!(
                        "[interrupt_router] time: {} (unknown event)",
                        SysTime::now().duration_since_epoch()
                    ));
                });
            }
        }

        #[task(binds = EXTI0, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti0_isr(ctx: exti0_isr::Context) {
            interrupt_router(InterruptParameters {
                button: ctx.shared.button,
                button_int_signal: ctx.shared.button_int_signal,
                cc1101_int: ctx.shared.cc1101_int,
                cc1101_int_signal: ctx.shared.cc1101_int_signal,
                led_green: ctx.shared.led_green,
                led_blue: ctx.shared.led_blue,
                led_red: ctx.shared.led_red,
                serial: ctx.shared.serial,
            });
        }

        #[task(binds = EXTI1, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti1_isr(ctx: exti1_isr::Context) {
            interrupt_router(InterruptParameters {
                button: ctx.shared.button,
                button_int_signal: ctx.shared.button_int_signal,
                cc1101_int: ctx.shared.cc1101_int,
                cc1101_int_signal: ctx.shared.cc1101_int_signal,
                led_green: ctx.shared.led_green,
                led_blue: ctx.shared.led_blue,
                led_red: ctx.shared.led_red,
                serial: ctx.shared.serial,
            });
        }

        #[task(binds = EXTI2, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti2_isr(ctx: exti2_isr::Context) {
            interrupt_router(InterruptParameters {
                button: ctx.shared.button,
                button_int_signal: ctx.shared.button_int_signal,
                cc1101_int: ctx.shared.cc1101_int,
                cc1101_int_signal: ctx.shared.cc1101_int_signal,
                led_green: ctx.shared.led_green,
                led_blue: ctx.shared.led_blue,
                led_red: ctx.shared.led_red,
                serial: ctx.shared.serial,
            });
        }

        #[task(binds = EXTI3, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti3_isr(ctx: exti3_isr::Context) {
            interrupt_router(InterruptParameters {
                button: ctx.shared.button,
                button_int_signal: ctx.shared.button_int_signal,
                cc1101_int: ctx.shared.cc1101_int,
                cc1101_int_signal: ctx.shared.cc1101_int_signal,
                led_green: ctx.shared.led_green,
                led_blue: ctx.shared.led_blue,
                led_red: ctx.shared.led_red,
                serial: ctx.shared.serial,
            });
        }

        #[task(binds = EXTI4, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti4_isr(ctx: exti4_isr::Context) {
            interrupt_router(InterruptParameters {
                button: ctx.shared.button,
                button_int_signal: ctx.shared.button_int_signal,
                cc1101_int: ctx.shared.cc1101_int,
                cc1101_int_signal: ctx.shared.cc1101_int_signal,
                led_green: ctx.shared.led_green,
                led_blue: ctx.shared.led_blue,
                led_red: ctx.shared.led_red,
                serial: ctx.shared.serial,
            });
        }

        #[task(binds = EXTI9_5, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti9_5_isr(ctx: exti9_5_isr::Context) {
            interrupt_router(InterruptParameters {
                button: ctx.shared.button,
                button_int_signal: ctx.shared.button_int_signal,
                cc1101_int: ctx.shared.cc1101_int,
                cc1101_int_signal: ctx.shared.cc1101_int_signal,
                led_green: ctx.shared.led_green,
                led_blue: ctx.shared.led_blue,
                led_red: ctx.shared.led_red,
                serial: ctx.shared.serial,
            });
        }

        #[task(binds = EXTI15_10, shared=[button, button_int_signal, cc1101_int, cc1101_int_signal, led_green, led_blue, led_red, serial])]
        fn exti15_10_isr(ctx: exti15_10_isr::Context) {
            interrupt_router(InterruptParameters {
                button: ctx.shared.button,
                button_int_signal: ctx.shared.button_int_signal,
                cc1101_int: ctx.shared.cc1101_int,
                cc1101_int_signal: ctx.shared.cc1101_int_signal,
                led_green: ctx.shared.led_green,
                led_blue: ctx.shared.led_blue,
                led_red: ctx.shared.led_red,
                serial: ctx.shared.serial,
            });
        }
    }
}
