use embedded_hal::blocking::spi::{Transfer, Write};
use fugit::RateExtU32;
use stm32f7xx_hal::{
    gpio::{Alternate, Output, Pin},
    pac::SPI3,
    rcc::{BusClock, Clocks, Enable, RccBus},
    spi::{self, Enabled, Error, Instance, Miso, Mosi, Sck, Spi},
};

pub struct SpiMaster<
    SPI,
    const P_CS: char,
    const N_CS: u8,
    const P_SCK: char,
    const N_SCK: u8,
    const A_SCK: u8,
    const P_MISO: char,
    const N_MISO: u8,
    const A_MISO: u8,
    const P_MOSI: char,
    const N_MOSI: u8,
    const A_MOSI: u8,
> {
    pub spi: Spi<
        SPI,
        (
            Pin<P_SCK, N_SCK, Alternate<A_SCK>>,
            Pin<P_MISO, N_MISO, Alternate<A_MISO>>,
            Pin<P_MOSI, N_MOSI, Alternate<A_MOSI>>,
        ),
        Enabled<u8>,
    >,
    pub cs: Pin<P_CS, N_CS, Output>,
}

impl<
        SPI,
        const P_CS: char,
        const N_CS: u8,
        const P_SCK: char,
        const N_SCK: u8,
        const A_SCK: u8,
        const P_MISO: char,
        const N_MISO: u8,
        const A_MISO: u8,
        const P_MOSI: char,
        const N_MOSI: u8,
        const A_MOSI: u8,
    >
    SpiMaster<SPI, P_CS, N_CS, P_SCK, N_SCK, A_SCK, P_MISO, N_MISO, A_MISO, P_MOSI, N_MOSI, A_MOSI>
where
    SPI: Instance + Enable + BusClock,
    Pin<P_SCK, N_SCK, Alternate<A_SCK>>: Sck<SPI>,
    Pin<P_MISO, N_MISO, Alternate<A_MISO>>: Miso<SPI>,
    Pin<P_MOSI, N_MOSI, Alternate<A_MOSI>>: Mosi<SPI>,
{
    pub fn new(
        spi: SPI,
        clocks: &Clocks,
        apb: &mut <SPI as RccBus>::Bus,
        pin_cs: Pin<P_CS, N_CS>,
        pin_sck: Pin<P_SCK, N_SCK>,
        pin_miso: Pin<P_MISO, N_MISO>,
        pin_mosi: Pin<P_MOSI, N_MOSI>,
    ) -> Self {
        // Initialize SPI pins
        let mut cs: Pin<P_CS, N_CS, Output> = pin_cs.into_push_pull_output();
        let sck: Pin<P_SCK, N_SCK, Alternate<A_SCK>> = pin_sck.into_alternate();
        let miso: Pin<P_MISO, N_MISO, Alternate<A_MISO>> = pin_miso.into_alternate();
        let mosi: Pin<P_MOSI, N_MOSI, Alternate<A_MOSI>> = pin_mosi.into_alternate();

        // Set nCS pin to high (disabled) initially
        cs.set_high();

        // Initialize SPI
        let spi = Spi::new(spi, (sck, miso, mosi)).enable::<u8>(
            spi::Mode {
                polarity: spi::Polarity::IdleHigh,
                phase: spi::Phase::CaptureOnSecondTransition,
            },
            250_u32.kHz(),
            clocks,
            apb,
        );

        Self { spi, cs }
    }

    /// Sends bytes to the slave chip
    pub fn write(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.spi.write(buffer)
    }

    /// Sends bytes to the slave chip. Returns the bytes received from the slave chip
    pub fn transfer<'a>(&'a mut self, buffer: &'a mut [u8]) -> Result<&[u8], Error> {
        self.spi.transfer(buffer)
    }

    /// Sets the CS pin low
    pub fn cs_set_low(&mut self) {
        self.cs.set_low();
    }

    /// Sets the CS pin high
    pub fn cs_set_high(&mut self) {
        self.cs.set_high();
    }
}

pub type SpiMaster3 = SpiMaster<SPI3, 'C', 9, 'C', 10, 6, 'C', 11, 6, 'C', 12, 6>;
