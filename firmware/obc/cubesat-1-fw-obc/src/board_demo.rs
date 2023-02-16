use core::{
    cell::{Cell, RefCell},
    fmt::{Arguments, Write},
    str::from_utf8,
};
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use cortex_m_rt::exception;
use cortex_m_semihosting::hprintln;
use embedded_hal::adc::{Channel as AdcChannel, OneShot};
use fugit::HertzU32 as Hertz;
use stm32f7xx_hal::{
    adc::Adc,
    gpio::{gpioc::PC13, Alternate, Edge, ExtiPin, Floating, Input, Output, Pin},
    interrupt,
    pac::{self, ADC1, ADC_COMMON, TIM3, USART3},
    prelude::*,
    serial::{self, Serial},
    signature::{Uid, VtempCal110, VtempCal30},
    timer::{Ch, Channel, PwmHz, SysCounter, SysEvent},
};

// Semaphore for synchronization
static SEMAPHORE: Mutex<Cell<bool>> = Mutex::new(Cell::new(true));

// GPIO pin that main thread and interrupt handler must share
static BUTTON_PIN: Mutex<RefCell<Option<PC13<Input<Floating>>>>> = Mutex::new(RefCell::new(None));

// Globally available SysTick variable
static SYSTICK_US: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

static TAB: &str = "    ";

#[derive(Debug, Copy, PartialEq)]
enum BoardDemoMode {
    GreenLedPulse,
    BlueLedBlink,
    RedLedBlink,
    DataReport,
    IdleState,
    ContTempReport,
}

impl Clone for BoardDemoMode {
    fn clone(&self) -> BoardDemoMode {
        *self
    }
}

impl BoardDemoMode {
    fn change(&mut self) {
        *self = match *self {
            BoardDemoMode::GreenLedPulse => BoardDemoMode::BlueLedBlink,
            BoardDemoMode::BlueLedBlink => BoardDemoMode::RedLedBlink,
            BoardDemoMode::RedLedBlink => BoardDemoMode::DataReport,
            BoardDemoMode::DataReport => BoardDemoMode::IdleState,
            BoardDemoMode::IdleState => BoardDemoMode::ContTempReport,
            BoardDemoMode::ContTempReport => BoardDemoMode::GreenLedPulse,
        };
    }
}

#[derive(Debug, Copy)]
struct BoardDemoMcuUid {
    x: u16,           // X coordinate on wafer
    y: u16,           // Y coordinate on wafer
    waf_num: u8,      // Wafer number
    lot_num: [u8; 7], // Lot number
}

impl Clone for BoardDemoMcuUid {
    fn clone(&self) -> BoardDemoMcuUid {
        *self
    }
}

#[allow(dead_code)]
struct BoardDemoGpio {
    led_blue: Pin<'B', 7, Output>,
    led_red: Pin<'B', 14, Output>,
}

struct TimerUs {
    init: u64,
    period: u64,
}

impl Default for TimerUs {
    fn default() -> Self {
        TimerUs { init: 0, period: 0 }
    }
}

impl TimerUs {
    fn init(&mut self, time: u64, period: u64) {
        self.init = time;
        self.period = period;
    }

    fn check_expired(&mut self, time: u64) -> bool {
        if (self.init + self.period) >= time {
            return false;
        } else {
            return true;
        }
    }
}

struct BoardDemoLedAttr<T> {
    pin: T,
    timer: TimerUs,
}

struct BoardDemoLed {
    blue: BoardDemoLedAttr<Pin<'B', 7, Output>>,
    red: BoardDemoLedAttr<Pin<'B', 14, Output>>,
}

struct BoardDemoPwmAttr<T> {
    pin: T,
    max_duty: u16,
    current_duty: u16,
    last_duty: u16,
}

struct BoardDemoPwm {
    led_green: BoardDemoPwmAttr<PwmHz<TIM3, Ch<2>, Pin<'B', 0, Alternate<2>>>>,
}

struct AdcTemperatureSensor;

impl AdcChannel<ADC1> for AdcTemperatureSensor {
    type ID = u8;
    fn channel() -> u8 {
        18_u8
    } // Temperature sensor is connected to ADC1_IN18
}

