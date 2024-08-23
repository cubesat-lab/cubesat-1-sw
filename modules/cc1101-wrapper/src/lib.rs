#![no_std]

pub use cc1101::{
    AddressFilter, AutoCalibration, Cc1101, CcaMode, Error, GdoCfg, MachineState, ModulationFormat,
    NumPreamble, PacketLength, RadioMode, SyncMode, UserError, FIFO_SIZE_MAX,
};
use embedded_hal::{digital::PinState, spi::SpiDevice};
use fugit::{Duration, Instant};
use rtic_monotonics::{systick::Systick, Monotonic};

pub const PACKET_LENGTH: u8 = FIFO_SIZE_MAX;

enum RxState {
    Waiting,
    Receiving,
    Received,
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cc1101WrapperError {
    /// Monitoring error
    MonitoringError,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cc1101RxMode {
    Polling,
    Interrupt,
}

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
    rx_mode: Cc1101RxMode,
    rx_init: bool,
    rx_int_pending: bool,
    rx_data: DataBuffer,
    tx_data: DataBuffer,
    last_rx_rssi: i16,
    last_rx_lqi: u8,
    timestamp_monitor: Instant<u64, 1, 1000>,
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
                rx_mode: Cc1101RxMode::Polling,
                rx_init: false,
                rx_int_pending: false,
                rx_data: DataBuffer::default(),
                tx_data: DataBuffer::default(),
                last_rx_rssi: 0,
                last_rx_lqi: 0,
                timestamp_monitor: Systick::now(),
                last_error: None,
                error_count: 0,
            },
            Err(_error) => panic!("Error initializing CC1101"),
        }
    }

    /// Initialize RF Transceiver's configuration specific to the project.
    pub fn init_config(&mut self) -> Result<(), Cc1101WrapperError> {
        // Reset CC1101
        self.cc1101.reset_chip()?;

        // Set project specific radio configuration
        self.cc1101.set_frequency(433_000_000)?; // 433 MHz
        self.cc1101.set_freq_if(203_125)?;
        self.cc1101.set_chanbw(101_562)?;
        self.cc1101.set_deviation(20_629)?;
        self.cc1101.set_data_rate(38_383)?;
        self.cc1101
            .set_modulation_format(ModulationFormat::BinaryFrequencyShiftKeying)?;
        self.cc1101.set_num_preamble(NumPreamble::Eight)?;
        self.cc1101.set_sync_mode(SyncMode::MatchFull(0xCAFE))?;
        self.cc1101
            .set_packet_length(PacketLength::Fixed(PACKET_LENGTH))?;
        self.cc1101.set_address_filter(AddressFilter::Disabled)?;
        self.cc1101.crc_enable(true)?;
        self.cc1101.crc_autoflush_enable(true)?;
        self.cc1101.append_status_enable(false)?;
        self.cc1101.white_data_enable(false)?;
        self.cc1101.set_cca_mode(CcaMode::CciAlways)?;
        self.cc1101.set_autocalibration(AutoCalibration::FromIdle)?;
        self.cc1101.set_gdo2_active_state(PinState::Low)?;
        self.cc1101.set_gdo2_config(GdoCfg::CRC_OK)?;

        // Set Rx mode
        self.rx_mode = Cc1101RxMode::Interrupt;

        Ok(())
    }

    /// Get HW partnum and version info
    pub fn get_hw_info(&mut self) -> Result<(u8, u8), Cc1101WrapperError> {
        Ok(self.cc1101.get_hw_info()?)
    }

    /// CC1101 main function which processes the RF operations and shall be called cyclically.
    pub async fn main(&mut self) {
        // Initialization activity
        if !self.rx_init {
            self.rx_init = true;

            // Start Rx state at the first run
            self.start_rx_state().await;
        }

        // Process RF receiving
        match self.rx_mode {
            Cc1101RxMode::Polling => {
                self.process_receive_polling().await;
            }
            Cc1101RxMode::Interrupt => {
                self.process_receive_interrupt().await;
            }
        };

        // Process RF transmitting
        self.process_transmit().await;

        // Transciever monitoring
        self.monitor().await;
    }

    pub fn signal_rx_int(&mut self) {
        self.rx_int_pending = true;
    }

    pub fn is_data_received(&mut self) -> bool {
        self.rx_data.ready
    }

    pub fn read_data(
        &mut self,
        data: &mut [u8],
        rssi: &mut i16,
        lqi: &mut u8,
    ) -> Result<(), Cc1101WrapperError> {
        if self.rx_data.ready {
            // Copy data from internal Rx Buffer
            *rssi = self.last_rx_rssi;
            *lqi = self.last_rx_lqi;
            data[..(self.rx_data.length as usize)]
                .copy_from_slice(&self.rx_data.data[..(self.rx_data.length as usize)]);

            self.rx_data.ready = false;
        } else {
            // Rx buffer is empty. No data was received on RF
            return Err(Cc1101WrapperError::RxBufferEmpty);
        }

        Ok(())
    }

    pub fn write_data(&mut self, data: &[u8]) -> Result<(), Cc1101WrapperError> {
        if !self.tx_data.ready {
            // Copy data into internal Tx Buffer
            self.tx_data.length = data.len() as u8;
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

    async fn start_idle_state(&mut self) {
        let timeout = fugit::ExtU64::millis(10);
        let result = self.set_radio_mode(RadioMode::Idle, timeout).await;
        self.process_native_result(result);
    }

    async fn start_rx_state(&mut self) {
        let timeout = fugit::ExtU64::millis(10);
        let result = self.set_radio_mode(RadioMode::Receive, timeout).await;
        self.process_native_result(result);
    }

    async fn process_receive_polling(&mut self) {
        // Flush FIFO RX
        let result = self.cc1101.flush_rx_fifo_buffer();
        self.process_result(result);
        let result = self.cc1101.get_machine_state();
        let _ = self.process_result(result);

        // Start Rx
        self.start_rx_state().await;

        match Systick::timeout_after(fugit::ExtU64::millis(100), self.receive_polling()).await {
            Ok(result) => match result {
                Ok(state) => match state {
                    RxState::Received => {
                        self.rx_data.ready = true;
                    }
                    _ => { /* Unreachable statement */ }
                },
                Err(error) => {
                    self.store_error(error);
                }
            },
            Err(_) => { /* Timeout Error */ }
        };

        // Start Idle state
        self.start_idle_state().await;
    }

    async fn process_receive_interrupt(&mut self) {
        // Check if Rx interrupt is pending
        if self.rx_int_pending {
            self.rx_int_pending = false;

            // Start Idle state
            self.start_idle_state().await;

            // Receive data
            let result = self.receive_interrupt();
            self.process_native_result(result);

            // Restart Rx state
            self.start_rx_state().await;
        }
    }

    async fn process_transmit(&mut self) {
        let timeout = fugit::ExtU64::millis(10);

        // Check if data is available for write
        if self.tx_data.ready {
            if self.rx_mode == Cc1101RxMode::Interrupt {
                // Start Idle state
                self.start_idle_state().await;

                // Flush FIFO RX
                let result = self.cc1101.flush_rx_fifo_buffer();
                self.process_result(result);
                let result = self.cc1101.get_machine_state();
                let _ = self.process_result(result);
            }

            // Flush FIFO Tx
            let result = self.cc1101.flush_tx_fifo_buffer();
            self.process_result(result);
            let result = self.cc1101.get_machine_state();
            let _ = self.process_result(result);

            // let mut length: Option<u8> = Some(self.tx_data.length + 1); // Plus address byte
            let mut length: Option<u8> = None;
            // let mut address: Option<u8> = Some(self.tx_data.address);
            let mut address: Option<u8> = None;

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

            self.tx_data.ready = false;

            if self.rx_mode == Cc1101RxMode::Interrupt {
                // Restart Rx state
                self.start_rx_state().await;
            }
        }
    }

    async fn monitor(&mut self) {
        let period: Duration<u64, 1, 1000> = fugit::ExtU64::millis(1000);
        let timestamp_now = Systick::now();

        if (timestamp_now - self.timestamp_monitor) > period {
            self.timestamp_monitor = timestamp_now;

            let result = self.cc1101.get_machine_state();

            match self.process_result(result) {
                Some(state) => {
                    if state != MachineState::RX {
                        self.store_error(Cc1101WrapperError::MonitoringError);

                        // Restart Rx state
                        self.start_rx_state().await;
                    }
                }
                None => {
                    // Start Idle state
                    self.start_idle_state().await;

                    // Restart Rx state
                    self.start_rx_state().await;
                }
            }
        }
    }

    async fn receive_polling(&mut self) -> Result<RxState, Cc1101WrapperError> {
        let mut rx_state = RxState::Waiting;
        let mut last_rxbytes = 0;

        loop {
            match rx_state {
                RxState::Waiting => {
                    Systick::delay(fugit::ExtU64::millis(5)).await;

                    let packet_status = self.cc1101.get_packet_status()?;
                    if packet_status.sof_delimiter {
                        rx_state = RxState::Receiving;
                    }
                }
                RxState::Receiving => {
                    Systick::delay(fugit::ExtU64::millis(1)).await;

                    let num_rxbytes = self.cc1101.get_rx_bytes()?;
                    if (num_rxbytes > 0) && (num_rxbytes == last_rxbytes) {
                        let packet_status = self.cc1101.get_packet_status()?;
                        if packet_status.crc_ok {
                            rx_state = RxState::Received;
                        } else {
                            rx_state = RxState::Error;
                        }
                    }
                    last_rxbytes = num_rxbytes;
                    if last_rxbytes > FIFO_SIZE_MAX {
                        // TODO: Check how to treat this case in a reliable way
                        last_rxbytes = FIFO_SIZE_MAX;
                        // rx_state = RxState::Error;
                    }
                }
                RxState::Received => {
                    let mut length: Option<u8> = None;
                    let mut address: Option<u8> = None;
                    let mut rssi: Option<i16> = None;
                    let mut lqi: Option<u8> = None;

                    self.cc1101.read_data(
                        &mut length,
                        &mut address,
                        &mut rssi,
                        &mut lqi,
                        &mut self.rx_data.data[0..(last_rxbytes as usize)],
                    )?;

                    // Store received data
                    self.rx_data.length = last_rxbytes; // Fixed Length?

                    // self.rx_data.length = length.unwrap() - 1; // Minus address byte
                    // self.rx_data.address = address.unwrap();
                    self.rx_data.address = 0;
                    self.last_rx_rssi = self.cc1101.get_rssi_dbm()?;
                    self.last_rx_lqi = self.cc1101.get_lqi()?;
                    break;
                }
                RxState::Error => {
                    return Err(Cc1101WrapperError::CrcMismatch);
                }
            }
        }

        Ok(rx_state)
    }

    fn receive_interrupt(&mut self) -> Result<(), Cc1101WrapperError> {
        let mut length: Option<u8> = None;
        let mut address: Option<u8> = None;
        let mut rssi: Option<i16> = None;
        let mut lqi: Option<u8> = None;

        let rxbytes = self.cc1101.get_rx_bytes()?;
        let packet_status = self.cc1101.get_packet_status()?;

        if packet_status.crc_ok {
            self.cc1101.read_data(
                &mut length,
                &mut address,
                &mut rssi,
                &mut lqi,
                &mut self.rx_data.data[0..(rxbytes as usize)],
            )?;

            // Store received data
            self.rx_data.ready = true;
            self.rx_data.length = rxbytes;
            self.rx_data.address = 0;

            self.last_rx_rssi = self.cc1101.get_rssi_dbm()?;
            self.last_rx_lqi = self.cc1101.get_lqi()?;
        } else {
            return Err(Cc1101WrapperError::CrcMismatch);
        }

        Ok(())
    }

    /// Store error
    fn store_error(&mut self, error: Cc1101WrapperError) {
        self.last_error = Some(error);
        self.error_count += 1;
    }

    fn process_result<R>(&mut self, result: Result<R, Error<SpiE>>) -> Option<R> {
        match result {
            Ok(value) => {
                // Return the value
                Some(value)
            }
            Err(error) => {
                self.store_error(error.into());
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
                self.store_error(error);
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
}
