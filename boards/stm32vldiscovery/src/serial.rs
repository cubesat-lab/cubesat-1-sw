use core::convert::Infallible;
use core::fmt::{Arguments, Write as WriteFmt};
use stm32f1xx_hal::{
    afio::Parts,
    gpio::{Alternate, Pin, HL},
    pac::USART1,
    prelude::*,
    rcc::Clocks,
    serial::{Config, Error, Instance, Pins, Rx, Serial, Tx},
};

pub struct SerialParameters<'a, UART, const P: char, const N_TX: u8, const N_RX: u8>
where
    Pin<P, N_TX>: HL,
    Pin<P, N_RX>: HL,
{
    pub uart: UART,
    pub clocks: &'a Clocks,
    pub pin_tx: Pin<P, N_TX>,
    pub pin_rx: Pin<P, N_RX>,
    pub afio: &'a mut Parts,
    pub cr: &'a mut <Pin<P, N_TX> as HL>::Cr,
}

pub struct SerialUart<UART, const P: char, const N_TX: u8, const N_RX: u8, const A: u8> {
    tx: Tx<UART>,
    rx: Rx<UART>,
}

impl<UART: Instance, const P: char, const N_TX: u8, const N_RX: u8, const A: u8>
    SerialUart<UART, P, N_TX, N_RX, A>
where
    (Pin<P, N_TX, Alternate>, Pin<P, N_RX>): Pins<UART>,
    Pin<P, N_TX>: HL,
    Pin<P, N_RX>: HL,
{
    pub fn new(serial_parameters: SerialParameters<UART, P, N_TX, N_RX>) -> Self {
        // Init UART pins
        let pin_uart_tx = serial_parameters
            .pin_tx
            .into_alternate_push_pull(serial_parameters.cr);
        let pin_uart_rx = serial_parameters.pin_rx;

        // Init UART Serial - Default to 115_200 bauds
        let serial = Serial::new(
            serial_parameters.uart,
            (pin_uart_tx, pin_uart_rx),
            &mut serial_parameters.afio.mapr,
            Config::default()
                .baudrate(115200.bps())
                .wordlength_9bits()
                .parity_none(),
            serial_parameters.clocks,
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
