#![no_main]
#![no_std]

mod bsp;

use bsp::Bsp;
use cortex_m_rt::entry;
use panic_halt as _;

static USE_GDB: bool = false;

#[entry]
fn main() -> ! {
    // Initialization part
    let mut bsp_obj = Bsp::init(USE_GDB);

    // User Code

    // Start part
    bsp_obj.start();

    // Cyclic part
    loop {
        // User Code
    }
}
