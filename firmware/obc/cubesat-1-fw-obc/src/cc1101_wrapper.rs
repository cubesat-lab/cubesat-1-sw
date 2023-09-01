extern crate cc1101;

// use bsp::Bsp;
use panic_halt as _;

use embedded_hal::digital::OutputPin;
use cc1101::{AddressFilter, Cc1101, Modulation, PacketLength, RadioMode, SyncMode, Error};
use embedded_hal::blocking::spi::{Transfer, Write};



pub struct Cs<'a, CsPin> {
    cs_instance: &'a mut CsPin,
    set_low_fn: fn(&mut CsPin),
    set_high_fn: fn(&mut CsPin),
}

#[warn(deprecated)]
impl<'a, CsPin> OutputPin for Cs<'a, CsPin> {
    fn set_low(&mut self){
        (self.set_low_fn)(&mut self.cs_instance);
    }
    fn set_high(&mut self) {
        (self.set_high_fn)(&mut self.cs_instance);
    }
}

impl<'a, CsPin> Cs <'a, CsPin> {
    pub fn new (cs_instance: &'a mut CsPin, set_low_fn: fn(&mut CsPin), set_high_fn: fn(&mut CsPin)) -> Self {
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

impl<'a, SPI, SpiError> Transfer<u8> for SpiStruct<'a,  SPI, SpiError> {
    type Error = SpiError;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        (self.spi_transfer_fn)(self.spi_instance, words)
    }
}

impl<'a, 'w, SPI, SpiError> Write<u8> for SpiStruct<'a,  SPI, SpiError> {
    type Error = SpiError;

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        (self.spi_write_fn)(self.spi_instance, words)
    }
}


impl<'a, SPI, SpiError> SpiStruct<'a, SPI, SpiError> {
    pub fn new (spi_instance: &'a mut SPI, spi_transfer_fn: for<'w> fn(&mut SPI, &'w mut [u8]) -> Result<&'w [u8], SpiError>, 
                spi_write_fn: fn(&mut SPI, &[u8]) -> Result<(), SpiError>) -> Self {
        SpiStruct {
            spi_instance,
            spi_transfer_fn,
            spi_write_fn,
        }
    }
}

#[allow(dead_code)]
pub struct Cc1101Wrapper<'a, 'b, SPI, SpiError, GpioError, CsPin> 
where GpioError: Default {
    temp_gpio_error: GpioError,
    cc1101: Cc1101<SpiStruct<'b, SPI, SpiError>, Cs<'a, CsPin>>,
}

impl<'a, 'b, SPI, SpiError, GpioError, CsPin> Cc1101Wrapper<'a, 'b, SPI, SpiError, GpioError, CsPin> 
where GpioError: Default {
    pub fn new(spi: SpiStruct<'b, SPI, SpiError,>, cs: Cs<'a, CsPin>) -> Self {
        let cc1101 = Cc1101::new(spi, cs);

        match cc1101 {
            Ok(cc1101) => Cc1101Wrapper {
                temp_gpio_error: GpioError::default(),
                cc1101,
            },
            Err(_error) => panic!("Error initializing CC1101"),
        }
    }

    pub fn configure_radio(&mut self, frequency: u64, modulation: Modulation, sync_mode: SyncMode, packet_length: PacketLength,
                             address_filter: AddressFilter) 
                            -> Result<(), Error<SpiError>> {
        self.cc1101.set_defaults()?;
        self.cc1101.set_frequency(frequency)?; // 868_000_000u64
        self.cc1101.set_modulation(modulation)?; // Modulation::BinaryFrequencyShiftKeying
        self.cc1101.set_sync_mode(sync_mode)?; // SyncMode::MatchFull(0xD201)
        self.cc1101.set_packet_length(packet_length)?; // PacketLength::Variable(17)
        self.cc1101.set_address_filter(address_filter)?; // AddressFilter::Device(0x3e)

        Ok(())
    }


    fn receive_packet(cc1101: &mut Cc1101<SpiStruct<'b, SPI, SpiError>, Cs<'a, CsPin>>) -> Result<(), Error<SpiError>> {
        cc1101.set_radio_mode(RadioMode::Receive)?;
    
        //  wait 10 ms

        let mut dst = 0u8;
        let mut payload = [0u8; 17];

        let _length = cc1101.receive(&mut dst, &mut payload)?;
        let _rssi = cc1101.get_rssi_dbm()?;
        let _lqi = cc1101.get_lqi()?;

        Ok(())
    }
}