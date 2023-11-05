#![no_main]
#![no_std]

use cc1101_wrapper::Cc1101Wrapper;
use core::cell::{Cell, RefCell};
use cortex_m::{
    delay::Delay,
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
};
use panic_halt as _;
use stm32f7xx_hal::{gpio::Edge, interrupt, pac::Peripherals as Stm32F7Peripherals, prelude::*};

static USE_GDB: bool = false;

// Signal used by the main thread to do action on Button event
static SIGNAL: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

// Button that main thread and interrupt handler must share
static BUTTON: Mutex<RefCell<Option<Button>>> = Mutex::new(RefCell::new(None));

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

    // Example of printing with GDB debugger
    if USE_GDB {
        hprintln!("Hello World!"); // printing via debugger
    }

    // Initialize delay functionality
    let mut delay = Delay::new(cp.SYST, 216.MHz::<1, 1>().raw());

    // Initialize LEDs
    let mut _led_green = LedGreen::new(gpiob.pb0);
    let mut _led_blue = LedBlue::new(gpiob.pb7);
    let mut _led_red = LedRed::new(gpiob.pb14);

    // Initialize UART for serial communication through USB
    let mut serial = SerialUartUsb::new(pac.USART3, &clocks, gpiod.pd8, gpiod.pd9);

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

    // Initialize CC1101 Wrapper - RF Device 1
    let mut cc1101_wrp_1 = Cc1101Wrapper::new(spi_3.spi, spi_3.cs);
    cc1101_wrp_1.configure_radio().unwrap();

    // Cyclic part
    loop {
        free(|cs| {
            // Wait for the interrupt signal from the Button
            if false == SIGNAL.borrow(cs).get() {
                // Perform actions on the Button push event
                // User Code

                SIGNAL.borrow(cs).set(true);
            }
        });

        serial.println(" ");
        delay.delay_us(1_000_000);

        // User Code
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
