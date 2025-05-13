#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

// WIP file for RF testing
// TODO:
// - Rewrite this code to align with the main.rs file
// - Implement the packet handler
// - Implement the logic to send and receive packets via RF
// - Implement the logic to send and receive packets via USB / Serial

use fugit::{Duration, HertzU32, Instant};
use panic_halt as _;
use rtic::app;
use rtic_monotonics::{systick::Systick, Monotonic, TimeoutError};
use nb::block;

mod nucleo_f767zi_board {
    use super::*;
    use cc1101_wrapper::{Cc1101Wrapper, PACKET_LENGTH};
    use embedded_hal::digital::v2::PinState;
    use nucleo_f767zi::{
        button::{Button, ButtonParameters},
        event_pin::{EventPinCc1101Gdo2, EventPinParameters},
        led::{LedBlue, LedGreen, LedParameters, LedRed},
        serial::{SerialParameters, SerialUartUsb, SerialEvent},
        spi::SpiMaster3,
        spi_adapter::SpiAdapter,
    };
    use stm32f7xx_hal::{gpio::Edge, pac, prelude::*, interrupt};
    use packet_handler::Protocol;

    #[app(device = pac, dispatchers = [TIM2, TIM3])]
    mod app {
        use packet_handler::State;

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
            led_green: LedGreen,
            led_blue: LedBlue,
            led_red: LedRed,
            button_int_signal: bool,
            cc1101_int_signal: bool,
            serial_to_rf_signal: bool,
            rf_to_serial_signal: bool,
            data_serial_to_rf: [u8; PACKET_LENGTH as usize],
            data_rf_to_serial: [u8; PACKET_LENGTH as usize],
        }

        #[local]
        struct Local {
            button: Button,
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

            // Initialize systick
            let sysclk = (216.MHz() as HertzU32).to_Hz();
            let systick_token = rtic_monotonics::create_systick_token!();
            Systick::start(cp.SYST, sysclk, systick_token);

            // Initialize LEDs
            let led_green = LedGreen::new(LedParameters { pin: gpiob.pb0 });
            let led_blue = LedBlue::new(LedParameters { pin: gpiob.pb7 });
            let led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

            // Initialize UART for serial communication through USB
            let serial = SerialUartUsb::new(SerialParameters {
                uart: dp.USART3,
                clocks: &clocks,
                pin_tx: gpiod.pd8,
                pin_rx: gpiod.pd9,
                interrupt: Some(SerialEvent::Rxne),
            });
            // rtic::pend(interrupt::USART3);

            // serial.println("Hello RTIC!");

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
                debounce_period: fugit::ExtU64::millis(150),
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
            task_serial_com::spawn().ok();
            task_rf_com::spawn().ok();

            // Return
            (
                Shared {
                    serial,
                    led_green,
                    led_blue,
                    led_red,
                    button_int_signal: false,
                    cc1101_int_signal: false,
                    serial_to_rf_signal: false,
                    rf_to_serial_signal: false,
                    data_serial_to_rf: [0; PACKET_LENGTH as usize],
                    data_rf_to_serial: [0; PACKET_LENGTH as usize],
                },
                Local {
                    button,
                    cc1101_int,
                    cc1101_wrp,
                },
            )
        }

        async fn await_serial_read(
            serial: &mut SerialUartUsb,
            timeout: Duration<u64, 1, 1000>,
        ) -> Result<u8, TimeoutError> {
            Systick::timeout_after(timeout, serial_read(serial)).await
        }

        async fn serial_read(serial: &mut SerialUartUsb) -> u8 {
            loop {
                match serial.read() {
                    Ok(byte) => {
                        return byte;
                    }
                    Err(_) => {
                        // Do nothing
                    }
                }

                // Systick::delay(fugit::ExtU64::micros(1000)).await;
            }
        }

        // #[allow(unused_variables, unused_mut)]
        #[task(priority = 1, shared = [serial, led_green, led_blue, led_red])]
        async fn task_serial_com(mut ctx: task_serial_com::Context) {
            Systick::delay(100.millis().into()).await;

            loop {
                // let mut instant = Systick::now();
                // instant += 10.millis();

                let _task_serial_com = {
                    // Read serial with timeout
                    // match await_serial_read(ctx.local.serial, fugit::ExtU64::micros(20000)).await {
                    // match ctx.local.serial.read() {
                    //     Ok(byte) => {
                    //         ctx.local.led_red.set_state(PinState::Low);

                    //         // ctx.local.led_blue.set_state(PinState::High);

                    //         ctx.local.led_green.set_state(PinState::High);

                    //         if byte == Protocol::PKT_REQ as u8 {
                    //             ctx.local.led_blue.toggle();
                    //             ctx.local.serial.write(Protocol::PKT_REQ_ACK as u8);
                    //         }

                    //         // ctx.local.led_blue.toggle();
                    //         ctx.local.serial.write(byte);

                    //         // if byte == 0xC3 {
                    //         //     ctx.local.led_blue.toggle();
                    //         // }
                    //     }
                    //     Err(e) => {
                    //         ctx.local.led_red.set_state(PinState::High);

                    //         ctx.local.led_green.set_state(PinState::Low);
                    //     }
                    // }

                    // ctx.local.serial.write(0x39);

                    // Systick::delay(10.millis().into()).await;


                    // let received = block!(ctx.local.serial.read()).unwrap_or(0);
                    // if received != 0 {
                    //     block!(ctx.local.serial.write(received)).ok();
                    // }


                    // Lock shared "serial" resource. Use it in the critical section
                    // ctx.shared.serial.lock(|serial| {
                    //     serial.formatln(format_args!(
                    //         "[task_10ms] time: {}",
                    //         Systick::now().duration_since_epoch()
                    //     ));
                    // });
                };

                // Systick::delay_until(instant).await;
            }
        }

