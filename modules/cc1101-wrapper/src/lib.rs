#![no_std]

pub use cc1101::{
    AddressFilter, Cc1101, CcaMode, Error, MachineState, ModulationFormat, NumPreamble,
    PacketLength, RadioMode, SyncMode, UserError, FIFO_SIZE_MAX,
};
use embedded_hal::spi::SpiDevice;
use fugit::Duration;
use rtic_monotonics::systick::Systick;

enum RxState {
    Waiting,
    Receiving,
    Received,
    // Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cc1101WrapperError {
    /// Receive buffer is empty
    RxBufferEmpty,
    /// Transmit buffer has unsent data
    TxBufferBusy,
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
    UserInputError(UserError),
    /// Platform-dependent SPI-errors, such as IO errors.
    Spi,
}

impl<SpiE> From<Error<SpiE>> for Cc1101WrapperError {
    fn from(e: Error<SpiE>) -> Self {
        match e {
            Error::TxUnderflow => Cc1101WrapperError::TxUnderflow,
            Error::RxOverflow => Cc1101WrapperError::RxOverflow,
            Error::CrcMismatch => Cc1101WrapperError::CrcMismatch,
            Error::InvalidState(value) => Cc1101WrapperError::InvalidState(value),
            Error::UserInputError(value) => Cc1101WrapperError::UserInputError(value),
            Error::Spi(_) => Cc1101WrapperError::Spi,
        }
    }
}

// Radio configuration
pub struct Cc1101RfConfig {
    pub frequency: u64,
    pub bandwidth: u64,
    pub deviation: u64,
    pub datarate: u64,
    pub modulation: ModulationFormat,
    pub num_preamble: NumPreamble,
    pub sync_mode: SyncMode,
    pub packet_length: PacketLength,
    pub address_filter: AddressFilter,
    pub crc_enable: bool,
    pub white_data: bool,
    pub cca_mode: CcaMode,
}

impl Default for Cc1101RfConfig {
    fn default() -> Self {
        Self {
            frequency: 0,
            bandwidth: 0,
            deviation: 0,
            datarate: 0,
            modulation: ModulationFormat::BinaryFrequencyShiftKeying,
            num_preamble: NumPreamble::Two,
            sync_mode: SyncMode::Disabled,
            packet_length: PacketLength::Fixed(0),
            address_filter: AddressFilter::Disabled,
            crc_enable: false,
            white_data: false,
            cca_mode: CcaMode::CciAlways,
        }
    }
}

// #[derive(Default, Clone, Copy)]
struct DataBuffer {
    data: [u8; FIFO_SIZE_MAX as usize],
    length: u8,
    address: u8,
    ready: bool,
}

impl Default for DataBuffer {
    fn default() -> Self {
        Self {
            data: [0; FIFO_SIZE_MAX as usize],
            length: 0,
            address: 0,
            ready: false,
        }
    }
}

pub struct Cc1101Wrapper<SPI> {
    cc1101: Cc1101<SPI>,
    rf_config: Cc1101RfConfig,
    rx_data: DataBuffer,
    tx_data: DataBuffer,
    last_error: Option<Cc1101WrapperError>,
    error_count: u32,
}

