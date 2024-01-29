#![no_main]
#![no_std]

use core::cell::{Cell, RefCell};
use cortex_m::{
    delay::Delay,
    interrupt::{free, Mutex},
    peripheral::Peripherals as CortexMPeripherals,
    peripheral::NVIC,
};
use cortex_m_rt::entry;
use nucleo_f767zi::{
    button::{Button, ButtonParameters},
    led::{LedBlue, LedGreen, LedParameters, LedRed},
};
use panic_halt as _;
use stm32f7xx_hal::{
    gpio::{Edge, PinState},
    interrupt,
    pac::Peripherals as Stm32F7Peripherals,
    prelude::*,
};

// Signal used by the main thread to do action on Button event
static SIGNAL: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

// Button that main thread and interrupt handler must share
static BUTTON: Mutex<RefCell<Option<Button>>> = Mutex::new(RefCell::new(None));

// Led States
enum LedState {
    Red,
    Blue,
    Green,
}

#[entry]
fn main() -> ! {
    // Initialize Core Peripherals
    let cp: CortexMPeripherals = CortexMPeripherals::take().unwrap();

    // Initialize Peripheral Access Crate
    let pac = Stm32F7Peripherals::take().unwrap();

    // Set up the system clock. We want to run at 216MHz for this one.
    let mut rcc = pac.RCC.constrain();

    // Initialize GPIO Ports
    let gpiob = pac.GPIOB.split();
    let gpioc = pac.GPIOC.split();

    // Initialize delay functionality
    let mut delay = Delay::new(cp.SYST, 216.MHz::<1, 1>().raw());

    // Initialize LEDs
    let mut led_green = LedGreen::new(LedParameters { pin: gpiob.pb0 });
    let mut led_blue = LedBlue::new(LedParameters { pin: gpiob.pb7 });
    let mut led_red = LedRed::new(LedParameters { pin: gpiob.pb14 });

    // Initialize User Button
    let mut syscfg = pac.SYSCFG;
    let mut exti = pac.EXTI;
    let button = Button::new(ButtonParameters {
        pin: gpioc.pc13,
        edge: Edge::Rising,
        syscfg: &mut syscfg,
        exti: &mut exti,
        apb: &mut rcc.apb2,
    });

    // Save information needed by the interrupt handler to the global variable
    free(|cs| {
        BUTTON.borrow(cs).replace(Some(button));
    });

    // Enable the button interrupt
    unsafe {
        NVIC::unmask::<interrupt>(interrupt::EXTI15_10);
    }

    // Initialize Led state
    let mut led_state = LedState::Red;
    led_red.set_state(PinState::High);

    // Cyclic part
    loop {
        free(|cs| {
            // Wait for the interrupt signal from the Button
            if false == SIGNAL.borrow(cs).get() {
                // Perform actions on the Button push event
                match led_state {
                    LedState::Red => {
                        led_state = LedState::Blue;
                        led_red.toggle();
                        led_blue.toggle();
                    }
                    LedState::Blue => {
                        led_state = LedState::Green;
                        led_blue.toggle();
                        led_green.toggle();
                    }
                    LedState::Green => {
                        led_state = LedState::Red;
                        led_green.toggle();
                        led_red.toggle();
                    }
                };

                SIGNAL.borrow(cs).set(true);
            }
        });

        // TODO Improve debounce mechanism
        // Delay for debounce
        delay.delay_ms(10);
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
