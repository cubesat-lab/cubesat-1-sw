use bxcan::{self, Can as BxCan, Data, Frame, StandardId, TransmitStatus};
use core;
// use core::marker::PhantomData;
use stm32f4xx_hal::can::{
    Can as HalCan, Can1 as HalCan1, Can2 as HalCan2, Instance as HalInstance,
};
use stm32f4xx_hal::gpio::alt::CanCommon;
use stm32f4xx_hal::gpio::{self, Alternate, Pin, PinMode, AF9, PA11, PA12, PB13, PB5, PB6};
use stm32f4xx_hal::pac::{CAN1, CAN2};
use stm32f4xx_hal::rcc::{BusClock, Clocks};

pub enum Error {
    InvalidId,
    InvalidData,
    TransmissionError,
    BufferOverrun,
}

pub enum CanPins {
    Can1((Pin<'A', 11, AF9>, Pin<'A', 12, AF9>)),
    Can2((Pin<'B', 5, AF9>, Pin<'B', 6, AF9>)),
}

// trait to abstract CAN1 and CAN2
pub trait SupportedCanInstance {
    const REGISTERS: *mut bxcan::RegisterBlock;
}

impl SupportedCanInstance for CAN1 {
    const REGISTERS: *mut bxcan::RegisterBlock = CAN1::ptr() as *mut _;
}

impl SupportedCanInstance for CAN2 {
    const REGISTERS: *mut bxcan::RegisterBlock = CAN2::ptr() as *mut _;
}

pub struct CanParameters<'a, CAN> {
    pub can: CAN,
    pub clocks: &'a Clocks,
    pub pins: CanPins,
}

// builder configuration parameters
pub struct CanConfiguration {
    pub bit_timing: u32,
    pub loopback: bool,
    pub silent: bool,
    pub auto_retransmission: bool,
}

// wrapper over CAN1 and CAN2
pub struct CanWrapper<CAN> {
    can: CAN,
}

unsafe impl<CAN: SupportedCanInstance> bxcan::Instance for CanWrapper<CAN> {
    const REGISTERS: *mut bxcan::RegisterBlock = CAN::REGISTERS;
}

unsafe impl<CAN: SupportedCanInstance> bxcan::FilterOwner for CanWrapper<CAN> {
    const NUM_FILTER_BANKS: u8 = 28;
}

// unsafe impl bxcan::MasterInstance for CanWrapper<CAN1> {}

// unsafe impl bxcan::FilterOwner for CanWrapper<CAN2> {
//     const NUM_FILTER_BANKS: u8 = 14;
// }

// unsafe impl bxcan::FilterOwner for CanWrapper<CAN> {
//     const NUM_FILTER_BANKS: u8 = CanWrapper::NUM_FILTER_BANKS;
// }

// wrapper over the wrapped pac::CAN
pub struct Can<CAN: SupportedCanInstance> {
    can: BxCan<CanWrapper<CAN>>,
}

impl<CAN> Can<CAN>
where
    CAN: SupportedCanInstance,
{
    pub fn new(can_parameters: CanParameters<CAN>, config: CanConfiguration) -> Self {
        let can_wrapper = CanWrapper {
            can: can_parameters.can,
        };

        // configure the builder and return the can interface
        let can_interface = BxCan::builder(can_wrapper)
            .set_bit_timing(config.bit_timing)
            .set_loopback(config.loopback)
            .set_silent(config.silent)
            .set_automatic_retransmit(config.auto_retransmission)
            .enable();

        Can { can: can_interface }
    }

    pub fn send(&mut self, id: u16, data: &[u8]) -> Result<TransmitStatus, Error> {
        let frame_id = StandardId::new(id).ok_or(Error::InvalidId)?;
        let frame_data = Data::new(data).ok_or(Error::InvalidData)?;

        let frame = Frame::new_data(frame_id, frame_data);
        self.can
            .transmit(&frame)
            .map_err(|_| Error::TransmissionError)
    }

    pub fn receive(&mut self) -> Result<Frame, Error> {
        self.can.receive().map_err(|_| Error::BufferOverrun)
    }

    pub fn configure_filters(
        &mut self,
        master: Option<&mut Can<CAN>>,
        index: u8,
        fifo: bxcan::Fifo,
        config: bxcan::filter::BankConfig,
    ) {
        if CAN::REGISTERS == CAN1::REGISTERS {
            self.can.modify_filters().enable_bank(index, fifo, config);
        } else {
            master
                .unwrap()
                .can
                .modify_filters()
                .enable_bank(index, fifo, config);
        }
    }
}
