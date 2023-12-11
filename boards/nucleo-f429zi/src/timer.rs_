/// TODO: This code shall be adapted and implemented according to needs for future use-cases
/// (Code taken from previous board_demo.rs)


// // Globally available SysTick variable
// static SYSTICK_US: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));


// sys_counter: SysCounter<1000000>,

// sys_counter: sys_counter_obj,



// struct TimerUs {
//     init: u64,
//     period: u64,
// }

// impl Default for TimerUs {
//     fn default() -> Self {
//         TimerUs { init: 0, period: 0 }
//     }
// }

// impl TimerUs {
//     #[allow(dead_code)]
//     fn init(&mut self, time: u64, period: u64) {
//         self.init = time;
//         self.period = period;
//     }

//     #[allow(dead_code)]
//     fn check_expired(&mut self, time: u64) -> bool {
//         if (self.init + self.period) >= time {
//             return false;
//         } else {
//             return true;
//         }
//     }
// }

//     // Create a timer based on SysTick
//     let mut sys_counter_obj = cortex_m::peripheral::Peripherals::take()
//         .unwrap()
//         .SYST
//         .counter_us(&clocks);

//     // Register to SysTick exception
//     sys_counter_obj.listen(SysEvent::Update);

//     // Set timer to 1 ms
//     sys_counter_obj.start(1.millis()).unwrap();



//     pub fn delay(&mut self, time_us: u32) {
//         let t1 = self.get_systick_us();
//         let mut t2 = t1;

//         while (t1 + time_us as u64) > t2 {
//             t2 = self.get_systick_us();
//         }
//     }

//     pub fn get_systick_us(&mut self) -> u64 {
//         let mut systick: u64 = 0;

//         free(|cs| {
//             systick = SYSTICK_US.borrow(cs).get();
//             systick += self.sys_counter.now().ticks() as u64;
//         });

//         systick
//     }




// #[exception]
// fn SysTick() {
//     free(|cs| {
//         // Increment with 1 ms the SYSTICK_US
//         let mut systick = SYSTICK_US.borrow(cs).get();
//         systick += 1000; // 1000 us
//         SYSTICK_US.borrow(cs).set(systick);
//     });
// }