impl<SPI, SpiE> Cc1101Wrapper<SPI>
where
    SPI: SpiDevice<u8, Error = SpiE>,
{
    /// Instantiate the CC1101 Wrapper module and the underlying CC1101 driver.
    pub fn new(spi: SPI) -> Self {
        let cc1101 = Cc1101::new(spi);

        match cc1101 {
            Ok(cc1101) => Cc1101Wrapper {
                cc1101,
                rf_config: Cc1101RfConfig::default(),
                rx_data: DataBuffer::default(),
                tx_data: DataBuffer::default(),
                last_error: None,
                error_count: 0,
            },
            Err(_error) => panic!("Error initializing CC1101"),
        }
    }

    /// Initialize RF Transceiver's configuration specific to the project.
    pub fn init_config(&mut self, rf_config: Cc1101RfConfig) -> Result<(), Cc1101WrapperError> {
        // Save RF config
        self.rf_config = rf_config;

        // TODO: Temp usage of set_defaults function
        self.cc1101.set_defaults()?;

        // Reset CC1101
        // self.cc1101.reset_chip()?;

        self.cc1101.set_frequency(self.rf_config.frequency)?;
        self.cc1101.set_chanbw(self.rf_config.bandwidth)?;
        self.cc1101.set_deviation(self.rf_config.deviation)?;
        self.cc1101.set_data_rate(self.rf_config.datarate)?;
        self.cc1101
            .set_modulation_format(self.rf_config.modulation)?;
        self.cc1101.set_num_preamble(self.rf_config.num_preamble)?;
        self.cc1101.set_sync_mode(self.rf_config.sync_mode)?;
        self.cc1101
            .set_packet_length(self.rf_config.packet_length)?;
        self.cc1101
            .set_address_filter(self.rf_config.address_filter)?;
        self.cc1101.crc_enable(self.rf_config.crc_enable)?;
        self.cc1101.white_data_enable(self.rf_config.white_data)?;
        self.cc1101.set_cca_mode(self.rf_config.cca_mode)?;

        Ok(())
    }

    /// Get HW partnum and version info
    pub fn get_hw_info(&mut self) -> Result<(u8, u8), Cc1101WrapperError> {
        Ok(self.cc1101.get_hw_info()?)
    }

    /// CC1101 main function which processes the RF operations and shall be called cyclically.
    pub async fn main(&mut self) {

        // Dummy check of Machine State
        let result = self.cc1101.get_machine_state();
        let _ = self.process_result(result);

        // Process RF receiving
        self.process_receive().await;

        // Process RF transmitting
        self.process_transmit().await;
    }

    pub fn is_data_received(&mut self) -> bool {
        self.rx_data.ready
    }

    pub fn read_data(
        &mut self,
        data: &mut [u8],
        length: &mut u8,
        address: &mut u8,
    ) -> Result<(), Cc1101WrapperError> {
        if self.rx_data.ready {
            // Copy data from internal Rx Buffer
            *length = self.rx_data.length;
            *address = self.rx_data.address;
            data[..(self.rx_data.length as usize)]
                .copy_from_slice(&self.rx_data.data[..(self.rx_data.length as usize)]);

            self.rx_data.ready = false;
        } else {
            // Rx buffer is empty. No data was received on RF
            return Err(Cc1101WrapperError::RxBufferEmpty);
        }

        Ok(())
    }

    pub fn write_data(&mut self, data: &[u8], address: u8) -> Result<(), Cc1101WrapperError> {
        if !self.tx_data.ready {
            // Copy data into internal Tx Buffer
            self.tx_data.length = data.len() as u8;
            self.tx_data.address = address;
            self.tx_data.data[..(self.tx_data.length as usize)].copy_from_slice(data);
            self.tx_data.ready = true;
        } else {
            // Tx buffer is busy holding previous data
            return Err(Cc1101WrapperError::TxBufferBusy);
        }

        Ok(())
    }

    pub fn read_last_error(&mut self) -> (Option<Cc1101WrapperError>, u32) {
        let last_error = self.last_error;
        let error_count = self.error_count;
        self.last_error = None;
        self.error_count = 0;

        (last_error, error_count)
    }

    // ---------------------------------------------------------------------------------

    async fn process_receive(&mut self) {
        let timeout = fugit::ExtU64::millis(10);

        // Calibrate
        let result = self.set_radio_mode(RadioMode::Calibrate, timeout).await;
        self.process_native_result(result);
        let result = self.await_machine_state(MachineState::IDLE, timeout).await;
        self.process_native_result(result);

        // Systick::delay(10.millis().into()).await;

        // Flush FIFO RX
        let result = self.cc1101.flush_rx_fifo_buffer();
        self.process_result(result);
        let result = self.cc1101.get_machine_state();
        let _ = self.process_result(result);

        // Start Rx
        let result = self.set_radio_mode(RadioMode::Receive, timeout).await;
        self.process_native_result(result);

        match Systick::timeout_after(fugit::ExtU64::millis(100), self.receive()).await {
            Ok(result) => match result {
                Ok(state) => match state {
                    RxState::Received => {
                        self.rx_data.ready = true;
                    }
                    _ => { /* Unreachable statement */ }
                },
                Err(error) => {
                    // Store error
                    self.last_error = Some(error);
                    self.error_count += 1;
                }
            },
            Err(_) => { /* Timeout Error */ }
        };

        let result = self.set_radio_mode(RadioMode::Idle, timeout).await;
        self.process_native_result(result);
        let result = self.cc1101.get_machine_state();
        let _ = self.process_result(result);
    }

    async fn process_transmit(&mut self) {
        let timeout = fugit::ExtU64::millis(10);

        // Check if data is available for write
        if self.tx_data.ready {

            let result = self.cc1101.get_machine_state();
            let _ = self.process_result(result);

            // Calibrate
            let result = self.set_radio_mode(RadioMode::Calibrate, timeout).await;
            self.process_native_result(result);
            let result = self.await_machine_state(MachineState::IDLE, timeout).await;
            self.process_native_result(result);

            // Flush FIFO Tx
            let result = self.cc1101.flush_tx_fifo_buffer();
            self.process_result(result);
            let result = self.cc1101.get_machine_state();
            let _ = self.process_result(result);

            let mut length: Option<u8> = Some(self.tx_data.length + 1); // Plus address byte
            let mut address: Option<u8> = Some(self.tx_data.address);

            // Write data
            let result = self.cc1101.write_data(
                &mut length,
                &mut address,
                &mut self.tx_data.data[..(self.tx_data.length as usize)],
            );
            self.process_result(result);
            let result = self.cc1101.get_tx_bytes();
            let _ = self.process_result(result);

            // Start Tx
            let result = self.set_radio_mode(RadioMode::Transmit, timeout).await;
            self.process_native_result(result);
            Systick::delay(fugit::ExtU64::millis(5)).await;

            // Wait for Tx to finish and get the result
            let result = self.await_machine_state(MachineState::IDLE, timeout).await;
            self.process_native_result(result);

            // match result {
            //     Ok(_) => {}
            //     Err(_) => {
            //         self.process_result(result);
            //     }
            // }

            let result = self.cc1101.get_machine_state();
            let _ = self.process_result(result);

            self.tx_data.ready = false;
        }
    }

    async fn receive(&mut self) -> Result<RxState, Cc1101WrapperError> {
        let mut rx_state = RxState::Waiting;
        let mut last_rxbytes = 0;

        loop {
            match rx_state {
                RxState::Waiting => {
                    Systick::delay(fugit::ExtU64::millis(5)).await;
                    let num_rxbytes = self.cc1101.get_rx_bytes()?;
                    if num_rxbytes > 0 {
                        rx_state = RxState::Receiving;
                    }
                }
                RxState::Receiving => {
                    Systick::delay(fugit::ExtU64::millis(1)).await;
                    let num_rxbytes = self.cc1101.get_rx_bytes()?;
                    if (num_rxbytes > 0) && (num_rxbytes == last_rxbytes) {
                        rx_state = RxState::Received;
                    }
                    last_rxbytes = num_rxbytes;
                    if last_rxbytes > FIFO_SIZE_MAX {
                        // TODO: Check how to treat this case in a reliable way
                        last_rxbytes = FIFO_SIZE_MAX;
                        // rx_state = RxState::Error;
                    }
                }
                RxState::Received => {
                    let mut length: Option<u8> = Some(0);
                    let mut address: Option<u8> = Some(0);

                    self.cc1101.read_data(
                        &mut length,
                        &mut address,
                        &mut self.rx_data.data[0..(last_rxbytes as usize)],
                    )?;

                    // Store received data
                    // self.rx_data.length = last_rxbytes; // Fixed Length?
                    self.rx_data.length = length.unwrap() - 1; // Minus address byte
                    self.rx_data.address = address.unwrap();
                    break;
                }
                // RxState::Error => {
                //     // return Err(Cc1101WrapperError::UserInputError(last_rxbytes as usize));
                // }
            }
        }

        Ok(rx_state)
    }

    fn process_result<R>(&mut self, result: Result<R, Error<SpiE>>) -> Option<R> {
        match result {
            Ok(value) => {
                // Return the value
                Some(value)
            }
            Err(error) => {
                // Store error
                self.last_error = Some(error.into());
                self.error_count += 1;
                None
            }
        }
    }

    fn process_native_result<R>(&mut self, result: Result<R, Cc1101WrapperError>) -> Option<R> {
        match result {
            Ok(value) => {
                // Return the value
                Some(value)
            }
            Err(error) => {
                // Store error
                self.last_error = Some(error);
                self.error_count += 1;
                None
            }
        }
    }

    // ---------------------------------------------------------------------------------

    async fn check_machine_state(
        &mut self,
        target_state: MachineState,
    ) -> Result<(), Cc1101WrapperError> {
        let delay = fugit::ExtU64::micros(1000);
        loop {
            let machine_state = self.cc1101.get_machine_state()?;

            if machine_state == target_state {
                /* Successful scenario */
                return Ok(());
            }

            /* Error scenarios */
            if machine_state == MachineState::RXFIFO_OVERFLOW {
                return Err(Cc1101WrapperError::RxOverflow);
            } else if machine_state == MachineState::TXFIFO_UNDERFLOW {
                return Err(Cc1101WrapperError::TxUnderflow);
            } else {
                /* Ignore other states */
            }

            Systick::delay(delay).await;
        }
    }

    async fn await_machine_state(
        &mut self,
        target_state: MachineState,
        timeout: Duration<u64, 1, 1000>,
    ) -> Result<(), Cc1101WrapperError> {
        match Systick::timeout_after(timeout, self.check_machine_state(target_state)).await {
            Ok(result) => result,
            Err(_) => Err(Cc1101WrapperError::TimeoutError),
        }
    }

    /// Set Radio Mode.
    async fn set_radio_mode(
        &mut self,
        radio_mode: RadioMode,
        timeout: Duration<u64, 1, 1000>,
    ) -> Result<(), Cc1101WrapperError> {
        // Set "Idle" mode before going into any other mode
        self.cc1101.exit_rx_tx()?;
        self.await_machine_state(MachineState::IDLE, timeout)
            .await?;

        match radio_mode {
            RadioMode::Idle => {
                // Do nothing
            }
            RadioMode::Sleep => {
                self.cc1101.enter_power_down_mode()?;
            }
            RadioMode::Calibrate => {
                self.cc1101.cal_freq_synth_and_turn_off()?;
                self.await_machine_state(MachineState::MANCAL, timeout)
                    .await?;
            }
            RadioMode::Transmit => {
                self.cc1101.enable_tx()?;
                self.await_machine_state(MachineState::TX, timeout).await?;
            }
            RadioMode::Receive => {
                self.cc1101.enable_rx()?;
                self.await_machine_state(MachineState::RX, timeout).await?;
            }
        };
        Ok(())
    }

    // pub async fn test_transmit_old(&mut self) -> Result<(), Cc1101WrapperError> {

    //     // Prepare data
    //     let mut data: [u8; 64] = [0; 64];
    //     for (index, element) in data.iter_mut().enumerate() {
    //         *element = index as u8;
    //     }
    //     // data[0] = 64;

    //     // Reset CC1101
    //     self.cc1101.command(CommandStrobe::ResetChip)?;
    //     Systick::delay(fugit::ExtU64::micros(2000)).await;
    //     let _ = self.cc1101.get_machine_state()?;

    //     // Flush FIFO TX
    //     self.cc1101.command(CommandStrobe::FlushTxFifoBuffer)?;
    //     let _ = self.cc1101.get_machine_state()?;

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
    //                 Cc1101WrapperError::TimeoutError => self.cc1101.command(CommandStrobe::ResetChip)?,
    //                 _ => self.cc1101.command(CommandStrobe::ResetRtcToEvent1)?,
    //             }
    //         },
    //     }

    //     let _ = self.cc1101.get_machine_state()?;

    //     Ok(())
    // }

    // pub async fn test_transmit(&mut self) -> Result<(), Cc1101WrapperError> {
    //     // Prepare data
    //     // let mut data: [u8; 64] = [0; 64];
    //     // for (index, element) in data.iter_mut().enumerate() {
    //     //     *element = index as u8;
    //     // }
    //     // data[0] = 64;

    //     let mut data: [u8; PACKET_LENGTH_BYTES + 2] = [0xCC; PACKET_LENGTH_BYTES + 2];
    //     data[0] = PACKET_LENGTH_BYTES as u8;
    //     data[1] = 0;

    //     // Reset CC1101
    //     self.cc1101.command(CommandStrobe::ResetChip)?;
    //     Systick::delay(fugit::ExtU64::micros(2000)).await;
    //     let _ = self.cc1101.get_machine_state()?;

    //     // Flush FIFO TX
    //     self.cc1101.command(CommandStrobe::FlushTxFifoBuffer)?;
    //     let _ = self.cc1101.get_machine_state()?;

    //     // Calibrate
    //     self.cc1101.set_radio_mode(RadioMode::Calibrate).await?;
    //     self.cc1101.await_machine_state(MachineState::IDLE).await?;

    //     // Write data, start TX
    //     self.cc1101.write_data(&mut data)?;
    //     self.cc1101.get_tx_bytes()?;
    //     self.cc1101.set_radio_mode(RadioMode::Transmit).await?;
    //     Systick::delay(fugit::ExtU64::micros(5000)).await;

    //     // Wait for TX to finish, get result
    //     let result = self.cc1101.await_machine_state(MachineState::IDLE).await;

    //     match result {
    //         Ok(_) => self.cc1101.command(CommandStrobe::NoOperation)?,
    //         Err(e) => match e {
    //             Cc1101WrapperError::TimeoutError => self.cc1101.command(CommandStrobe::ResetChip)?,
    //             _ => self.cc1101.command(CommandStrobe::ResetRtcToEvent1)?,
    //         },
    //     }

    //     let _ = self.cc1101.get_machine_state()?;

    //     Ok(())
    // }
}
