#![no_main]
#![no_std]

use core::str::from_utf8;
use cortex_m::{delay::Delay, peripheral::Peripherals as CortexMPeripherals};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f767zi::{serial::SerialUartUsb, temp::TemperatureSensor, uid::McuUid};
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
    let gpiod = pac.GPIOD.split();

    // Example of printing with GDB debugger
    if USE_GDB {
        hprintln!("Hello World!"); // printing via debugger
    }

    // Initialize delay functionality
    let mut delay = Delay::new(cp.SYST, 216.MHz::<1, 1>().raw());

    // Initialize UART for serial communication through USB
    let mut serial = SerialUartUsb::new(pac.USART3, &clocks, gpiod.pd8, gpiod.pd9);

    // Initialize Temperature Sensor
    let mut temp_sensor = TemperatureSensor::new(pac.ADC_COMMON, pac.ADC1, &mut rcc.apb2, &clocks);

    // Initialize MCU UID
    let mcu_uid = McuUid::new();

    // Send MCU UID data to serial
    serial.formatln(format_args!("MCU UID:"));
    serial.formatln(format_args!("    wafer x pos:  {}", mcu_uid.x));
    serial.formatln(format_args!("    wafer y pos:  {}", mcu_uid.y));
    serial.formatln(format_args!("    wafer number: {}", mcu_uid.waf_num));
    serial.formatln(format_args!(
        "    lot number:   {}",
        from_utf8(&mcu_uid.lot_num).unwrap()
    ));
    serial.println(" ");

    // Cyclic part
    loop {
        // Send temperature data to serial
        serial.formatln(format_args!(
            "Temp: {:.2} °C",
            temp_sensor.read_temperature()
        ));
        delay.delay_ms(1_000);
    }
}
