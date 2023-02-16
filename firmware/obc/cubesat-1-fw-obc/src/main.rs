#![no_main]
#![no_std]

mod board_demo;

use board_demo::BoardDemo;
use cortex_m_rt::entry;
use panic_halt as _;

static USE_BOARD_DEMO: bool = true;
static USE_GDB: bool = false;

#[entry]
fn main() -> ! {
    // Initialization part
    let mut board = BoardDemo::init(USE_GDB);

    // User Code

    // Start part
    if USE_BOARD_DEMO {
        board.start();
    } else {
        // User Code
    }

    // Cyclic part
    loop {
        if USE_BOARD_DEMO {
            board.cyclic();
        } else {
            // User Code
        }
    }
}
