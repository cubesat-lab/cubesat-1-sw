use embedded_hal::{
    blocking::spi::{Transfer, Write},
    spi::FullDuplex,
};
use embedded_hal_1::spi::{Error, ErrorKind, ErrorType, Operation, SpiDevice};
use stm32f7xx_hal::{
    gpio::{Alternate, Output, Pin},
    pac::{SPI3, SPI4},
    rcc::{BusClock, Clocks, Enable, RccBus},
    spi::{
        Enabled, Error as Stm32F7SpiError, Instance, Miso, Mode, Mosi, Phase, Polarity, Sck, Spi,
    },
};
use sys_time::prelude::*;

#[derive(Debug)]
pub enum SpiError {
    Overrun,
    ModeFault,
    FrameFormat,
    ChipSelectFault,
    Other,
}

impl From<Stm32F7SpiError> for SpiError {
    fn from(value: Stm32F7SpiError) -> Self {
        match value {
            Stm32F7SpiError::FrameFormat => Self::FrameFormat,
            Stm32F7SpiError::Overrun => Self::Overrun,
            Stm32F7SpiError::ModeFault => Self::ModeFault,
        }
    }
}

impl Error for SpiError {
    fn kind(&self) -> ErrorKind {
        match *self {
            SpiError::Overrun => ErrorKind::Overrun,
            SpiError::ModeFault => ErrorKind::ModeFault,
            SpiError::FrameFormat => ErrorKind::FrameFormat,
            SpiError::ChipSelectFault => ErrorKind::ChipSelectFault,
            SpiError::Other => ErrorKind::Other,
        }
    }
}

pub struct SpiParameters<
    'a,
    SPI: RccBus,
    const CS_PORT: char,
    const CS_NUM: u8,
    const SCK_PORT: char,
    const SCK_NUM: u8,
    const SCK_ALT: u8,
    const MISO_PORT: char,
    const MISO_NUM: u8,
    const MISO_ALT: u8,
    const MOSI_PORT: char,
    const MOSI_NUM: u8,
    const MOSI_ALT: u8,
> {
    pub spi: SPI,
    pub clocks: &'a Clocks,
    pub freq: FreqSize,
    pub apb: &'a mut <SPI as RccBus>::Bus,
    pub pin_cs: Pin<CS_PORT, CS_NUM>,
    pub pin_sck: Pin<SCK_PORT, SCK_NUM>,
    pub pin_miso: Pin<MISO_PORT, MISO_NUM>,
    pub pin_mosi: Pin<MOSI_PORT, MOSI_NUM>,
}

type PinSck<const SCK_PORT: char, const SCK_NUM: u8, const SCK_ALT: u8> =
    Pin<SCK_PORT, SCK_NUM, Alternate<SCK_ALT>>;

type PinMiso<const MISO_PORT: char, const MISO_NUM: u8, const MISO_ALT: u8> =
    Pin<MISO_PORT, MISO_NUM, Alternate<MISO_ALT>>;

type PinMosi<const MOSI_PORT: char, const MOSI_NUM: u8, const MOSI_ALT: u8> =
    Pin<MOSI_PORT, MOSI_NUM, Alternate<MOSI_ALT>>;

type PinCs<const CS_PORT: char, const CS_NUM: u8> = Pin<CS_PORT, CS_NUM, Output>;

pub struct SpiMaster<
    SPI,
    const CS_PORT: char,
    const CS_NUM: u8,
    const SCK_PORT: char,
    const SCK_NUM: u8,
    const SCK_ALT: u8,
    const MISO_PORT: char,
    const MISO_NUM: u8,
    const MISO_ALT: u8,
    const MOSI_PORT: char,
    const MOSI_NUM: u8,
    const MOSI_ALT: u8,
> {
    pub spi: Spi<
        SPI,
        (
            PinSck<SCK_PORT, SCK_NUM, SCK_ALT>,
            PinMiso<MISO_PORT, MISO_NUM, MISO_ALT>,
            PinMosi<MOSI_PORT, MOSI_NUM, MOSI_ALT>,
        ),
        Enabled<u8>,
    >,
    pub cs: PinCs<CS_PORT, CS_NUM>,
}

impl<
        SPI,
        const CS_PORT: char,
        const CS_NUM: u8,
        const SCK_PORT: char,
        const SCK_NUM: u8,
        const SCK_ALT: u8,
        const MISO_PORT: char,
        const MISO_NUM: u8,
        const MISO_ALT: u8,
        const MOSI_PORT: char,
        const MOSI_NUM: u8,
        const MOSI_ALT: u8,
    > ErrorType
    for SpiMaster<
        SPI,
        CS_PORT,
        CS_NUM,
        SCK_PORT,
        SCK_NUM,
        SCK_ALT,
        MISO_PORT,
        MISO_NUM,
        MISO_ALT,
        MOSI_PORT,
        MOSI_NUM,
        MOSI_ALT,
    >
{
    type Error = SpiError;
}