        #[task(priority = 1, local = [cc1101_wrp], shared = [button_int_signal, cc1101_int_signal])]
        async fn task_rf_com(mut ctx: task_rf_com::Context) {
            ctx.local.cc1101_wrp.init_config().unwrap();

            Systick::delay(100.millis().into()).await;

            loop {
                let _task_rf_com = {
                    let mut button_int_flag = false;
                    let mut cc1101_int_flag = false;
                    let mut data_rx: [u8; PACKET_LENGTH as usize] = [0; PACKET_LENGTH as usize];
                    let mut data_tx: [u8; PACKET_LENGTH as usize] = [0; PACKET_LENGTH as usize];
                    let mut rssi: i16 = 0;
                    let mut lqi: u8 = 0;

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

                    // Handle Rx interrupt for CC1101
                    if cc1101_int_flag {
                        ctx.local.cc1101_wrp.signal_rx_int();
                    }

                    let data_received_from_serial = false;

                    // TODO: Check if data received from serial
                    if data_received_from_serial {
                        // for (index, element) in data_tx.iter_mut().enumerate() {
                        //     *element = index as u8;
                        // }

                        // TODO: Pass data from serial to cc1101 Tx

                        // Send Tx data
                        let _ = ctx
                            .local
                            .cc1101_wrp
                            .write_data(&data_tx[0..(PACKET_LENGTH as usize)]);
                    }

                    // Process RF
                    ctx.local.cc1101_wrp.main().await;

                    if ctx.local.cc1101_wrp.is_data_received() {
                        ctx.local
                            .cc1101_wrp
                            .read_data(&mut data_rx, &mut rssi, &mut lqi)
                            .unwrap();

                        // Consume Rx data
                        // TODO: send to serial task
                    }

                    // Consume last error
                    let (error_option, error_count) = ctx.local.cc1101_wrp.read_last_error();
                    if let Some(error) = error_option {
                        let _ = error;
                        let _ = error_count;
                    }
                };
            }
        }

        #[idle(shared = [])]
        fn idle(mut _ctx: idle::Context) -> ! {
            loop {
                let _idle = {
                    // Do nothing
                };

                rtic::export::wfi();
            }
        }

        // #[task(binds = EXTI15_10, local = [button, led_green, led_blue, led_red], shared = [button_int_signal])]
        #[task(binds = EXTI15_10, local = [button], shared = [button_int_signal])]
        fn button_isr(mut ctx: button_isr::Context) {
            let instant = Systick::now();
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
                // ctx.local.led_green.toggle();
                // ctx.local.led_blue.toggle();
                // ctx.local.led_red.toggle();
            }

            // Obtain access to Button Peripheral and Clear Interrupt Pending Flag
            ctx.local.button.clear_interrupt_pending_bit();
        }

        #[task(binds = EXTI2, local = [cc1101_int], shared = [cc1101_int_signal])]
        fn cc1101_isr(mut ctx: cc1101_isr::Context) {
            // Lock shared "cc1101_int_signal" resource. Use it in the critical section
            ctx.shared.cc1101_int_signal.lock(|signal| {
                *signal = true;
            });

            // Obtain access to CC1101 Interrupt Pin and Clear Interrupt Pending Flag
            ctx.local.cc1101_int.clear_interrupt_pending_bit();
        }

        #[task(binds = USART3, shared = [serial, led_green, led_blue, led_red])]
        fn usart3_isr(mut ctx: usart3_isr::Context) {

            ctx.shared.serial.lock(|serial| {
                match serial.read() {
                    Ok(byte) => {
                        if byte == Protocol::PKT_REQ as u8 {
                            ctx.shared.led_green.lock(|led_green| {
                                led_green.set_state(PinState::High)
                            });
                        } else {
                            ctx.shared.led_blue.lock(|led_blue| {
                                led_blue.set_state(PinState::High)
                            });
                        }
                    }
                    Err(_) => {
                        ctx.shared.led_red.lock(|led_red| {
                            led_red.set_state(PinState::High)
                        });
                    }
                }
            });
        }
    }
}

mod packet_handler {

    #[allow(non_camel_case_types)]
    #[rustfmt::skip]
    #[repr(u8)]
    pub enum Protocol {
        PKT_REQ     = 0b1100_0011,  // 0xC3
        PKT_REQ_ACK = 0b1100_0110,  // 0xC6
        PKT_ACK     = 0b1100_1100,  // 0xCC
        PKT_SENT_RF = 0b1101_1000,  // 0xD8
        PKT_RCV_RF  = 0b1111_0000,  // 0xF0
    }

    pub enum State {
        AwaitingRequest,
        AwaitingSerialPacket,
        SendingSerialPacket,
    }

    pub struct PacketHandler {
        field: u8,
    }

    impl PacketHandler {
        pub fn new() -> Self {
            Self { field: 0 }
        }

        pub fn another_method() {
            // todo
        }
    }
}
