#![no_std]

// use core::convert::Infallible;
use cc1101::{
    AddressFilter, Cc1101, CcaMode, Error, Modulation, NumPreambleBytes, PacketLength, RadioMode,
    SyncMode,
};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
// use stm32f7xx_hal::spi::Error as SpiError;

// type GpioError = Infallible;
// type GpioError = i32;
// pub type Cc1101Error = Error<SpiError, GpioError>;

// Project specific radio configurations
const FREQUENCY: u64 = 433_000_000; // 433 MHz
const MODULATION: Modulation = Modulation::BinaryFrequencyShiftKeying;
const SYNC_MODE: SyncMode = SyncMode::MatchFull(0xD201);
const PACKET_LENGTH: PacketLength = PacketLength::Fixed(8);
const NUM_PREAMBLE_BYTES: NumPreambleBytes = NumPreambleBytes::Two;
const CCA_MODE: CcaMode = CcaMode::AlwaysClear;
const CRC_ENABLE: bool = true;
const ADDRESS_FILTER: AddressFilter = AddressFilter::Device(0x3e);
const DEVIATION: u64 = 20_629;
const DATARATE: u64 = 38_383;
const BANDWIDTH: u64 = 101_562;

pub struct Cc1101Wrapper<SPI, CS> {
    cc1101: Cc1101<SPI, CS>,
}

impl<SPI, CS, SpiE, GpioE> Cc1101Wrapper<SPI, CS>
where
    SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    CS: OutputPin<Error = GpioE>,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        let cc1101 = Cc1101::new(spi, cs);

        match cc1101 {
            Ok(cc1101) => Cc1101Wrapper { cc1101 },
            Err(_error) => panic!("Error initializing CC1101"),
        }
    }

    pub fn configure_radio(&mut self) -> Result<(), Error<SpiE, GpioE>> {
        self.cc1101.set_defaults()?;
        self.cc1101.set_frequency(FREQUENCY)?;
        self.cc1101.set_modulation(MODULATION)?;
        self.cc1101.set_sync_mode(SYNC_MODE)?;
        self.cc1101.set_cca_mode(CCA_MODE)?;
        self.cc1101.set_num_preamble(NUM_PREAMBLE_BYTES)?;
        self.cc1101.crc_enable(CRC_ENABLE)?;
        self.cc1101.set_packet_length(PACKET_LENGTH)?;
        self.cc1101.set_address_filter(ADDRESS_FILTER)?;
        self.cc1101.set_deviation(DEVIATION)?;
        self.cc1101.set_data_rate(DATARATE)?;
        self.cc1101.set_chanbw(BANDWIDTH)?;

        Ok(())
    }

    fn delay(&mut self, time_us: u32) {
        for _ in 0..time_us {
            cortex_m::asm::nop();
        }
    }

    pub fn receive_packet(
        &mut self,
        dst: &mut u8,
        buffer: &mut [u8],
    ) -> Result<(u8, i16, u8), Error<SpiE, GpioE>> {
        self.cc1101.set_radio_mode(RadioMode::Receive)?;

        //  wait 10 ms
        self.delay(10 * 1000);

        let length = self.cc1101.receive(dst, buffer)?;
        let rssi = self.cc1101.get_rssi_dbm()?;
        let lqi = self.cc1101.get_lqi()?;

        Ok((length, rssi, lqi))
    }

    pub fn transmit_packet(
        &mut self,
        dst: &mut u8,
        buffer: &mut [u8],
    ) -> Result<(), Error<SpiE, GpioE>> {
        self.cc1101.set_radio_mode(RadioMode::Transmit)?;
        self.cc1101.transmit(dst, buffer)?;

        Ok(())
    }

    pub fn get_hw_info(&mut self) -> Result<(u8, u8), Error<SpiE, GpioE>> {
        self.cc1101.get_hw_info()
    }
}