impl<
        SPI,
        const CS_PORT: char,
        const CS_NUM: u8,
        const SCK_PORT: char,
        const SCK_NUM: u8,
        const SCK_ALT: u8,
        const MISO_PORT: char,
        const MISO_NUM: u8,
        const MISO_ALT: u8,
        const MOSI_PORT: char,
        const MOSI_NUM: u8,
        const MOSI_ALT: u8,
    >
    SpiMaster<
        SPI,
        CS_PORT,
        CS_NUM,
        SCK_PORT,
        SCK_NUM,
        SCK_ALT,
        MISO_PORT,
        MISO_NUM,
        MISO_ALT,
        MOSI_PORT,
        MOSI_NUM,
        MOSI_ALT,
    >
where
    SPI: Instance + Enable + BusClock,
    PinSck<SCK_PORT, SCK_NUM, SCK_ALT>: Sck<SPI>,
    PinMiso<MISO_PORT, MISO_NUM, MISO_ALT>: Miso<SPI>,
    PinMosi<MOSI_PORT, MOSI_NUM, MOSI_ALT>: Mosi<SPI>,
{
    pub fn new(
        spi_parameters: SpiParameters<
            SPI,
            CS_PORT,
            CS_NUM,
            SCK_PORT,
            SCK_NUM,
            SCK_ALT,
            MISO_PORT,
            MISO_NUM,
            MISO_ALT,
            MOSI_PORT,
            MOSI_NUM,
            MOSI_ALT,
        >,
    ) -> Self {
        let mut cs = spi_parameters.pin_cs.into_push_pull_output();

        // Set nCS pin to high (disabled) initially
        cs.set_high();

        // Initialize SPI
        let spi = Spi::new(
            spi_parameters.spi,
            (
                spi_parameters.pin_sck.into_alternate(),
                spi_parameters.pin_miso.into_alternate(),
                spi_parameters.pin_mosi.into_alternate(),
            ),
        )
        .enable::<u8>(
            Mode {
                polarity: Polarity::IdleHigh,
                phase: Phase::CaptureOnSecondTransition,
            },
            spi_parameters.freq,
            spi_parameters.clocks,
            spi_parameters.apb,
        );

        Self { spi, cs }
    }

    fn _transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        // TODO: Use internal buffer to copy data in between transfer
        let _ = read;
        let _ = write;
        Ok(())
    }
}

impl<
        SPI,
        const CS_PORT: char,
        const CS_NUM: u8,
        const SCK_PORT: char,
        const SCK_NUM: u8,
        const SCK_ALT: u8,
        const MISO_PORT: char,
        const MISO_NUM: u8,
        const MISO_ALT: u8,
        const MOSI_PORT: char,
        const MOSI_NUM: u8,
        const MOSI_ALT: u8,
    > SpiDevice
    for SpiMaster<
        SPI,
        CS_PORT,
        CS_NUM,
        SCK_PORT,
        SCK_NUM,
        SCK_ALT,
        MISO_PORT,
        MISO_NUM,
        MISO_ALT,
        MOSI_PORT,
        MOSI_NUM,
        MOSI_ALT,
    >
where
    SPI: Instance + Enable + BusClock,
    PinSck<SCK_PORT, SCK_NUM, SCK_ALT>: Sck<SPI>,
    PinMiso<MISO_PORT, MISO_NUM, MISO_ALT>: Miso<SPI>,
    PinMosi<MOSI_PORT, MOSI_NUM, MOSI_ALT>: Mosi<SPI>,
{
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        self.cs.set_low();
        for op in operations {
            match op {
                Operation::Read(buf) => {
                    // TODO
                    let _ = buf;
                    let _ = self.spi.read();
                    todo!();
                }
                Operation::Write(buf) => self.spi.write(buf)?,
                Operation::Transfer(read, write) => self._transfer(read, write)?,
                Operation::TransferInPlace(buf) => {
                    self.spi.transfer(buf)?;
                }
                Operation::DelayNs(ns) => {
                    // TODO: Check how to use monotonic delay for embedded_hal DelayNs
                    // SysTime::delay(TimeSize::nanos(ns)).await;
                    let _ = ns;
                    todo!()
                }
            }
        }
        self.cs.set_high();
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        // TODO
        self.cs.set_low();
        let _ = buf;
        let _ = self.spi.read();
        self.cs.set_high();
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        self.spi.write(buf)?;
        self.cs.set_high();
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        self._transfer(read, write)?;
        self.cs.set_high();
        Ok(())
    }

    fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        self.spi.transfer(buf)?;
        self.cs.set_high();
        Ok(())
    }
}

pub type SpiMaster3 = SpiMaster<SPI3, 'C', 9, 'C', 10, 6, 'C', 11, 6, 'C', 12, 6>;
pub type SpiMaster4 = SpiMaster<SPI4, 'E', 4, 'E', 2, 5, 'E', 5, 5, 'E', 6, 5>;
