use embedded_hal::blocking::spi::Write;
use embedded_hal_1::spi::{Error, ErrorKind, ErrorType, Operation, SpiDevice};
use stm32f1xx_hal::{
    afio::Parts as AfioParts,
    gpio::{Alternate, Cr, Output, Pin, PushPull},
    pac::SPI1,
    rcc::Clocks,
    spi::{Error as Stm32F1SpiError, Mode, Phase, Polarity, Spi, Spi1NoRemap},
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

impl From<Stm32F1SpiError> for SpiError {
    fn from(value: Stm32F1SpiError) -> Self {
        match value {
            Stm32F1SpiError::Overrun => Self::Overrun,
            Stm32F1SpiError::ModeFault => Self::ModeFault,
            Stm32F1SpiError::Crc => Self::Other,
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

pub struct SpiParameters<'a> {
    pub spi: SPI1,
    pub clocks: &'a Clocks,
    pub afio: &'a mut AfioParts,
    pub freq: FreqSize,
    pub pin_cs: Pin<'A', 4>,
    pub pin_sck: Pin<'A', 5>,
    pub pin_miso: Pin<'A', 6>,
    pub pin_mosi: Pin<'A', 7>,
    pub cr: &'a mut Cr<'A', false>,
}

pub type Spi1Type =
    Spi<SPI1, Spi1NoRemap, (Pin<'A', 5, Alternate>, Pin<'A', 6>, Pin<'A', 7, Alternate>), u8>;

pub type CS = Pin<'A', 4, Output>;

pub struct SpiMaster {
    pub spi: Spi1Type,
    pub cs: Pin<'A', 4, Output<PushPull>>,
}

// impl<SPI: Instance> ErrorType for SpiMaster<SPI> {
impl ErrorType for SpiMaster {
    type Error = SpiError;
}

// impl<SPI: Instance> SpiMaster<SPI> {
impl SpiMaster {
    // pub fn new(spi_parameters: SpiParameters<SPI>) -> Self
    pub fn new(spi_parameters: SpiParameters) -> Self
// where
    //     <SPI1 as SpiCommon>::Sck: From<Pin<'A', 13>>,
    //     <SPI1 as SpiCommon>::Miso: From<Pin<'A', 14>>,
    //     <SPI1 as SpiCommon>::Mosi: From<Pin<'A', 15>>,
    {
        // let cr = unsafe { &mut (*stm32f1xx_hal::pac::GPIOB::ptr()).crl };
        // // let mut cs = spi_parameters.pin_cs.into_push_pull_output(cr);
        // let mut gpiob = unsafe { stm32f1xx_hal::pac::Peripherals::steal().GPIOB.split() };
        // let mut cs = spi_parameters.pin_cs.into_push_pull_output(&mut gpiob.crl);
        // cs.set_high();
        let cs = spi_parameters
            .pin_cs
            .into_push_pull_output(spi_parameters.cr);

        // Initialize SPI
        // let spi = Spi::new(
        //     spi_parameters.spi,
        //     (
        //         spi_parameters.pin_sck.into_alternate::<5>(),
        //         spi_parameters.pin_miso.into_alternate::<5>(),
        //         spi_parameters.pin_mosi.into_alternate::<5>(),
        //     ),
        //     Mode {
        //         polarity: Polarity::IdleHigh,
        //         phase: Phase::CaptureOnSecondTransition,
        //     },
        //     spi_parameters.freq,
        //     spi_parameters.clocks,
        // );
        let mode = Mode {
            polarity: Polarity::IdleHigh,
            phase: Phase::CaptureOnSecondTransition,
        };
        let spi: Spi1Type = Spi::spi1(
            spi_parameters.spi,
            (
                spi_parameters
                    .pin_sck
                    .into_alternate_push_pull(spi_parameters.cr),
                spi_parameters.pin_miso,
                spi_parameters
                    .pin_mosi
                    .into_alternate_push_pull(spi_parameters.cr),
            ),
            &mut spi_parameters.afio.mapr,
            mode,
            spi_parameters.freq,
            *spi_parameters.clocks,
        );

        Self { spi, cs }
    }
}

impl SpiDevice for SpiMaster {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), SpiError> {
        for op in operations {
            match op {
                Operation::Read(buf) => {
                    let _ = buf;
                    // self.spi.read(buf).map_err(SpiError::from)?;
                }
                Operation::Write(buf) => {
                    self.spi.write(buf).map_err(SpiError::from)?;
                }
                Operation::Transfer(read, write) => {
                    let _ = read;
                    let _ = write;
                    // self.spi.transfer(read, write).map_err(SpiError::from)?;
                }
                Operation::TransferInPlace(buf) => {
                    let _ = buf;
                    // self.spi.transfer_in_place(buf).map_err(SpiError::from)?;
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

    // fn write_bytes(&mut self, buf: &[u8]) -> Result<(), SpiError> {
    //     self.cs.set_low();
    //     self.spi.write(buf).map_err(SpiError::from)?;
    //     self.cs.set_high();
    //     Ok(())
    // }

    // fn read_bytes(&mut self, buf: &mut [u8]) -> Result<(), SpiError> {
    //     self.cs.set_low();
    //     for byte in buf.iter_mut() {
    //         *byte = 0x00;  // Send dummy data
    //     }
    //     self.spi.transfer(buf).map_err(SpiError::from)?;
    //     self.cs.set_high();
    //     Ok(())
    // }

    // fn transfer_bytes<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8], SpiError> {
    //     self.cs.set_low();
    //     let result = self.spi.transfer(buf).map_err(SpiError::from)?;
    //     self.cs.set_high();
    //     Ok(result)
    // }
}

// TODO: Make this module generic
// pub type SpiMaster2 = SpiMaster<SPI2, 'A', 4, 'A', 13, 5, 'A', 14, 5, 'A', 15, 5>;
pub type SpiMaster1 = SpiMaster;
