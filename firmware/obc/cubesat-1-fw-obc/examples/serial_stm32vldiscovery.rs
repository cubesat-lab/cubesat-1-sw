// Serial communication on qemu

#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use cortex_m_semihosting::hprintln;

use cortex_m_rt::entry;
use frame_processing::{frame::pack_frame, frame::process_incoming_frame};
use nb::block;
use nucleo_stm32vldiscovery::serial::{SerialParameters, SerialUartUsb};
use stm32f1xx_hal::{pac, prelude::*};

use panic_halt as _;

#[entry]
fn main() -> ! {
    hprintln!("Hello, stm32vldiscovery!");

    // Initialize Peripheral Access Crate
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let afio = dp.AFIO.constrain();
    let gpioa = dp.GPIOA.split();

    let pin_tx = gpioa.pa9;
    let pin_rx = gpioa.pa10;
    let crh = gpioa.crh;

    let mut serial_param = SerialParameters {
        afio: afio,
        crh: crh,
    };

    // Initialize UART for serial communication through USB
    let mut serial = SerialUartUsb::new(dp.USART1, &clocks, pin_tx, pin_rx, &mut serial_param);

    let mut buffer: [u8; 1024] = [0; 1024];
    let mut answer: [u8; 1024] = [0; 1024];
    let mut buffer_size = 0usize;

    loop {
        let received = block!(serial.read()).unwrap();

        if buffer_size < buffer.len() {
            buffer[buffer_size] = received;
            buffer_size += 1;
        }

        // Process the frame and see if it's valid or not
        let (complete_frame, frame_valid) = process_incoming_frame(&mut buffer, &mut buffer_size);

        if complete_frame {
            if frame_valid {
                // Frame was valid - prepare a message
                let data = [0xCA, 0xFE];
                let frame_len = pack_frame(&data, &mut answer);
                for i in 0..(frame_len) {
                    serial.write(answer[i]).unwrap();
                }
            } else {
                // Frame was not valid - prepare a message
                let data = [0xFF, 0xFF];
                let frame_len = pack_frame(&data, &mut answer);
                for i in 0..(frame_len) {
                    serial.write(answer[i]).unwrap();
                }
            }
        }
    }
}
