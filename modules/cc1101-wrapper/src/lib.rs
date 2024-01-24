#![no_std]

use cc1101_async::{
    Cc1101Async, Cc1101AsyncError,
    AddressFilter, CcaMode, Modulation, NumPreambleBytes, PacketLength, RadioMode,
    SyncMode, CommandStrobe, MachineState, FIFO_MAX_SIZE
};

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use rtic_monotonics::systick::Systick;

const PACKET_LENGTH_BYTES: usize = 8;
// Project specific radio configurations
const FREQUENCY: u64 = 433_000_000; // 433 MHz
const MODULATION: Modulation = Modulation::BinaryFrequencyShiftKeying;
const SYNC_MODE: SyncMode = SyncMode::MatchFull(0xA503);
// const PACKET_LENGTH: PacketLength = PacketLength::Fixed(64);
// const PACKET_LENGTH: PacketLength = PacketLength::Fixed(PACKET_LENGTH_BYTES as u8);
const PACKET_LENGTH: PacketLength = PacketLength::Variable(64);
const NUM_PREAMBLE_BYTES: NumPreambleBytes = NumPreambleBytes::Four;
const CCA_MODE: CcaMode = CcaMode::AlwaysClear;
const CRC_ENABLE: bool = false;
// const ADDRESS_FILTER: AddressFilter = AddressFilter::Device(0x3e);
// const ADDRESS_FILTER: AddressFilter = AddressFilter::DeviceLowBroadcast(1);
const ADDRESS_FILTER: AddressFilter = AddressFilter::Disabled;
const DEVIATION: u64 = 20_629;
const DATARATE: u64 = 38_383;
const BANDWIDTH: u64 = 101_562;

enum RxState {
    Waiting,
    Receiving,
    Received,
    Error,
}


#[derive(Debug, PartialEq)]
pub enum Cc1101WrapperError {
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
    Spi,
    /// Platform-dependent GPIO-errors, such as IO errors.
    Gpio,
}

impl<SpiE, GpioE> From<Cc1101AsyncError<SpiE, GpioE>> for Cc1101WrapperError {
    fn from(e: Cc1101AsyncError<SpiE, GpioE>) -> Self {
        match e {
            Cc1101AsyncError::TimeoutError => Cc1101WrapperError::TimeoutError,
            Cc1101AsyncError::TxUnderflow => Cc1101WrapperError::TxUnderflow,
            Cc1101AsyncError::RxOverflow => Cc1101WrapperError::RxOverflow,
            Cc1101AsyncError::CrcMismatch => Cc1101WrapperError::CrcMismatch,
            Cc1101AsyncError::InvalidState(value) => Cc1101WrapperError::InvalidState(value),
            Cc1101AsyncError::UserInputError(value) => Cc1101WrapperError::UserInputError(value),
            Cc1101AsyncError::Spi(_) => Cc1101WrapperError::Spi,
            Cc1101AsyncError::Gpio(_) => Cc1101WrapperError::Gpio,
        }
    }
}

pub struct Cc1101Wrapper<SPI, CS> {
    cc1101: Cc1101Async<SPI, CS>
}

