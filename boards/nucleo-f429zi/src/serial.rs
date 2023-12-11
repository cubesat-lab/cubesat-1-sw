use core::fmt::{Arguments, Write as WriteFmt};

//use embedded_hal::serial::{Read, Write};
//use stm32f4xx_hal::{
//    gpio::{Pin, Alternate},
//    pac::USART3,
//    rcc::Clocks,
//    uart::{Config, Rx, Serial},
//    serial::{self, Error, Instance, Rx, Serial, Tx, CommonPins},
//};

use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::gpio::{Pin, Alternate, PushPull};
use stm32f4xx_hal::uart::{Tx, Rx, Serial};
use stm32f4xx_hal::{
    pac::{USART3},
    prelude::*,
    serial::{self, Error, CommonPins, Instance},
};

pub struct SerialUart<UART, const P: char, const N_TX: u8, const N_RX: u8, const A: u8, WORD = u8>
where UART: Instance
{
    tx: Tx<UART, WORD>,
    rx: Rx<UART, WORD>,
}

//impl<UART: CommonPins + Instance, const P: char, const N_TX: u8, const N_RX: u8, const A: u8>
//    SerialUart<UART, P, N_TX, N_RX, A>
//where
//    Pin<P, N_TX, Alternate<A>>: Into<UART::Tx<PushPull>>,
//    Pin<P, N_RX, Alternate<A>>: Into<UART::Rx<PushPull>>,
//{
//    //pub fn new(uart: UART, clocks: &Clocks, pin_tx: Pin<P, N_TX>, pin_rx: Pin<P, N_RX>) -> Self;
//}

impl SerialUart<USART3, 'D', 8, 9, 7> {
    //where UART: CommonPins
    //where <UART as stm32f4xx_hal::serial::Instance>::RegisterBlock: stm32f4xx_hal::serial::uart_impls::RegisterBlockImpl
    pub fn new(uart: USART3, clocks: &Clocks, pin_tx: Pin<'D', 8>, pin_rx: Pin<'D', 9>) -> Self
    {
        // Init UART pins
        let pin_uart_tx = pin_tx.into_alternate();
        let pin_uart_rx = pin_rx.into_alternate();
        //let pin_uart_tx = pin_tx.into_alternate();
        //let pin_uart_rx = pin_rx.into_alternate();
        //let tx: Tx<UART, WORD> = 
        //let dp = Stm32F4Peripherals::take().unwrap();
        //let gpiod = dp.GPIOD.split();
        //let pin_uart_tx = gpiod.pd8.into_alternate();
        //let pin_uart_rx = gpiod.pd9.into_alternate();

        // Init UART Serial - Default to 115_200 bauds
        let serial = Serial::new(
            uart,
            (pin_uart_tx, pin_uart_rx),
            serial::Config {
                ..Default::default()
            },
            clocks,
        ).
        unwrap();

        let (tx, rx) = serial.split();

        Self { tx, rx }
    }

    pub fn read(&mut self) -> nb::Result<u8, Error> {
        self.rx.read()
    }

    pub fn write(&mut self, byte: u8) -> nb::Result<(), Error> {
        self.tx.write(byte)
    }

    pub fn println(&mut self, s: &str) {
        self.tx.write_fmt(format_args!("{}\n", s)).unwrap();
    }

    pub fn formatln(&mut self, args: Arguments) {
        self.tx.write_fmt(args).unwrap();
        self.tx.write_str("\n").unwrap();
    }
}


pub type SerialUartUsb = SerialUart<USART3, 'D', 8, 9, 7>;
