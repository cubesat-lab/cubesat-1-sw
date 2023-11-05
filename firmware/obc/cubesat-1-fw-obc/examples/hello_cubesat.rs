#![no_main]
#![no_std]

use cortex_m::{delay::Delay, peripheral::Peripherals as CortexMPeripherals};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f767zi::serial::SerialUartUsb;
use panic_halt as _;
use stm32f7xx_hal::{pac::Peripherals as Stm32F7Peripherals, prelude::*};

static USE_GDB: bool = false;

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
const HELLO_WORLD: &str  = "Hello world!\nI dream to be an OBC firmware for the CubeSat-1 project when I will grow up (^_^)";

#[entry]
fn main() -> ! {
    // Initialize Core Peripherals
    let cp: CortexMPeripherals = CortexMPeripherals::take().unwrap();

    // Initialize Peripheral Access Crate
    let pac = Stm32F7Peripherals::take().unwrap();

    // Set up the system clock. We want to run at 216MHz for this one.
    let rcc = pac.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    // Initialize GPIO Ports
    let gpiod = pac.GPIOD.split();

    // Example of printing with GDB debugger
    if USE_GDB {
        hprintln!("Hello World!"); // printing via debugger
    }

    // Initialize delay functionality
    let mut delay = Delay::new(cp.SYST, 216.MHz::<1, 1>().raw());

    // Initialize UART for serial communication through USB
    let mut serial = SerialUartUsb::new(pac.USART3, &clocks, gpiod.pd8, gpiod.pd9);

    // Print "CubeSat-1"
    serial.println(CUBESAT);

    delay.delay_ms(1_000);
    serial.println(" ");

    // Print "Hello World"
    serial.println(HELLO_WORLD);

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
