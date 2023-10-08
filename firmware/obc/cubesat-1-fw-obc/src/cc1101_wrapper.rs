extern crate cc1101;

use panic_halt as _;

use cc1101::{
    AddressFilter, Cc1101, CcaMode, Error, Modulation, NumPreambleBytes, PacketLength, RadioMode,
    SyncMode,
};
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

pub struct Cs<'a, CsPin, Error> {
    cs_instance: &'a mut CsPin,
    set_low_fn: fn(&mut CsPin) -> Result<(), Error>,
    set_high_fn: fn(&mut CsPin) -> Result<(), Error>,
}

impl<'a, CsPin, Error> OutputPin for Cs<'a, CsPin, Error> {
    type Error = Error;

    fn set_low(&mut self) -> Result<(), Error> {
        (self.set_low_fn)(&mut self.cs_instance)?;

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Error> {
        (self.set_high_fn)(&mut self.cs_instance)?;

        Ok(())
    }
}

impl<'a, CsPin, Error> Cs<'a, CsPin, Error> {
    pub fn new(
        cs_instance: &'a mut CsPin,
        set_low_fn: fn(&mut CsPin) -> Result<(), Error>,
        set_high_fn: fn(&mut CsPin) -> Result<(), Error>,
    ) -> Self {
        Cs {
            cs_instance,
            set_low_fn,
            set_high_fn,
        }
    }
}

pub struct SpiStruct<'a, SPI, SpiError> {
    spi_instance: &'a mut SPI,
    spi_transfer_fn: for<'w> fn(&mut SPI, &'w mut [u8]) -> Result<&'w [u8], SpiError>,
    spi_write_fn: fn(&mut SPI, &[u8]) -> Result<(), SpiError>,
}

impl<'a, SPI, SpiError> Transfer<u8> for SpiStruct<'a, SPI, SpiError> {
    type Error = SpiError;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], SpiError> {
        (self.spi_transfer_fn)(self.spi_instance, words)
    }
}

impl<'a, 'w, SPI, SpiError> Write<u8> for SpiStruct<'a, SPI, SpiError> {
    type Error = SpiError;

    fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        (self.spi_write_fn)(self.spi_instance, words)
    }
}

impl<'a, SPI, SpiError> SpiStruct<'a, SPI, SpiError> {
    pub fn new(
        spi_instance: &'a mut SPI,
        spi_transfer_fn: for<'w> fn(&mut SPI, &'w mut [u8]) -> Result<&'w [u8], SpiError>,
        spi_write_fn: fn(&mut SPI, &[u8]) -> Result<(), SpiError>,
    ) -> Self {
        SpiStruct {
            spi_instance,
            spi_transfer_fn,
            spi_write_fn,
        }
    }
}

pub struct Cc1101Wrapper<'a, 'b, SPI, SpiError, GpioError, CsPin> {
    cc1101: Cc1101<SpiStruct<'b, SPI, SpiError>, Cs<'a, CsPin, GpioError>>,
}

impl<'a, 'b, SPI, SpiError, GpioError, CsPin>
    Cc1101Wrapper<'a, 'b, SPI, SpiError, GpioError, CsPin>
{
    pub fn new(spi: SpiStruct<'b, SPI, SpiError>, cs: Cs<'a, CsPin, GpioError>) -> Self {
        let cc1101 = Cc1101::new(spi, cs);

        match cc1101 {
            Ok(cc1101) => Cc1101Wrapper { cc1101 },
            Err(_error) => panic!("Error initializing CC1101"),
        }
    }

    pub fn configure_radio(
        &mut self,
        frequency: u64,
        modulation: Modulation,
        sync_mode: SyncMode,
        cca_mode: CcaMode,
        packet_length: PacketLength,
        num_preamble: NumPreambleBytes,
        crc_enable: bool,
        address_filter: AddressFilter,
        deviation: u64,
        baud: u64,
        bandwidth: u64,
    ) -> Result<(), Error<SpiError, GpioError>> {
        self.cc1101.set_defaults()?;
        self.cc1101.set_frequency(frequency)?;
        self.cc1101.set_modulation(modulation)?;
        self.cc1101.set_sync_mode(sync_mode)?;
        self.cc1101.set_cca_mode(cca_mode)?;
        self.cc1101.set_num_preamble(num_preamble)?;
        self.cc1101.crc_enable(crc_enable)?;
        self.cc1101.set_packet_length(packet_length)?;
        self.cc1101.set_address_filter(address_filter)?;
        self.cc1101.set_deviation(deviation)?;
        self.cc1101.set_data_rate(baud)?;
        self.cc1101.set_chanbw(bandwidth)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_hw_info(&mut self) -> Result<(u8, u8), Error<SpiError, GpioError>> {
        self.cc1101.get_hw_info()
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
    ) -> Result<(u8, i16, u8), Error<SpiError, GpioError>> {
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
    ) -> Result<(), Error<SpiError, GpioError>> {
        self.cc1101.set_radio_mode(RadioMode::Transmit)?;
        self.cc1101.transmit(dst, buffer)?;

        Ok(())
    }
}
