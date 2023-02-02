#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_halt as _;

use crate::hal::{pac, prelude::*};
use stm32f7xx_hal as hal;

#[entry]
fn main() -> ! {
    let _cubesat = "
           ___________  
         /            /|
        /___________ / |
        |           |  |
        |           |  |
        | CubeSat-1 |  |
        |           | / 
        |___________|/  
    ";
    let _hello_world = "Hello world!\nI dream to be an OBC firmware for the CubeSat-1 project when I will grow up (^_^)";

    #[allow(unused_variables)]
    let mut counter: u32 = 0;

    // Init GPIO Led
    let p = pac::Peripherals::take().unwrap();
    let gpiob = p.GPIOB.split();
    let mut led_green = gpiob.pb0.into_push_pull_output();
    let mut led_blue = gpiob.pb7.into_push_pull_output();
    let mut led_red = gpiob.pb14.into_push_pull_output();

    loop {
        // Dummy variable incrementing
        counter += 1;

        for _ in 0..10_000 {
            led_green.set_high();
        }
        for _ in 0..10_000 {
            led_green.set_low();
        }
        for _ in 0..10_000 {
            led_blue.set_high();
        }
        for _ in 0..10_000 {
            led_blue.set_low();
        }
        for _ in 0..10_000 {
            led_red.set_high();
        }
        for _ in 0..10_000 {
            led_red.set_low();
        }
    }
}
