#![no_main]
#![no_std]

use cc1101_wrapper::Cc1101Wrapper;
use cortex_m::{delay::Delay, peripheral::Peripherals as CortexMPeripherals};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f767zi::{
    led::{LedBlue, LedGreen, LedParameters, LedRed},
    serial::{SerialParameters, SerialUartUsb},
    spi::{SpiMaster3, SpiMaster4},
};
use panic_halt as _;
use stm32f7xx_hal::{pac::Peripherals as Stm32F7Peripherals, prelude::*};

static USE_GDB: bool = false;

#[entry]
fn main() -> ! {
    // Initialize Core Peripherals
    let cp: CortexMPeripherals = CortexMPeripherals::take().unwrap();

    // Initialize Peripheral Access Crate
    let pac = Stm32F7Peripherals::take().unwrap();

    // Set up the system clock. We want to run at 216MHz for this one.
    let mut rcc = pac.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    // Initialize GPIO Ports
    let gpiob = pac.GPIOB.split();
    let gpioc = pac.GPIOC.split();
    let gpiod = pac.GPIOD.split();
    let gpioe = pac.GPIOE.split();

    // Example of printing with GDB debugger
    if USE_GDB {
        hprintln!("Hello World!"); // printing via debugger
    }

    // Initialize delay functionality
    let mut delay = Delay::new(cp.SYST, 216.MHz::<1, 1>().raw());

    // Initialize LEDs
    let mut led_green = LedGreen::new(LedParameters { pin: gpiob.pb0 });
    let mut led_blue = LedBlue::new(LedParameters { pin: gpiob.pb7 });
    let mut led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

    // Initialize UART for serial communication through USB
    let serial_parameters = SerialParameters {
        uart: pac.USART3,
        clocks: &clocks,
        pin_tx: gpiod.pd8,
        pin_rx: gpiod.pd9,
    };
    let mut serial = SerialUartUsb::new(serial_parameters);

    // Initialize SPI3
    let spi_3 = SpiMaster3::new(
        pac.SPI3,
        &clocks,
        &mut rcc.apb1,
        gpioc.pc9,
        gpioc.pc10,
        gpioc.pc11,
        gpioc.pc12,
    );

    // Initialize SPI4
    let spi_4 = SpiMaster4::new(
        pac.SPI4,
        &clocks,
        &mut rcc.apb2,
        gpioe.pe4,
        gpioe.pe2,
        gpioe.pe5,
        gpioe.pe6,
    );

    // Initialize CC1101 Wrapper - RF Device 1
    let mut cc1101_wrp_1 = Cc1101Wrapper::new(spi_3.spi, spi_3.cs);
    cc1101_wrp_1.configure_radio().unwrap();

    // Initialize CC1101 Wrapper - RF Device 2
    let mut cc1101_wrp_2 = Cc1101Wrapper::new(spi_4.spi, spi_4.cs);
    cc1101_wrp_2.configure_radio().unwrap();

    // Get HW Info from both RF Devices
    let (partnum_1, version_1) = cc1101_wrp_1.get_hw_info().unwrap();
    let (partnum_2, version_2) = cc1101_wrp_2.get_hw_info().unwrap();

    // Send HW Info to serial
    serial.formatln(format_args!(
        "Device 1: partnum = {}, version = {}",
        partnum_1, version_1
    ));
    serial.formatln(format_args!(
        "Device 2: partnum = {}, version = {}",
        partnum_2, version_2
    ));

    // Cyclic part
    loop {
        // Test Code
        led_green.toggle();
        led_blue.toggle();
        led_red.toggle();
        delay.delay_us(1_000_000);

        // TODO: This code below isn't functional - fix CC1101 driver and CC1101 Wrapper implementation
        // Transmit the packet
        let mut dst = 0u8;
        let mut buffer = [0, 1, 2, 3, 4, 5, 6, 7];
        let _result = cc1101_wrp_1.transmit_packet(&mut dst, &mut buffer).unwrap();

        delay.delay_us(10_000);

        // Attempt to read data on the radio
        // If read is succesful, send the packet via UART
        let mut dst = 0u8;
        let mut buffer = [0u8; 8];
        if let Ok(_result) = cc1101_wrp_2.receive_packet(&mut dst, &mut buffer) {
            serial.formatln(format_args!("Message: {:?}", buffer));
        }
    }
}
