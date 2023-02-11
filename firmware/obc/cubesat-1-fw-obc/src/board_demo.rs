use core::fmt::Arguments;
use core::fmt::Write;

#[allow(unused_imports)]
use cortex_m_semihosting::hprintln;
use stm32f7xx_hal::{
    gpio::{Output, Pin},
    pac::{self, USART3},
    prelude::*,
    serial::{self, Serial},
    timer::SysDelay,
};

struct BoardDemoPins {
    led_green: Pin<'B', 0, Output>,
    led_blue: Pin<'B', 7, Output>,
    led_red: Pin<'B', 14, Output>,
}

#[allow(dead_code)]
struct BoardDemoSerial {
    tx: stm32f7xx_hal::serial::Tx<USART3>,
    rx: stm32f7xx_hal::serial::Rx<USART3>,
}

pub struct BoardDemo {
    counter: u32,
    pin: BoardDemoPins,
    serial: BoardDemoSerial,
    sys_delay: SysDelay,
}

impl BoardDemo {
    pub fn init() -> Self {
        // Initialize Peripheral Access
        let pac_obj = pac::Peripherals::take().unwrap();

        // Initialize GPIO Ports
        let gpiob = pac_obj.GPIOB.split();
        let gpiod = pac_obj.GPIOD.split();

        // Init GPIO Led pins
        let pin_led_green = gpiob.pb0.into_push_pull_output();
        let pin_led_blue = gpiob.pb7.into_push_pull_output();
        let pin_led_red = gpiob.pb14.into_push_pull_output();

        // Init USART3 pins
        let pin_uart_tx = gpiod.pd8.into_alternate();
        let pin_uart_rx = gpiod.pd9.into_alternate();

        // Set up the system clock. We want to run at 216MHz for this one.
        let cortex_obj = cortex_m::peripheral::Peripherals::take().unwrap();
        let rcc = pac_obj.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();
        let usart = pac_obj.USART3;

        // Create a delay abstraction based on SysTick
        let sys_delay_obj = cortex_obj.SYST.delay(&clocks);

        // Init UART3 - Default to 115_200 bauds
        let serial = Serial::new(
            usart,
            (pin_uart_tx, pin_uart_rx),
            &clocks,
            serial::Config {
                ..Default::default()
            },
        );
        let (serial_tx, serial_rx) = serial.split();

        BoardDemo {
            counter: 0,
            pin: BoardDemoPins {
                led_green: pin_led_green,
                led_blue: pin_led_blue,
                led_red: pin_led_red,
            },
            serial: BoardDemoSerial {
                tx: serial_tx,
                rx: serial_rx,
            },
            sys_delay: sys_delay_obj,
        }
    }

    pub fn start(&mut self) {
        // Initialize string literals
        let cubesat = "
                   ___________
                 /            /|
                /___________ / |
                |           |  |
                |           |  |
                | CubeSat-1 |  |
                |           | /
                |___________|/
            ";
        let hello_world = "Hello world!\nI dream to be an OBC firmware for the CubeSat-1 project when I will grow up (^_^)";

        self.println(cubesat);
        self.println(hello_world);

        // gdb print
        // hprintln!("{}", hello_world); // Uncomment for printing via debugger
    }

    pub fn cyclic(&mut self) {
        // Dummy variable incrementing
        self.counter += 1;

        self.pin.led_green.set_high();
        self.delay(500_000);
        self.pin.led_green.set_low();

        self.pin.led_blue.set_high();
        self.delay(500_000);
        self.pin.led_blue.set_low();

        self.pin.led_red.set_high();
        self.delay(500_000);
        self.pin.led_red.set_low();

        let counter = self.counter;
        self.formatln(format_args!("{}", counter));
    }

    pub fn delay(&mut self, us: u32) {
        self.sys_delay.delay_us(us);
    }

    pub fn println(&mut self, s: &str) {
        self.serial.tx.write_fmt(format_args!("{}\n", s)).unwrap();
    }

    pub fn formatln(&mut self, args: Arguments) {
        self.serial.tx.write_fmt(args).unwrap();
        self.serial.tx.write_str("\n").unwrap();
    }
}
