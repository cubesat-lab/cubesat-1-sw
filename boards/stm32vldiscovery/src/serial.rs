use core::fmt::{Arguments, Write as WriteFmt};

use core::convert::Infallible;

use stm32f1xx_hal::{
    gpio::{Alternate, Pin},
    pac::{USART1},
    rcc::Clocks,
    prelude::*,
    serial::{Error, Instance, Rx, Config, Serial, Tx, Pins},
};

pub struct SerialUart<UART, const P: char, const N_TX: u8, const N_RX: u8, const A: u8> {
    tx: Tx<UART>,
    rx: Rx<UART>,
}


impl<UART: Instance, const P: char, const N_TX: u8, const N_RX: u8, const A: u8>
    SerialUart<UART, P, N_TX, N_RX, A>
where
    (Pin<P, N_TX, Alternate>, Pin<P, N_RX>): Pins<UART>,
{
    pub fn new(afio: &mut stm32f1xx_hal::afio::Parts, uart: UART, clocks: &Clocks, pin_tx: Pin<P, N_TX, Alternate>, pin_rx: Pin<P, N_RX>) -> Self {

       
        let pin_uart_tx = pin_tx;
        let pin_uart_rx = pin_rx;     

        // Init UART Serial - Default to 115_200 bauds
        let serial = Serial::new(
            uart,
            (pin_uart_tx, pin_uart_rx),
            &mut afio.mapr,
            Config::default()
                .baudrate(115200.bps())
                .wordlength_9bits()
                .parity_none(),
            &clocks,
        );

        let (tx, rx) = serial.split();

        Self { tx, rx }
    }

    pub fn read(&mut self) -> nb::Result<u8, Error> {
        self.rx.read()
    }

    pub fn write(&mut self, byte: u8) -> nb::Result<(), Infallible> {
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

pub type SerialUartUsb = SerialUart<USART1, 'A', 9, 10, 7>;
