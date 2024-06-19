#![no_std]
#![feature(type_alias_impl_trait)]

use cc1101::Error;

pub use cc1101::{
    AddressFilter, Cc1101, CcaMode, MachineState, Modulation, NumPreambleBytes,
    PacketLength, RadioMode, SyncMode, CommandStrobe, FIFO_MAX_SIZE
};

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use fugit::Duration;
use rtic_monotonics::systick::Systick;


#[derive(Debug)]
pub enum Cc1101AsyncError<SpiE, GpioE> {
    /// Operation timeout
    TimeoutError,
    /// The TX FIFO buffer underflowed, too large packet for configured packet length.
    TxUnderflow,
    /// The RX FIFO buffer overflowed, too small buffer for configured packet length.
    RxOverflow,
    /// Corrupt packet received with invalid CRC.
    CrcMismatch,
    /// Invalid state read from MARCSTATE register
    InvalidState(u8),
    /// User Input Error
    UserInputError(usize),
    /// Platform-dependent SPI-errors, such as IO errors.
    Spi(SpiE),
    /// Platform-dependent GPIO-errors, such as IO errors.
    Gpio(GpioE),
}

impl<SpiE, GpioE> From<Error<SpiE, GpioE>> for Cc1101AsyncError<SpiE, GpioE> {
    fn from(e: Error<SpiE, GpioE>) -> Self {
        match e {
            Error::TxUnderflow => Cc1101AsyncError::TxUnderflow,
            Error::RxOverflow => Cc1101AsyncError::RxOverflow,
            Error::CrcMismatch => Cc1101AsyncError::CrcMismatch,
            Error::InvalidState(value) => Cc1101AsyncError::InvalidState(value),
            Error::UserInputError(value) => Cc1101AsyncError::UserInputError(value),
            Error::Spi(inner) => Cc1101AsyncError::Spi(inner),
            Error::Gpio(inner) => Cc1101AsyncError::Gpio(inner),
        }
    }
}

pub struct Cc1101Async<SPI, CS> {
    cc1101: Cc1101<SPI, CS>,
}

impl<SPI, CS, SpiE, GpioE> Cc1101Async<SPI, CS>
where
    SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    CS: OutputPin<Error = GpioE>,
{
    pub fn new(spi: SPI, cs: CS) -> Result<Self, Cc1101AsyncError<SpiE, GpioE>> {
        Ok(Cc1101Async { cc1101: Cc1101::new(spi, cs)?})
    }

    pub fn command(&mut self, command: CommandStrobe) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.command(command)?)
    }

    pub fn set_frequency(&mut self, hz: u64) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_frequency(hz)?)
    }

    pub fn set_deviation(&mut self, deviation: u64) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_deviation(deviation)?)
    }

    pub fn set_data_rate(&mut self, baud: u64) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_data_rate(baud)?)
    }

    pub fn enable_fec(&mut self, enable: bool) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.enable_fec(enable)?)
    }

    pub fn set_cca_mode(&mut self, cca_mode: CcaMode) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_cca_mode(cca_mode)?)
    }

    pub fn set_num_preamble(&mut self, num_preamble: NumPreambleBytes) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_num_preamble(num_preamble)?)
    }

    pub fn crc_enable(&mut self, enable: bool) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.crc_enable(enable)?)
    }

    pub fn set_chanbw(&mut self, bandwidth: u64) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_chanbw(bandwidth)?)
    }

    pub fn get_hw_info(&mut self) -> Result<(u8, u8), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.get_hw_info()?)
    }

    pub fn get_rssi_dbm(&mut self) -> Result<i16, Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.get_rssi_dbm()?)
    }

    pub fn get_lqi(&mut self) -> Result<u8, Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.get_lqi()?)
    }

    pub fn set_sync_mode(&mut self, sync_mode: SyncMode) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_sync_mode(sync_mode)?)
    }

    pub fn set_modulation(&mut self, format: Modulation) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_modulation(format)?)
    }

    pub fn set_address_filter(&mut self, filter: AddressFilter) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_address_filter(filter)?)
    }

    pub fn set_packet_length(&mut self, length: PacketLength) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.set_packet_length(length)?)
    }

    pub fn white_data(&mut self, enable: bool) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.white_data(enable)?)
    }

    pub fn read_tx_bytes(&mut self) -> Result<u8, Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.read_tx_bytes()?)
    }

    pub fn read_rx_bytes(&mut self) -> Result<u8, Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.read_rx_bytes()?)
    }

    pub fn read_machine_state(&mut self) -> Result<MachineState, Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.read_machine_state()?)
    }

    pub async fn check_machine_state(&mut self, target_state: MachineState) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        let delay = fugit::ExtU64::micros(1000);
        loop {
            let machine_state = self.read_machine_state()?;

            if machine_state == target_state {
                /* Successful scenario */
                return Ok(());
            }

            /* Error scenarios */
            if machine_state == MachineState::RXFIFO_OVERFLOW {
                return Err(Cc1101AsyncError::RxOverflow);
            } else if machine_state == MachineState::RXFIFO_OVERFLOW {
                return Err(Cc1101AsyncError::TxUnderflow);
            } else {
                /* Ignore other states */
            }

            Systick::delay(delay).await;
        }
    }

    pub async fn await_machine_state(&mut self, target_state: MachineState, timeout: Duration<u64, 1, 1000>) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        match Systick::timeout_after(timeout, self.check_machine_state(target_state)).await {
            Ok(result) => result,
            Err(_) => Err(Cc1101AsyncError::TimeoutError),
        }
    }

    pub fn read_data(&mut self, data: &mut [u8]) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.read_data(data)?)
    }

    pub fn write_data(&mut self, data: &mut [u8]) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        Ok(self.cc1101.write_data(data)?)
    }

    /// Set Radio Mode.
    pub async fn set_radio_mode(&mut self, radio_mode: RadioMode, timeout: Duration<u64, 1, 1000>) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {

        // Set "Idle" mode before going into any other mode
        self.command(CommandStrobe::ExitRxTx)?;
        self.await_machine_state(MachineState::IDLE, timeout).await?;

        match radio_mode {
            RadioMode::Idle => {
                // Do nothing
            }
            RadioMode::Sleep => {
                self.command(CommandStrobe::EnterPowerDownMode)?;
            }
            RadioMode::Calibrate => {
                self.command(CommandStrobe::CalFreqSynthAndTurnOff)?;
                self.await_machine_state(MachineState::MANCAL, timeout).await?;
            }
            RadioMode::Transmit => {
                self.command(CommandStrobe::EnableTx)?;
                self.await_machine_state(MachineState::TX, timeout).await?;
            }
            RadioMode::Receive => {
                self.command(CommandStrobe::EnableRx)?;
                self.await_machine_state(MachineState::RX, timeout).await?;
            }
        };
        Ok(())
    }
}