#[allow(dead_code)]
struct BoardDemoAdc {
    adc_common: ADC_COMMON,
    adc1: Adc<ADC1>,
    adc_ch_ts: AdcTemperatureSensor,
}

struct BoardDemoTemperatureSensor {
    cal30: u16,
    cal110: u16,
    timer: TimerUs,
}

#[allow(dead_code)]
struct BoardDemoBtnAttr<T> {
    pin: T,
}

#[allow(dead_code)]
struct BoardDemoBtn {
    btn_b1_user: BoardDemoBtnAttr<Pin<'C', 13, Input>>,
}

#[allow(dead_code)]
struct BoardDemoSerial {
    tx: stm32f7xx_hal::serial::Tx<USART3>,
    rx: stm32f7xx_hal::serial::Rx<USART3>,
}

pub struct BoardDemo {
    mode: BoardDemoMode,
    mcu_uid: BoardDemoMcuUid,
    counter: u32,
    led: BoardDemoLed,
    pwm: BoardDemoPwm,
    adc: BoardDemoAdc,
    temp_sensor: BoardDemoTemperatureSensor,
    serial: BoardDemoSerial,
    sys_counter: SysCounter<1000000>,
    gdb: bool,
}

impl BoardDemo {
    pub fn init(gdb: bool) -> Self {
        // Initialize Peripheral Access
        let pac_obj = pac::Peripherals::take().unwrap();
        let mut syscfg = pac_obj.SYSCFG;
        let mut exti = pac_obj.EXTI;

        // Set up the system clock. We want to run at 216MHz for this one.
        let mut rcc = pac_obj.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();
        let usart = pac_obj.USART3;

        // Initialize GPIO Ports
        let gpiob = pac_obj.GPIOB.split();
        let gpiod = pac_obj.GPIOD.split();
        let gpioc = pac_obj.GPIOC.split();

        // Init GPIO Led pins
        let pin_led_blue = gpiob.pb7.into_push_pull_output();
        let pin_led_red = gpiob.pb14.into_push_pull_output();

        // Init PWM pins
        let pin_led_green = gpiob.pb0.into_alternate();
        let mut pwm_led_green = pac_obj
            .TIM3
            .pwm_hz(pin_led_green, Hertz::Hz(1_000), &clocks);
        let max_duty_cycle = pwm_led_green.get_max_duty();
        pwm_led_green.enable(Channel::C3);
        pwm_led_green.set_duty(Channel::C3, 0);

        // Init USART3 pins
        let pin_uart_tx = gpiod.pd8.into_alternate();
        let pin_uart_rx = gpiod.pd9.into_alternate();

        // Create a timer based on SysTick
        let mut sys_counter_obj = cortex_m::peripheral::Peripherals::take()
            .unwrap()
            .SYST
            .counter_us(&clocks);

        // Register to SysTick exception
        sys_counter_obj.listen(SysEvent::Update);

        // Set timer to 1 ms
        sys_counter_obj.start(1.millis()).unwrap();

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

        // Setup ADC
        let adc1 = Adc::adc1(pac_obj.ADC1, &mut rcc.apb2, &clocks, 12, true);

        // Initialize internal temperature sensor
        // Enable the temperature and vref internal channels
        let adc_common = pac_obj.ADC_COMMON;
        adc_common.ccr.modify(|_, w| w.vbate().clear_bit()); // Disable VBAT
        adc_common.ccr.modify(|_, w| w.tsvrefe().set_bit()); // Enable TS and VREF

        // Init GPIO Button pins
        let mut pin_btn_b1_user = gpioc.pc13.into_floating_input();

        // Push button configuration
        pin_btn_b1_user.make_interrupt_source(&mut syscfg, &mut rcc.apb2);
        pin_btn_b1_user.trigger_on_edge(&mut exti, Edge::Rising);
        pin_btn_b1_user.enable_interrupt(&mut exti);

        // Save information needed by the interrupt handler to the global variable
        free(|cs| {
            BUTTON_PIN.borrow(cs).replace(Some(pin_btn_b1_user));
        });

        // Enable the button interrupt
        unsafe {
            NVIC::unmask::<interrupt>(interrupt::EXTI15_10);
        }

        // Copy lot number slice from UID
        let mut lot_num: [u8; 7] = Default::default();
        lot_num.clone_from_slice(Uid::get().lot_num().as_bytes());

        BoardDemo {
            mode: BoardDemoMode::GreenLedPulse,
            mcu_uid: BoardDemoMcuUid {
                x: Uid::get().x(),
                y: Uid::get().y(),
                waf_num: Uid::get().waf_num(),
                lot_num,
            },
            counter: 0,
            led: BoardDemoLed {
                blue: BoardDemoLedAttr {
                    pin: pin_led_blue,
                    timer: TimerUs::default(),
                },
                red: BoardDemoLedAttr {
                    pin: pin_led_red,
                    timer: TimerUs::default(),
                },
            },
            pwm: BoardDemoPwm {
                led_green: BoardDemoPwmAttr {
                    pin: pwm_led_green,
                    max_duty: max_duty_cycle,
                    current_duty: 0,
                    last_duty: 0,
                },
            },
            adc: BoardDemoAdc {
                adc_common,
                adc1,
                adc_ch_ts: AdcTemperatureSensor,
            },
            temp_sensor: BoardDemoTemperatureSensor {
                cal30: VtempCal30::get().read(),
                cal110: VtempCal110::get().read(),
                timer: TimerUs::default(),
            },
            serial: BoardDemoSerial {
                tx: serial_tx,
                rx: serial_rx,
            },
            sys_counter: sys_counter_obj,
            gdb,
        }
    }

