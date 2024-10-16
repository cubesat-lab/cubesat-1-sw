#![no_std]

pub mod prelude {

    pub use rtic_monotonics::systick::prelude::*;

    systick_monotonic!(SysTime, 1_000);
}
