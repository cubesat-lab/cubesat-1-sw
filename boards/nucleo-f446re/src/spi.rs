use embedded_hal::spi::{Error, ErrorKind, ErrorType, Operation, SpiDevice};
use stm32f4xx_hal::{
    gpio::{alt::SpiCommon, Alternate, Output, Pin},
    // TODO: Make this module generic
    // pac::SPI2,
    rcc::Clocks,
    spi::{Error as Stm32F4SpiError, Instance, Mode, Phase, Polarity, Spi},
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

impl From<Stm32F4SpiError> for SpiError {
    fn from(value: Stm32F4SpiError) -> Self {
        match value {
            Stm32F4SpiError::Overrun => Self::Overrun,
            Stm32F4SpiError::ModeFault => Self::ModeFault,
            Stm32F4SpiError::Crc => Self::Other,
            _ => todo!(),
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

pub type CS = Pin<'B', 4, Output>;

pub struct SpiParameters<'a, SPI> {
    pub spi: SPI,
    pub clocks: &'a Clocks,
    pub freq: FreqSize,
    pub pin_cs: Pin<'B', 4, Alternate<0>>,
    pub pin_sck: Pin<'B', 13>,
    pub pin_miso: Pin<'B', 14>,
    pub pin_mosi: Pin<'B', 15>,
}

pub struct SpiMaster<SPI: Instance> {
    pub spi: Spi<SPI>,
    pub cs: CS,
}

impl<SPI: Instance> ErrorType for SpiMaster<SPI> {
    type Error = SpiError;
}

impl<SPI: Instance> SpiMaster<SPI> {
    pub fn new(spi_parameters: SpiParameters<SPI>) -> Self
    where
        <SPI as SpiCommon>::Sck: From<Pin<'B', 13, Alternate<5>>>,
        <SPI as SpiCommon>::Miso: From<Pin<'B', 14, Alternate<5>>>,
        <SPI as SpiCommon>::Mosi: From<Pin<'B', 15, Alternate<5>>>,
    {
        let cs = spi_parameters.pin_cs.into_push_pull_output();

        // Initialize SPI
        let spi = Spi::new(
            spi_parameters.spi,
            (
                spi_parameters.pin_sck.into_alternate::<5>(),
                spi_parameters.pin_miso.into_alternate::<5>(),
                spi_parameters.pin_mosi.into_alternate::<5>(),
            ),
            Mode {
                polarity: Polarity::IdleHigh,
                phase: Phase::CaptureOnSecondTransition,
            },
            spi_parameters.freq,
            spi_parameters.clocks,
        );

        Self { spi, cs }
    }
}

impl<SPI: Instance> SpiDevice for SpiMaster<SPI> {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        self.cs.set_low();
        for op in operations {
            match op {
                Operation::Read(buf) => {
                    self.spi.read(buf)?;
                }
                Operation::Write(buf) => {
                    self.spi.write(buf)?;
                }
                Operation::Transfer(read, write) => {
                    self.spi.transfer(read, write)?;
                }
                Operation::TransferInPlace(buf) => {
                    self.spi.transfer_in_place(buf)?;
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
        self.cs.set_low();
        self.spi.read(buf)?;
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
        self.spi.transfer(read, write)?;
        self.cs.set_high();
        Ok(())
    }

    fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        self.spi.transfer_in_place(buf)?;
        self.cs.set_high();
        Ok(())
    }
}

// TODO: Make this module generic
// pub type SpiMaster2 = SpiMaster<SPI2, 'B', 4, 'B', 13, 5, 'B', 14, 5, 'B', 15, 5>;
