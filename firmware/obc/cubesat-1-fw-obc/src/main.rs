#![no_main]
#![no_std]

mod bsp;
mod cc1101_wrapper;

use bsp::Bsp;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f7xx_hal:: {
    gpio::{Alternate, Output, Pin},
    pac::SPI3,
    spi::{Enabled, Spi, Error as SpiError},
};
use crate::cc1101_wrapper::{Cc1101Wrapper, Cs, SpiStruct};
use cc1101::{Modulation, AddressFilter, SyncMode, PacketLength};
use embedded_hal::blocking::spi::{Transfer, Write};

static USE_GDB: bool = false;

#[entry]
fn main() -> ! {
    // Initialization part
    let mut bsp_obj = Bsp::init(USE_GDB);

    // Start part
    bsp_obj.start();

    // Initialize CC1101Wrapper
    type PIN =  Pin<'C', 9, Output>;
    type SPI = Spi<SPI3, (Pin<'C', 10, Alternate<6>>, Pin<'C', 11, Alternate<6>>, Pin<'C', 12, Alternate<6>>), Enabled<u8>>;

    type CsWrapper<'a> = Cs<'a, PIN>;
    type SpiWrapper<'a, 'b> = SpiStruct<'a, SPI, SpiError>;
    type GpioError = i32;

    let cs_wrp = CsWrapper::new(&mut bsp_obj.spi.cs, PIN::set_low, PIN::set_high);
    let spi_wrp = SpiWrapper::new(&mut bsp_obj.spi.spi, SPI::transfer, SPI::write);

    let freq: u64 = 868_000_000u64;
    let modulation = Modulation::BinaryFrequencyShiftKeying;
    let sync_mode = SyncMode::MatchFull(0xD201);
    let packet_length = PacketLength::Variable(17);
    let address_filter = AddressFilter::Device(0x3e);

    let mut cc1101_object: Cc1101Wrapper<'_, '_, SPI, SpiError, GpioError, PIN> = Cc1101Wrapper::new(spi_wrp, cs_wrp);
    let _ = cc1101_object.configure_radio(freq, modulation, sync_mode, packet_length, address_filter).unwrap();

    // Cyclic part
    loop {
        // User Code
    }
}
