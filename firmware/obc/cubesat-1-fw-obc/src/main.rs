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
    let gpioi = p.GPIOI.split();
    let mut led = gpioi.pi1.into_push_pull_output();

    loop {
        // Dummy variable incrementing
        counter += 1;

        for _ in 0..10_000 {
            led.set_high();
        }
        for _ in 0..10_000 {
            led.set_low();
        }
    }
}