    fn get_temperature(&mut self) -> f32 {
        // Read Temperature
        let adc_data: u16 = self.adc.adc1.read(&mut self.adc.adc_ch_ts).unwrap();

        // Temperature conversion formula
        let temperature: f32 = (110.0 - 30.0) * (adc_data - self.temp_sensor.cal30) as f32
            / (self.temp_sensor.cal110 - self.temp_sensor.cal30) as f32
            + 30.0;

        temperature
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
        self.println(" \n");
        self.report_mode();

        // gdb print
        if self.gdb {
            hprintln!("{}", hello_world); // printing via debugger
        }
    }

    fn play_pwm_led_green(&mut self) {
        let last_duty_frozen = self.pwm.led_green.last_duty;
        let step = 1;

        self.pwm
            .led_green
            .pin
            .set_duty(Channel::C3, self.pwm.led_green.current_duty);

        self.pwm.led_green.last_duty = self.pwm.led_green.current_duty;

        if self.pwm.led_green.current_duty == 0 {
            self.pwm.led_green.current_duty += step;
        } else if last_duty_frozen < self.pwm.led_green.current_duty {
            if (self.pwm.led_green.current_duty + step) > self.pwm.led_green.max_duty {
                self.pwm.led_green.current_duty = self.pwm.led_green.max_duty;
            } else {
                self.pwm.led_green.current_duty += step;
            }
        } else if self.pwm.led_green.current_duty == self.pwm.led_green.max_duty {
            self.pwm.led_green.current_duty -= step;
        } else if last_duty_frozen > self.pwm.led_green.current_duty {
            if self.pwm.led_green.current_duty < step {
                self.pwm.led_green.current_duty = 0;
            } else {
                self.pwm.led_green.current_duty -= step;
            }
        }

        self.delay(1);
    }

    fn stop_pwm_led_green(&mut self) {
        self.pwm.led_green.last_duty = 0;
        self.pwm.led_green.current_duty = 0;
        self.pwm.led_green.pin.set_duty(Channel::C3, 0);
    }

    fn report_mode(&mut self) {
        let mode = self.mode;
        self.formatln(format_args!("Mode: [{:?}]", mode));
    }

    fn on_button_action(&mut self) {
        self.mode.change();

        // Perform necessary steps before transition into a new state
        // Start timers, turn off LEDs, etc.
        if self.mode == BoardDemoMode::BlueLedBlink {
            self.stop_pwm_led_green();

            let time = self.get_systick_us();
            self.led.blue.timer.init(time, 500_000);
        } else if self.mode == BoardDemoMode::RedLedBlink {
            self.led.blue.pin.set_low();

            let time = self.get_systick_us();
            self.led.red.timer.init(time, 100_000);
        } else if self.mode == BoardDemoMode::DataReport {
            self.led.red.pin.set_low();
        }

        self.report_mode();
    }