impl<SPI, CS, SpiE, GpioE> Cc1101Wrapper<SPI, CS>
where
    SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    CS: OutputPin<Error = GpioE>,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        let cc1101 = Cc1101Async::new(spi, cs);

        match cc1101 {
            Ok(cc1101) => Cc1101Wrapper { cc1101 },
            Err(_error) => panic!("Error initializing CC1101"),
        }
    }

    pub fn init_config(&mut self) -> Result<(), Cc1101WrapperError> {
        self.cc1101.set_frequency(FREQUENCY)?;
        self.cc1101.set_modulation(MODULATION)?;
        self.cc1101.set_sync_mode(SYNC_MODE)?;
        self.cc1101.set_cca_mode(CCA_MODE)?;
        self.cc1101.set_num_preamble(NUM_PREAMBLE_BYTES)?;
        self.cc1101.crc_enable(CRC_ENABLE)?;
        self.cc1101.set_packet_length(PACKET_LENGTH)?;
        self.cc1101.set_address_filter(ADDRESS_FILTER)?;
        self.cc1101.set_deviation(DEVIATION)?;
        self.cc1101.set_data_rate(DATARATE)?;
        self.cc1101.set_chanbw(BANDWIDTH)?;

        Ok(())
    }

    // pub async fn test_transmit(&mut self) -> Result<(), Cc1101WrapperError> {

    //     // Prepare data
    //     let mut data: [u8; 64] = [0; 64];
    //     for (index, element) in data.iter_mut().enumerate() {
    //         *element = index as u8;
    //     }
    //     // data[0] = 64;

    //     // Reset CC1101
    //     self.cc1101.command(CommandStrobe::ResetChip)?;
    //     Systick::delay(fugit::ExtU64::micros(2000)).await;
    //     let _ = self.cc1101.read_machine_state()?;

    //     // Flush FIFO TX
    //     self.cc1101.command(CommandStrobe::FlushTxFifoBuffer)?;
    //     let _ = self.cc1101.read_machine_state()?;

    //     // Calibrate
    //     self.cc1101.set_radio_mode(RadioMode::Calibrate).await?;
    //     self.cc1101.await_machine_state(MachineState::IDLE).await?;

    //     // Write data, start TX
    //     self.cc1101.write_data(&mut data)?;
    //     self.cc1101.set_radio_mode(RadioMode::Transmit).await?;
    //     Systick::delay(fugit::ExtU64::micros(5000)).await;

    //     // Wait for TX to finish, get result
    //     let result = self.cc1101.await_machine_state(MachineState::IDLE).await;

    //     match result {
    //         Ok(_) => self.cc1101.command(CommandStrobe::NoOperation)?,
    //         Err(e) => {
    //             match e {
    //                 Cc1101AsyncError::TimeoutError => self.cc1101.command(CommandStrobe::ResetChip)?,
    //                 _ => self.cc1101.command(CommandStrobe::ResetRtcToEvent1)?,
    //             }
    //         },
    //     }

    //     let _ = self.cc1101.read_machine_state()?;

    //     Ok(())
    // }

    pub async fn test_transmit(&mut self) -> Result<(), Cc1101WrapperError> {

        // Prepare data
        // let mut data: [u8; 64] = [0; 64];
        // for (index, element) in data.iter_mut().enumerate() {
        //     *element = index as u8;
        // }
        // data[0] = 64;

        let mut data: [u8; PACKET_LENGTH_BYTES+2] = [0xCC; PACKET_LENGTH_BYTES+2];
        data[0] = PACKET_LENGTH_BYTES as u8;
        data[1] = 0;

        // Reset CC1101
        self.cc1101.command(CommandStrobe::ResetChip)?;
        Systick::delay(fugit::ExtU64::micros(2000)).await;
        let _ = self.cc1101.read_machine_state()?;

        // Flush FIFO TX
        self.cc1101.command(CommandStrobe::FlushTxFifoBuffer)?;
        let _ = self.cc1101.read_machine_state()?;

        // Calibrate
        self.cc1101.set_radio_mode(RadioMode::Calibrate).await?;
        self.cc1101.await_machine_state(MachineState::IDLE).await?;

        // Write data, start TX
        self.cc1101.write_data(&mut data)?;
        self.cc1101.read_tx_bytes()?;
        self.cc1101.set_radio_mode(RadioMode::Transmit).await?;
        Systick::delay(fugit::ExtU64::micros(5000)).await;

        // Wait for TX to finish, get result
        let result = self.cc1101.await_machine_state(MachineState::IDLE).await;

        match result {
            Ok(_) => self.cc1101.command(CommandStrobe::NoOperation)?,
            Err(e) => {
                match e {
                    Cc1101AsyncError::TimeoutError => self.cc1101.command(CommandStrobe::ResetChip)?,
                    _ => self.cc1101.command(CommandStrobe::ResetRtcToEvent1)?,
                }
            },
        }

        let _ = self.cc1101.read_machine_state()?;

        Ok(())
    }

    pub async fn test_receive_init(&mut self) -> Result<(), Cc1101WrapperError> {
        // Reset CC1101
        self.cc1101.command(CommandStrobe::ResetChip)?;
        Systick::delay(fugit::ExtU64::micros(2000)).await;
        let _ = self.cc1101.read_machine_state()?;

        // Flush FIFO RX
        self.cc1101.command(CommandStrobe::FlushRxFifoBuffer)?;
        let _ = self.cc1101.read_machine_state()?;

        // Calibrate
        self.cc1101.set_radio_mode(RadioMode::Calibrate).await?;

        Ok(())
    }

    pub async fn test_receive(&mut self, data: &mut [u8], length: &mut u8) -> Result<(), Cc1101WrapperError> {

        let mut rx_state = RxState::Waiting;
        let mut last_rxbytes = 0;

        loop {
            match rx_state {
                RxState::Waiting => {
                    Systick::delay(fugit::ExtU64::millis(5)).await;
                    let num_rxbytes = self.cc1101.read_rx_bytes()?;
                    if num_rxbytes > 0 {
                        rx_state = RxState::Receiving;
                    }
                },
                RxState::Receiving => {
                    Systick::delay(fugit::ExtU64::millis(1)).await;
                    let num_rxbytes = self.cc1101.read_rx_bytes()?;
                    if (num_rxbytes > 0) && (num_rxbytes == last_rxbytes) {
                        rx_state = RxState::Received;
                    }
                    last_rxbytes = num_rxbytes;
                    if last_rxbytes > FIFO_MAX_SIZE {
                        rx_state = RxState::Error;
                    }
                },
                RxState::Received => {
                    self.cc1101.read_data(&mut data[0..(last_rxbytes as usize)])?;
                    *length = last_rxbytes;
                    break;
                },
                RxState::Error => {
                    return Err(Cc1101WrapperError::UserInputError(last_rxbytes as usize));
                },
            }
        }

        Ok(())
    }

    pub async fn test_await_receive(&mut self, data: &mut [u8], length: &mut u8) -> Result<(), Cc1101WrapperError> {
        let timeout = fugit::ExtU64::millis(100);

        // Start RX
        self.cc1101.set_radio_mode(RadioMode::Receive).await?;

        let result = match Systick::timeout_after(timeout, self.test_receive(data, length)).await {
            Ok(result) => result,
            Err(_) => Err(Cc1101WrapperError::TimeoutError),
        };

        self.cc1101.set_radio_mode(RadioMode::Idle).await?;

        // let result = self.cc1101.await_machine_state(MachineState::IDLE).await;

        // match result {
        //     Ok(_) => self.cc1101.command(CommandStrobe::NoOperation)?,
        //     Err(e) => {
        //         match e {
        //             Cc1101AsyncError::TimeoutError => self.cc1101.command(CommandStrobe::ResetChip)?,
        //             _ => self.cc1101.command(CommandStrobe::ResetRtcToEvent1)?,
        //         }
        //     },
        // }

        let _ = self.cc1101.read_machine_state()?;

        result
    }

    pub async fn receive_packet(
        &mut self,
        dst: &mut u8,
        buffer: &mut [u8],
    ) -> Result<(u8, i16, u8), Cc1101WrapperError> {
        // self.cc1101.set_radio_mode(RadioMode::Receive)?;

        //  wait 10 ms
        Systick::delay(fugit::ExtU64::millis(10)).await;

        // let length = self.cc1101.receive(dst, buffer)?;
        let length = 0;
        let rssi = self.cc1101.get_rssi_dbm()?;
        let lqi = self.cc1101.get_lqi()?;

        Ok((length, rssi, lqi))
    }

    pub fn transmit_packet(
        &mut self,
        dst: &mut u8,
        buffer: &mut [u8],
    ) -> Result<(), Cc1101WrapperError> {
        // self.cc1101.set_radio_mode(RadioMode::Transmit)?;
        // self.cc1101.transmit(dst, buffer)?;

        Ok(())
    }

    pub fn get_hw_info(&mut self) -> Result<(u8, u8), Cc1101WrapperError> {
        Ok(self.cc1101.get_hw_info()?)
    }
}
