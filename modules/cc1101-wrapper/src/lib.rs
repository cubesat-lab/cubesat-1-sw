#![no_std]

use cc1101_async::{
    Cc1101Async, Cc1101AsyncError,
    AddressFilter, CcaMode, Modulation, NumPreambleBytes, PacketLength, RadioMode,
    SyncMode, CommandStrobe, MachineState
};

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use rtic_monotonics::systick::Systick;


// Project specific radio configurations
const FREQUENCY: u64 = 433_000_000; // 433 MHz
const MODULATION: Modulation = Modulation::BinaryFrequencyShiftKeying;
const SYNC_MODE: SyncMode = SyncMode::MatchFull(0xD201);
const PACKET_LENGTH: PacketLength = PacketLength::Fixed(8);
const NUM_PREAMBLE_BYTES: NumPreambleBytes = NumPreambleBytes::Two;
const CCA_MODE: CcaMode = CcaMode::AlwaysClear;
const CRC_ENABLE: bool = true;
const ADDRESS_FILTER: AddressFilter = AddressFilter::Device(0x3e);
const DEVIATION: u64 = 20_629;
const DATARATE: u64 = 38_383;
const BANDWIDTH: u64 = 101_562;

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

    pub fn configure_radio(&mut self) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
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

    pub async fn sandbox(&mut self) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {

        let mut data: [u8; 70] = [0; 70];
        for (index, element) in data.iter_mut().enumerate() {
            *element = index as u8;
        }
        data[0] = 64;

        self.cc1101.command(CommandStrobe::ResetChip)?;

        Systick::delay(fugit::ExtU64::micros(2000)).await;

        let _ = self.cc1101.read_machine_state()?;

        self.cc1101.command(CommandStrobe::FlushTxFifoBuffer)?;

        let _ = self.cc1101.read_machine_state()?;

        self.cc1101.write_data(&mut data)?;

        self.cc1101.command(CommandStrobe::EnableTx)?;

        self.cc1101.await_machine_state(MachineState::TX).await?;

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

    pub async fn receive_packet(
        &mut self,
        dst: &mut u8,
        buffer: &mut [u8],
    ) -> Result<(u8, i16, u8), Cc1101AsyncError<SpiE, GpioE>> {
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
    ) -> Result<(), Cc1101AsyncError<SpiE, GpioE>> {
        // self.cc1101.set_radio_mode(RadioMode::Transmit)?;
        // self.cc1101.transmit(dst, buffer)?;

        Ok(())
    }

    pub fn get_hw_info(&mut self) -> Result<(u8, u8), Cc1101AsyncError<SpiE, GpioE>> {
        self.cc1101.get_hw_info()
    }
}
