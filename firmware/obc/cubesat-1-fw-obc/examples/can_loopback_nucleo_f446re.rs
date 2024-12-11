#![no_main]
#![no_std]

use panic_halt as _;

use bxcan::filter::Mask32;
use bxcan::{ExtendedId, Fifo, Frame, Id, MasterInstance, StandardId};
use core::{
    fmt::{self, Debug},
    ops::Deref,
};
// use cortex_m;
use cortex_m_rt::entry;
// use nb::block;
use nucleo_f446re::serial::{SerialParameters, SerialUartUsb};
use stm32f4xx_hal::can::Can;
use stm32f4xx_hal::{pac, prelude::*};

#[allow(dead_code)]
pub struct Can1Wrapper(Can<pac::CAN1>);

unsafe impl bxcan::Instance for Can1Wrapper {
    const REGISTERS: *mut bxcan::RegisterBlock = pac::CAN1::ptr() as *mut _;
}

unsafe impl bxcan::FilterOwner for Can1Wrapper {
    const NUM_FILTER_BANKS: u8 = 28;
}

unsafe impl MasterInstance for Can1Wrapper {}

#[allow(dead_code)]
pub struct Can2Wrapper(Can<pac::CAN2>);
unsafe impl bxcan::Instance for Can2Wrapper {
    const REGISTERS: *mut bxcan::RegisterBlock = pac::CAN2::ptr() as *mut _;
}

// wrap the Frame structure in order to provide a Debug trait implementation that shows the id and the frame's data as hexadecimal values
struct FrameDisplay(Frame);

impl Debug for FrameDisplay {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Frame")
            .field(
                "id",
                &format_args!(
                    "{:#X}",
                    match self.0.id() {
                        Id::Standard(id) => id.as_raw() as u32,
                        Id::Extended(id) => id.as_raw(),
                    }
                ),
            )
            .field(
                "data",
                &format_args!("{:#X?}", &self.0.data().unwrap().deref()),
            )
            .finish()
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.CAN1.fmr().modify(|_, w| unsafe { w.can2sb().bits(14) });
    dp.CAN1.ier().modify(|_, w| w.tmeie().set_bit());

    let rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(144.MHz())
        .pclk1(36.MHz())
        .pclk2(72.MHz())
        .freeze();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    // initialize the CAN1 peripheral
    let mut can1 = {
        let rx = gpioa.pa11.into_alternate::<9>();
        let tx = gpioa.pa12.into_alternate::<9>();

        // let can = Can::new(dp.CAN1, (tx, rx));
        let can = Can1Wrapper(dp.CAN1.can((tx, rx)));

        bxcan::Can::builder(can)
            .set_bit_timing(0x001e0003)
            .set_loopback(false)
            .set_silent(false)
            .set_automatic_retransmit(true)
            .enable()
    };

    // configure CAN1's filters (0..13)
    let mut filters = can1.modify_filters();
    for bank in 0..14 {
        filters.enable_bank(bank, Fifo::Fifo0, Mask32::accept_all());
    }

    // make the slave's filters start from the 14th
    filters.set_split(14);

    // get CAN2's filters
    let mut slave_filters = filters.slave_filters();

    // configure CAN2's filters (14..27)
    for bank in 14..28 {
        slave_filters.enable_bank(bank, Fifo::Fifo0, Mask32::accept_all());
    }

    drop(filters);

    // initialize the CAN2 peripheral
    let mut can2 = {
        let tx = gpiob.pb6.into_alternate::<9>();
        let rx = gpiob.pb12.into_alternate::<9>();

        let can = Can2Wrapper(dp.CAN2.can((tx, rx)));

        let can2 = bxcan::Can::builder(can)
            .set_bit_timing(0x001e0003)
            .set_loopback(false)
            .set_silent(false)
            .set_automatic_retransmit(false)
            .enable();

        can2
    };

    // initialize the serial interface
    let serial_parameters = SerialParameters {
        uart: dp.USART2,
        clocks: &clocks,
        pin_tx: gpioa.pa2.into_alternate(),
        pin_rx: gpioa.pa3.into_alternate(),
    };
    let mut serial = SerialUartUsb::new(serial_parameters);
    serial.println("Serial initialized");

    // initialize the count used in the test frames
    let mut count: u8 = 0;

    // set the id used in the test frames
    let can1_id: u16 = 0x0001;
    let can2_id: u16 = 0x0002;

    // initialize the test data
    let mut can1_test_data: [u8; 8] = [count, 1, 1, 1, 1, 1, 1, 1];
    let mut can2_test_data: [u8; 8] = [count, 2, 2, 2, 2, 2, 2, 2];
    loop {
        // update the first byte of the test frames
        can1_test_data[0] = count;
        can2_test_data[0] = count;
        // initialize the test frames
        let can1_standard_frame = Frame::new_data(StandardId::new(can1_id).unwrap(), can1_test_data);
        let _can1_extended_frame = Frame::new_data(ExtendedId::new(can1_id as u32).unwrap(), can1_test_data);

        let can2_standard_frame = Frame::new_data(StandardId::new(can2_id).unwrap(), can2_test_data);
        let _can2_extended_frame = Frame::new_data(ExtendedId::new(can2_id as u32).unwrap(), can2_test_data);


        // check if the transmitter is available
        if can1.is_transmitter_idle() {
            // try to send the frames on the CAN bus
            let _standard_status = can1.transmit(&can1_standard_frame);
            // let _extended_status = can1.transmit(&can1_extended_frame);
        }

        loop {
            // try to receive all frames
            let can1_receive = can1.receive();
            match can1_receive {
                Ok(frame) => {
                    // show the frame on the serial interface (run python3 ./tools/serial_link.py -p <board_port> -b <bit_rate>)
                    let frame_display = FrameDisplay(frame);
                    serial.formatln(format_args!("Received frame on CAN1: {:?}", frame_display));
                }
                Err(err) => {
                    // show whether there was a buffer overrun, otherwise it's a WouldBlock error which is ignored
                    if let nb::Error::Other(e) = err {
                        serial.formatln(format_args!("CAN1 receive error: {:?}", e));
                    } else {
                        break;
                    }
                },
            }
        }

        // check if the transmitter is available
        if can2.is_transmitter_idle() {
            // try to send the frames on the CAN bus
            let _standard_status = can2.transmit(&can2_standard_frame);
            // let _extended_status = can2.transmit(&can2_extended_frame);
        }

        loop {
            // try to receive all frames
            let can2_receive = can2.receive();
            match can2_receive {
                Ok(frame) => {
                    // show the frame on the serial interface (run python3 ./tools/serial_link.py -p <board_port> -b 115200>)
                    let frame_display = FrameDisplay(frame);
                    serial.formatln(format_args!("Received frame on CAN2: {:?}", frame_display));
                }
                Err(err) => {
                    // show whether there was a buffer overrun, otherwise it's a WouldBlock error which is ignored
                    if let nb::Error::Other(e) = err {
                        serial.formatln(format_args!("CAN2 receive error: {:?}", e));
                    } else {
                        break;
                    }
                },
            }
        }

        // increment the counter used as the first byte of the frames
        if count < 255 {
            count += 1;
        } else {
            count = 0;
        }
    }
}
