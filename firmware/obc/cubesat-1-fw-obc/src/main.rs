#![no_main]
#![no_std]

use cc1101_wrapper::Cc1101Wrapper;
use core::cell::{Cell, RefCell};
use cortex_m::{
    delay::{self, Delay},
    interrupt::{free, Mutex},
    peripheral::{Peripherals as CortexMPeripherals, NVIC},
};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f767zi::{
    button::Button,
    led::{LedBlue, LedGreen, LedRed},
    serial::SerialUartUsb,
    spi::SpiMaster3,
    temp::TemperatureSensor,
    uid::McuUid,
};
use panic_halt as _;
use stm32f7xx_hal::{
    gpio::{Alternate, Edge, Output, Pin},
    interrupt,
    pac::{Peripherals as Stm32F7Peripherals, SPI3},
    prelude::*,
    spi::{Enabled, Spi},
};

#[allow(dead_code)]
enum RfDevice {
    Transmitter,
    Listener,
}

static USE_GDB: bool = false;
static RF_DEVICE: RfDevice = RfDevice::Listener;

// Signal used by the main thread to do action on Button event
static SIGNAL: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

// Button that main thread and interrupt handler must share
static BUTTON: Mutex<RefCell<Option<Button>>> = Mutex::new(RefCell::new(None));

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
    let mut rcc = pac.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    // Initialize GPIO Ports
    let gpiob = pac.GPIOB.split();
    let gpiod = pac.GPIOD.split();
    let gpioc = pac.GPIOC.split();

    // Example of printing with GDB debugger
    if USE_GDB {
        hprintln!("Hello World!"); // printing via debugger
    }

    // Initialize delay functionality
    let mut delay = Delay::new(cp.SYST, 216.MHz::<1, 1>().raw());

    // Initialize LEDs
    let mut led_green = LedGreen::new(gpiob.pb0);
    let mut led_blue = LedBlue::new(gpiob.pb7);
    let mut led_red = LedRed::new(gpiob.pb14);
    // led_green.set_state(PinState::High);
    // led_blue.set_state(PinState::High);
    // led_red.set_state(PinState::High);

    // Initialize UART for serial communication through USB
    let mut serial = SerialUartUsb::new(pac.USART3, &clocks, gpiod.pd8, gpiod.pd9);
    // serial.println("Hello there!");

    // Initialize Temperature Sensor
    let mut temp_sensor = TemperatureSensor::new(pac.ADC_COMMON, pac.ADC1, &mut rcc.apb2, &clocks);
    serial.formatln(format_args!("Temp: [{:?}]", temp_sensor.read_temperature()));

    // Initialize MCU UID
    let mcu_uid = McuUid::new();

    // Initialize SPI3
    let mut spi = SpiMaster3::new(
        pac.SPI3,
        &clocks,
        &mut rcc.apb1,
        gpioc.pc9,
        gpioc.pc10,
        gpioc.pc11,
        gpioc.pc12,
    );

    // Initialize User Button
    let mut syscfg = pac.SYSCFG;
    let mut exti = pac.EXTI;
    let mut button = Button::new(gpioc.pc13);
    button.enable_interrupt(Edge::Rising, &mut syscfg, &mut exti, &mut rcc.apb2);

    // Save information needed by the interrupt handler to the global variable
    free(|cs| {
        BUTTON.borrow(cs).replace(Some(button));
    });

    // Enable the button interrupt
    unsafe {
        NVIC::unmask::<interrupt>(interrupt::EXTI15_10);
    }

    // Initialize CC1101 Wrapper
    let mut cc1101_wrp = Cc1101Wrapper::new(spi.spi, spi.cs);
    cc1101_wrp.configure_radio().unwrap();

    let (partnum, version) = cc1101_wrp.get_hw_info().unwrap();
    serial.formatln(format_args!("partnum = {}, version = {}", partnum, version));

    // Cyclic part
    loop {
        free(|cs| {
            // Wait for the interrupt signal from the Button
            if false == SIGNAL.borrow(cs).get() {
                // Perform actions on the Button push event
                led_green.toggle();
                led_blue.toggle();
                led_red.toggle();

                SIGNAL.borrow(cs).set(true);
            }
        });

        // delay.delay_us(1_000_000);
        // led_green.toggle();

        // User Code

        // match RF_DEVICE {
        //     RfDevice::Transmitter => {
        //         // transmit packet and delay(1s)
        //         delay(1_000_000);
        //         let mut dst = 0u8;
        //         let mut buffer = [0u8; 15];
        //         let _result = cc1101_object.transmit_packet(&mut dst,  &mut buffer).unwrap();

        //     },

        //     RfDevice::Listener => {
        //     // attempt to read on radio
        //     // if read is succesful, send packet via uart
        //         let mut dst = 0u8;
        //         let mut buffer = [0u8; 17];
        //         if let Ok(_result) = cc1101_object.receive_packet(&mut dst, & mut buffer) {
        //             let _x = 0;
        //             // bsp_obj.formatln(format_args!("Message: {}", x));
        //         }
        //      },
        // };

        // Receive part
    }
}

#[interrupt]
fn EXTI15_10() {
    free(|cs| {
        if let Some(button) = BUTTON.borrow(cs).borrow_mut().as_mut() {
            button.clear_interrupt_pending_bit()
        }

        // Signal that the interrupt fired
        SIGNAL.borrow(cs).set(false);
    });
}
