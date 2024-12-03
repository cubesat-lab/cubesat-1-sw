// #![no_main]
// #![no_std]

// use cortex_m_rt::entry;
// use nb::block;
// use stm32f4xx_hal::{
//     can::{Can as HalCan},
//     gpio::{self, gpioa::{PA11, PA12}, Alternate, AF9},
//     pac::{self, CAN1},
//     prelude::*,
// };

// use bxcan::{self, Data, Frame, StandardId};
// use cortex_m_semihosting::hprintln;
// use panic_semihosting;

// use fugit::Hertz;

// use nucleo_f446re::can::{Can, CanConfiguration, CanPins, CanParameters};
// use nucleo_f446re::serial::{SerialParameters, SerialUartUsb};

// #[entry]
// fn main() -> ! {
//     let _cp = cortex_m::Peripherals::take().unwrap();
//     let dp = pac::Peripherals::take().unwrap();

//     let rcc = dp.RCC.constrain();
//     let clocks = rcc.cfgr.sysclk(180.MHz() as Hertz<u32>).freeze();

//     // dp.CAN1.fmr().modify(|_, w| unsafe { w.can2sb().bits(14) });

//     let gpioa = dp.GPIOA.split();
//     let can1_rx = gpioa.pa11.into_alternate::<9>();
//     let can1_tx = gpioa.pa12.into_alternate::<9>();

//     let gpiob = dp.GPIOB.split();
//     let can2_rx = gpiob.pb5.into_alternate::<9>();
//     let can2_tx = gpiob.pb6.into_alternate::<9>();

//     let can1_parameters = CanParameters {
//         can: dp.CAN1,
//         clocks: &clocks,
//         pins: CanPins::Can1((can1_rx, can1_tx)),
//     };

//     let can1_config = CanConfiguration {
//         bit_timing: 0x001e0013,
//         loopback: false,
//         silent: false,
//         auto_retransmission: true,
//     };

//     let can2_parameters = CanParameters {
//         can: dp.CAN2,
//         clocks: &clocks,
//         pins: CanPins::Can2((can2_rx, can2_tx)),
//     };

//     let can2_config = CanConfiguration {
//         bit_timing: 0x001e0013,
//         loopback: false,
//         silent: false,
//         auto_retransmission: true,
//     };

//     let mut can1 = Can::new(can1_parameters, can1_config);
//     let mut can2 = Can::new(can2_parameters, can2_config);

// //     can1.configure_filters(
// //         None,
// //         0,
// //         bxcan::Fifo::Fifo0,
// //         bxcan::filter::BankConfig::Mask32(bxcan::filter::Mask32::accept_all()),
// //     );

// //     can2.configure_filters(
// //         Some(&mut can1),
// //         14,
// //         bxcan::Fifo::Fifo0,
// //         bxcan::filter::BankConfig::Mask32(bxcan::filter::Mask32::accept_all())
// // ,
// //     );

//     const DATA: [u8; 8] = [0, 0xFF, 0, 0xFF, 0, 0xFF, 0, 0xFF];

//     let serial_parameters = SerialParameters {
//         uart: dp.USART2,
//         clocks: &clocks,
//         pin_tx: gpioa.pa2.into_alternate(),
//         pin_rx: gpioa.pa3.into_alternate(),
//     };
//     let mut serial = SerialUartUsb::new(serial_parameters);

//     // let can = Can1Wrapper(HalCan::new(dp.CAN1, (can_tx, can_rx)));
//     // let mut bx_can = bxcan::Can::builder(can)
//     //     .set_bit_timing(0x001a0005)
//     //     .set_loopback(true)
//     //     .enable();

//     // let frame = Frame::new_data(
//     //     StandardId::new(0).unwrap(),
//     //     bxcan::Data::new(&[0, 0xFF, 0, 0xFF, 0, 0xFF, 0, 0xFF]).unwrap());
//     let mut id: u16 = 0;

//     loop {
//         // bx_can.transmit(&frame);
//         id = (id + 1) % 0x7FF;

