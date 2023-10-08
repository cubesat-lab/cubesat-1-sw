#![no_main]
#![no_std]

mod bsp;
mod cc1101_wrapper;

use crate::cc1101_wrapper::{Cc1101Wrapper, Cs, SpiStruct};
use bsp::Bsp;
use cc1101::{AddressFilter, CcaMode, Modulation, NumPreambleBytes, PacketLength, SyncMode};
use cortex_m_rt::entry;
use embedded_hal::blocking::spi::{Transfer, Write};
use panic_halt as _;
use stm32f7xx_hal::{
    gpio::{Alternate, Output, Pin},
    pac::SPI3,
    spi::{Enabled, Error as SpiError, Spi},
};

#[allow(dead_code)]
enum RfDevice {
    Transmitter,
    Listener,
}

static USE_GDB: bool = false;
static RF_DEVICE: RfDevice = RfDevice::Listener;

pub fn pin_set_low(pin: &mut Pin<'C', 9, Output>) -> Result<(), i32> {
    pin.set_low();

    Ok(())
}

pub fn pin_set_high(pin: &mut Pin<'C', 9, Output>) -> Result<(), i32> {
    pin.set_high();

    Ok(())
}

fn delay(time_us: u32) {
    for _ in 0..time_us {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    // Initialization part
    let mut bsp_obj: Bsp = Bsp::init(USE_GDB);

    // Start part
    bsp_obj.start();

    // Initialize CC1101 Wrapper
    type PIN = Pin<'C', 9, Output>;
    type SPI = Spi<
        SPI3,
        (
            Pin<'C', 10, Alternate<6>>,
            Pin<'C', 11, Alternate<6>>,
            Pin<'C', 12, Alternate<6>>,
        ),
        Enabled<u8>,
    >;

    type CsWrapper<'a> = Cs<'a, PIN, GpioError>;
    type SpiWrapper<'a, 'b> = SpiStruct<'a, SPI, SpiError>;
    type GpioError = i32;

    let (spi_ref, cs_ref) = bsp_obj.get_spi_cs();

    let cs_wrp = CsWrapper::new(cs_ref, pin_set_low, pin_set_high);
    let spi_wrp = SpiWrapper::new(spi_ref, SPI::transfer, SPI::write);

    let freq: u64 = 433_000_000u64; // 433 MHz
    let modulation = Modulation::BinaryFrequencyShiftKeying;
    let sync_mode = SyncMode::MatchFull(0xD201);
    let packet_length = PacketLength::Fixed(32);
    let num_preamble = NumPreambleBytes::Two;
    let cca_mode = CcaMode::AlwaysClear;
    let crc_enable = true;
    let address_filter = AddressFilter::Device(0x3e);
    let deviation = 20_629;
    let baud = 38_383;
    let bandwidth = 101_562;

    let mut cc1101_object: Cc1101Wrapper<'_, '_, SPI, SpiError, GpioError, PIN> =
        Cc1101Wrapper::new(spi_wrp, cs_wrp);
    let _ = cc1101_object
        .configure_radio(
            freq,
            modulation,
            sync_mode,
            cca_mode,
            packet_length,
            num_preamble,
            crc_enable,
            address_filter,
            deviation,
            baud,
            bandwidth,
        )
        .unwrap();

    delay(1_000_000);

    // Cyclic part
    loop {
        // Test Code
        match RF_DEVICE {
            RfDevice::Transmitter => {
                // Transmit the packet and delay for 1s
                delay(1_000_000);
                let mut dst = 0u8;
                let mut buffer = [0u8; 15];
                let _result = cc1101_object
                    .transmit_packet(&mut dst, &mut buffer)
                    .unwrap();
            }

            RfDevice::Listener => {
                // Attempt to read data on the radio
                // If read is succesful, send the packet via UART
                let mut dst = 0u8;
                let mut buffer = [0u8; 17];
                if let Ok(_result) = cc1101_object.receive_packet(&mut dst, &mut buffer) {
                    let _x = 0;
                    // bsp_obj.formatln(format_args!("Message: {}", x));
                }
            }
        };
    }
}
