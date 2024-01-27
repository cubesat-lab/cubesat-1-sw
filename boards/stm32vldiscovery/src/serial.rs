use core::convert::Infallible;
use core::fmt::{Arguments, Write as WriteFmt};
use stm32f1xx_hal::gpio::HL;
use stm32f1xx_hal::{
    gpio::{Alternate, Pin},
    pac::USART1,
    prelude::*,
    rcc::Clocks,
    serial::{Config, Error, Instance, Pins, Rx, Serial, Tx},
};

pub struct SerialParameters<const P: char, const N_TX: u8>
where
    Pin<P, N_TX>: HL,
{
    pub afio: stm32f1xx_hal::afio::Parts,
    pub crh: <Pin<P, N_TX> as HL>::Cr,
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
    pub fn new(
        uart: UART,
        clocks: &Clocks,
        pin_tx: Pin<P, N_TX>,
        pin_rx: Pin<P, N_RX>,
        optional_parameter: &mut SerialParameters<P, N_TX>,
    ) -> Self {
        // Init UART pins
        let pin_uart_tx = pin_tx.into_alternate_push_pull(&mut optional_parameter.crh);
        let pin_uart_rx = pin_rx;

        // Init UART Serial - Default to 115_200 bauds
        let serial = Serial::new(
            uart,
            (pin_uart_tx, pin_uart_rx),
            &mut optional_parameter.afio.mapr,
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