    pub fn cyclic(&mut self) {
        // Wait for the interrupt to fire
        free(|cs| {
            if !SEMAPHORE.borrow(cs).get() {
                // Perform actions on button push event
                self.on_button_action();

                SEMAPHORE.borrow(cs).set(true);
            }
        });

        match self.mode {
            BoardDemoMode::GreenLedPulse => {
                self.play_pwm_led_green();
            }
            BoardDemoMode::BlueLedBlink => {
                let time = self.get_systick_us();

                if self.led.blue.timer.check_expired(time) {
                    self.led.blue.pin.toggle();
                    self.led.blue.timer.init(time, 500_000)
                }
            }
            BoardDemoMode::RedLedBlink => {
                let time = self.get_systick_us();

                if self.led.red.timer.check_expired(time) {
                    self.led.red.pin.toggle();
                    self.led.red.timer.init(time, 100_000)
                }
            }
            BoardDemoMode::DataReport => {
                self.counter += 1;
                let counter = self.counter;
                let mcu_uid = self.mcu_uid;
                let temperature = self.get_temperature();

                self.formatln(format_args!("{}UID:", TAB));
                self.formatln(format_args!("{}{}waf_x:   {}", TAB, TAB, mcu_uid.x));
                self.formatln(format_args!("{}{}waf_y:   {}", TAB, TAB, mcu_uid.y));
                self.formatln(format_args!("{}{}waf_num: {}", TAB, TAB, mcu_uid.waf_num));
                self.formatln(format_args!(
                    "{}{}lot_num: {}",
                    TAB,
                    TAB,
                    from_utf8(&mcu_uid.lot_num).unwrap()
                ));
                self.formatln(format_args!("{}Temperature: {:.2} °C", TAB, temperature));
                self.formatln(format_args!("{}Counter = {}", TAB, counter));

                // Prepare timer for temperature sensor
                let time = self.get_systick_us();
                self.temp_sensor.timer.init(time, 100_000);
                self.mode.change();
            }
            BoardDemoMode::ContTempReport => {
                let time = self.get_systick_us();

                if self.temp_sensor.timer.check_expired(time) {
                    let temperature = self.get_temperature();
                    self.formatln(format_args!("{}T = {:.2} °C", TAB, temperature));

                    self.temp_sensor.timer.init(time, 100_000);
                }
            }
            _ => { /* Do nothing in Idle state */ }
        }
    }

    pub fn delay(&mut self, time_us: u32) {
        let t1 = self.get_systick_us();
        let mut t2 = t1;

        while (t1 + time_us as u64) > t2 {
            t2 = self.get_systick_us();
        }
    }

    pub fn get_systick_us(&mut self) -> u64 {
        let mut systick: u64 = 0;

        free(|cs| {
            systick = SYSTICK_US.borrow(cs).get();
            systick += self.sys_counter.now().ticks() as u64;
        });

        systick
    }

    pub fn println(&mut self, s: &str) {
        self.serial.tx.write_fmt(format_args!("{}\n", s)).unwrap();
    }

    pub fn formatln(&mut self, args: Arguments) {
        self.serial.tx.write_fmt(args).unwrap();
        self.serial.tx.write_str("\n").unwrap();
    }
}

#[interrupt]
fn EXTI15_10() {
    free(|cs| {
        if let Some(btn) = BUTTON_PIN.borrow(cs).borrow_mut().as_mut() {
            btn.clear_interrupt_pending_bit()
        }

        // Signal that the interrupt fired
        SEMAPHORE.borrow(cs).set(false);
    });
}

#[exception]
fn SysTick() {
    free(|cs| {
        // Increment with 1 ms the SYSTICK_US
        let mut systick = SYSTICK_US.borrow(cs).get();
        systick += 1000; // 1000 us
        SYSTICK_US.borrow(cs).set(systick);
    });
}
