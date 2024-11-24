#![no_std]

pub mod prelude {

    // Define the Time types with "empty type" using unit type
    #[cfg(not(any(
        feature = "stm32f767zi",
        feature = "stm32f446re",
        feature = "stm32f100rb"
    )))]
    mod empty_types {
        pub type FreqSize = ();
        pub type TimeSize = ();
        pub type TimeDuration = ();
        pub type TimeInstant = ();
    }

    #[cfg(not(any(
        feature = "stm32f767zi",
        feature = "stm32f446re",
        feature = "stm32f100rb"
    )))]
    pub use empty_types::*;

    #[allow(unused_imports)]
    use rtic_monotonics::fugit::{Duration, Instant};

    #[cfg(feature = "cortex-m-systick")]
    pub use rtic_monotonics::systick::prelude::*;

    #[cfg(feature = "cortex-m-systick")]
    systick_monotonic!(SysTime, 1_000);

    #[cfg(any(
        feature = "stm32f767zi",
        feature = "stm32f446re",
        feature = "stm32f100rb"
    ))]
    pub use rtic_monotonics::fugit::HertzU32 as FreqSize;

    #[cfg(any(feature = "stm32f767zi", feature = "stm32f446re"))]
    pub use rtic_monotonics::fugit::ExtU64 as TimeSize;

    #[cfg(feature = "stm32f100rb")]
    pub use rtic_monotonics::fugit::ExtU32 as TimeSize;

    #[cfg(any(feature = "stm32f767zi", feature = "stm32f446re"))]
    pub type TimeDuration = Duration<u64, 1, 1000>;

    #[cfg(feature = "stm32f100rb")]
    pub type TimeDuration = Duration<u32, 1, 1000>;

    #[cfg(any(feature = "stm32f767zi", feature = "stm32f446re"))]
    pub type TimeInstant = Instant<u64, 1, 1000>;

    #[cfg(feature = "stm32f100rb")]
    pub type TimeInstant = Instant<u32, 1, 1000>;
}
