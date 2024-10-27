#![no_std]

pub mod prelude {
    use rtic_monotonics::fugit::{Duration, Instant};

    #[cfg(feature = "cortex-m-systick")]
    pub use rtic_monotonics::systick::prelude::*;

    #[cfg(feature = "cortex-m-systick")]
    systick_monotonic!(SysTime, 1_000);

    pub use rtic_monotonics::fugit::HertzU32 as FreqSize;

    #[cfg(any(feature = "stm32f767zi", feature = "stm32f446re"))]
    pub use rtic_monotonics::fugit::ExtU64 as TimeSize;

    #[cfg(feature = "stm32f100rb")]
    pub use rtic_monotonics::fugit::ExtU32 as TimeSize;

    #[cfg(any(feature = "stm32f767zi", feature = "stm32f446re"))]
    pub type TimeDuration = Duration<u64, 1, 1000>;

    #[cfg(feature = "stm32f100rb")]
    pub type TimeDuration = Duration<u32, 1, 1>;

    #[cfg(any(feature = "stm32f767zi", feature = "stm32f446re"))]
    pub type TimeInstant = Instant<u64, 1, 1000>;

    #[cfg(feature = "stm32f100rb")]
    pub type TimeInstant = Instant<u32, 1, 1>;
}
