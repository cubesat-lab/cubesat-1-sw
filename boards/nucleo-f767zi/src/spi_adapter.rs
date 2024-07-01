use core::convert::Infallible;
use embedded_hal_1::spi::{SpiDevice, ErrorType, Error, ErrorKind, Operation};
use embedded_hal::{
    blocking::spi::{Transfer, Write},
    spi::FullDuplex,
    digital::v2::OutputPin
};
use stm32f7xx_hal::spi::Error as SpiError;


#[derive(Debug)]
pub enum SpiAdapterError {
    Overrun,
    ModeFault,
    FrameFormat,
    ChipSelectFault,
    Other,
}

pub struct SpiAdapter<SPI, CS> {
    pub spi: SPI,
    pub cs: CS,
}

impl From<SpiError> for SpiAdapterError {
    fn from(value: SpiError) -> Self {
        match value {
            SpiError::FrameFormat => Self::FrameFormat,
            SpiError::Overrun => Self::Overrun,
            SpiError::ModeFault => Self::ModeFault,
        }
    }
}

impl From<Infallible> for SpiAdapterError {
    fn from(_value: Infallible) -> Self {
        SpiAdapterError::Other
    }
}

impl<SPI, CS> ErrorType for SpiAdapter<SPI, CS> {
    type Error = SpiAdapterError;
}

impl Error for SpiAdapterError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl<SPI, CS> SpiAdapter<SPI, CS> {
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self { spi, cs }
    }

    fn _transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        // TODO: Use internal buffer to copy data in between transfer
        let _ = read;
        let _ = write;
        Ok(())
    }
}

impl<SPI, CS> SpiDevice for SpiAdapter<SPI, CS>
where
    SPI: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError> + FullDuplex<u8>,
    CS: OutputPin<Error = Infallible>,
{
    fn transaction(&mut self, operations: &mut [embedded_hal_1::spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
        self.cs.set_low()?;
        for op in operations {
            match op {
                Operation::Read(buf) => {
                    // TODO
                    let _ = buf;
                    let _ = self.spi.read();
                    todo!();
                },
                Operation::Write(buf) => {
                    self.spi.write(buf)?
                },
                Operation::Transfer(read, write) => {
                    self._transfer(read, write)?
                },
                Operation::TransferInPlace(buf) => {
                    self.spi.transfer(buf)?;
                },
                Operation::DelayNs(ns) => {
                    // TODO
                    let _ = ns;
                    todo!()
                }
            }
        }
        self.cs.set_high()?;
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        // TODO
        self.cs.set_low()?;
        let _ = buf;
        let _ = self.spi.read();
        self.cs.set_high()?;
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low()?;
        self.spi.write(buf)?;
        self.cs.set_high()?;
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low()?;
        self._transfer(read, write)?;
        self.cs.set_high()?;
        Ok(())
    }

    fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low()?;
        self.spi.transfer(buf)?;
        self.cs.set_high()?;
        Ok(())
    }
}
