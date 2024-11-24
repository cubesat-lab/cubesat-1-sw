#![no_main]
#![no_std]

use cortex_m::{delay::Delay, peripheral::Peripherals as CortexMPeripherals};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f446re::serial::{SerialParameters, SerialUartUsb};
use panic_halt as _;
use stm32f4xx_hal::{pac::Peripherals as Stm32F4Peripherals, prelude::*};

const USE_GDB: bool = false;

// Initialize string literals
const CUBESAT: &str = "
      //=============//|
     //             //||
    //=============// ||
    ||             || ||
    ||             || ||
    || [CubeSat-1] || ||
    ||             || ||
    ||             ||//
    ||=============||/
";
const HELLO: &str = "Hello from NUCLEO-F446RE!";

#[entry]
fn main() -> ! {
    // Initialize Core Peripherals
    let cp: CortexMPeripherals = CortexMPeripherals::take().unwrap();

    // Initialize Peripheral Access Crate
    let pac = Stm32F4Peripherals::take().unwrap();

    // Set up the system clock. We want to run at 180MHz for this one.
    let rcc = pac.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(180.MHz()).freeze();

    // Initialize GPIO Ports
    let gpioa = pac.GPIOA.split();

    // Example of printing with GDB debugger
    if USE_GDB {
        hprintln!("Hello World!"); // printing via debugger
    }

    // Initialize delay functionality
    let mut delay = Delay::new(cp.SYST, 180.MHz::<1, 1>().raw());

    // Initialize UART for serial communication through USB
    let serial_parameters = SerialParameters {
        uart: pac.USART2,
        clocks: &clocks,
        pin_tx: gpioa.pa2.into_alternate(),
        pin_rx: gpioa.pa3.into_alternate(),
    };
    let mut serial = SerialUartUsb::new(serial_parameters);

    // Print "CubeSat-1"
    serial.println(CUBESAT);

    delay.delay_ms(1_000);
    serial.println(" ");

    // Print "Hello"
    serial.println(HELLO);

    delay.delay_ms(1_000);
    serial.println(" ");
    serial.println(" ");

    let mut counter_s: i64 = 0;

    serial.println("Starting boredom counter...");
    serial.println(" ");

    // Cyclic part
    loop {
        // Elapsed time counter
        counter_s += 1;
        serial.formatln(format_args!("ET: {} s", counter_s));
        delay.delay_ms(1_000);
    }
}
