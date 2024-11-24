use core::fmt::{Arguments, Write as WriteFmt};
use core::marker::PhantomData;
use stm32f4xx_hal::{
    gpio::{Alternate, Pin, PushPull},
    nb,
    pac::USART2,
    prelude::{_embedded_hal_serial_nb_Read, _embedded_hal_serial_nb_Write},
    rcc::Clocks,
    serial::{self, CommonPins, Error, Instance, Rx, Serial, Tx},
};

pub struct SerialParameters<'a, UART, TX, RX> {
    pub uart: UART,
    pub clocks: &'a Clocks,
    pub pin_tx: TX,
    pub pin_rx: RX,
}

pub struct SerialUart<UART: CommonPins, TX, RX> {
    tx: Tx<UART>,
    rx: Rx<UART>,
    _tx_pin: PhantomData<TX>,
    _rx_pin: PhantomData<RX>,
}

impl<UART, TX, RX> SerialUart<UART, TX, RX>
where
    UART: Instance,
    TX: Into<UART::Tx<PushPull>>,
    RX: Into<UART::Rx<PushPull>>,
{
    pub fn new(serial_parameters: SerialParameters<UART, TX, RX>) -> Self {
        // Initialize UART Serial - Default to 115_200 bauds
        let serial = Serial::new(
            serial_parameters.uart,
            (
                serial_parameters.pin_tx.into(),
                serial_parameters.pin_rx.into(),
            ),
            serial::Config {
                ..Default::default()
            },
            serial_parameters.clocks,
        )
        .unwrap();

        let (tx, rx) = serial.split();

        Self {
            tx,
            rx,
            _tx_pin: PhantomData,
            _rx_pin: PhantomData,
        }
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

/// Type alias for USART2 on pins PA2 (TX) and PA3 (RX) with AF7
pub type SerialUartUsb =
    SerialUart<USART2, Pin<'A', 2, Alternate<7, PushPull>>, Pin<'A', 3, Alternate<7, PushPull>>>;
