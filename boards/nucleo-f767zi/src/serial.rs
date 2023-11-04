use core::fmt::{Arguments, Write as WriteFmt};
use embedded_hal::serial::{Read, Write};
use stm32f7xx_hal::{
    gpio::{Alternate, Pin},
    pac::USART3,
    rcc::Clocks,
    serial::{self, Error, Instance, PinRx, PinTx, Rx, Serial, Tx},
};

pub struct SerialUart<UART, const P: char, const N_TX: u8, const N_RX: u8, const A: u8> {
    tx: Tx<UART>,
    rx: Rx<UART>,
}

impl<UART: Instance, const P: char, const N_TX: u8, const N_RX: u8, const A: u8>
    SerialUart<UART, P, N_TX, N_RX, A>
where
    Pin<P, N_TX, Alternate<A>>: PinTx<UART>,
    Pin<P, N_RX, Alternate<A>>: PinRx<UART>,
{
    pub fn new(uart: UART, clocks: &Clocks, pin_tx: Pin<P, N_TX>, pin_rx: Pin<P, N_RX>) -> Self {
        // Init UART pins
        let pin_uart_tx: Pin<P, N_TX, Alternate<A>> = pin_tx.into_alternate();
        let pin_uart_rx: Pin<P, N_RX, Alternate<A>> = pin_rx.into_alternate();

        // Init UART Serial - Default to 115_200 bauds
        let serial = Serial::new(
            uart,
            (pin_uart_tx, pin_uart_rx),
            clocks,
            serial::Config {
                ..Default::default()
            },
        );

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