//         match can1.send(0, &DATA) {
//             Ok(_transmit_status) => {
//                 serial.println("Successful transmission");
//             },
//             Err(_err) => {
//                 serial.println("Transmission error");
//             }
//         }
//         match can1.receive() {
//             Ok(_frame) => {
//                 serial.println("Received successfully");
//             },
//             Err(_err) => {
//                 serial.println("Error receiving");
//             }
//         }

//         match can2.send(0, &DATA) {
//             Ok(_transmit_status) => {
//                 serial.println("Successful transmission");
//             },
//             Err(_err) => {
//                 serial.println("Transmission error");
//             }
//         }
//         match can2.receive() {
//             Ok(_frame) => {
//                 serial.println("Received successfully");
//             },
//             Err(_err) => {
//                 serial.println("Error receiving");
//             }
//         }
//         // bx_can.receive();
//     }
// }

// --------------------------------------------------------------

#![no_main]
#![no_std]

use panic_halt as _;

use bxcan::filter::Mask32;
use bxcan::{ExtendedId, Fifo, Frame, MasterInstance, StandardId};
// use cortex_m;
use cortex_m_rt::entry;
// use nb::block;
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

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.CAN1.fmr().modify(|_, w| unsafe { w.can2sb().bits(14) });
    dp.CAN1.ier().modify(|_, w| w.tmeie().set_bit());

    let rcc = dp.RCC.constrain();
    let _clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(144.MHz())
        .pclk1(36.MHz())
        .freeze();

    // let clock1 = clocks.sysclk();
    // let clock2 = clocks.pclk1();
    // let clock3 = clocks.pclk2();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

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

    let mut filters = can1.modify_filters();
    for bank in 0..14 {
        filters.enable_bank(bank, Fifo::Fifo0, Mask32::accept_all());
    }

    filters.set_split(14);
    let mut slave_filters = filters.slave_filters();
    for bank in 14..28 {
        slave_filters.enable_bank(bank, Fifo::Fifo0, Mask32::accept_all());
    }

    drop(filters);

    let mut can2 = {
        let tx = gpiob.pb6.into_alternate::<9>();
        let rx = gpiob.pb12.into_alternate::<9>();

        let can = Can2Wrapper(dp.CAN2.can((tx, rx)));

        let can2 = bxcan::Can::builder(can)
            .set_bit_timing(0x001e0007)
            .set_loopback(false)
            .set_silent(false)
            .set_automatic_retransmit(false)
            .enable();

        can2
    };

    let mut test: [u8; 8] = [0; 8];
    let mut count: u8 = 0;
    let _id: u16 = 0x0000;

    test[1] = 0;
    test[2] = 0;
    test[3] = 0;
    test[4] = 0;
    test[5] = 0;
    test[6] = 0;
    test[7] = 0;
    loop {
        test[0] = count;
        let test_frame = Frame::new_data(StandardId::new(1).unwrap(), test);
        let second_test_frame = Frame::new_data(ExtendedId::new(0).unwrap(), test);

        if can1.is_transmitter_idle() {
            // let _status = block!(can1.transmit(&test_frame)).unwrap();
            // let _second_status = block!(can1.transmit(&second_test_frame)).unwrap();

            let _status = can1.transmit(&test_frame).unwrap();
            let _second_status = can1.transmit(&second_test_frame).unwrap();
        }

        if can2.is_transmitter_idle() {
            // let _third_status = block!(can2.transmit(&second_test_frame)).unwrap();
            // let _fourth_status = block!(can2.transmit(&test_frame)).unwrap();
            let _third_status = can2.transmit(&second_test_frame).unwrap();
            let _fourth_status = can2.transmit(&test_frame).unwrap();
        }

        // let status2 = can2.transmit(&test_frame).unwrap();
        // let _first_received = block!(can1.receive()).unwrap();
        // let _received = block!(can2.receive()).unwrap();
        // let _first_received = can1.receive().unwrap();
        // let _received = can2.receive().unwrap();
        if count < 255 {
            count += 1;
        } else {
            count = 0;
        }
    }
}
