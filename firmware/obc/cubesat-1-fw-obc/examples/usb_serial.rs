//! CDC-ACM serial port example using polling in a busy loop.
//! Target board: any STM32F7 with an OTG FS/HS peripheral
//! This example works on the NUCLEO-F767ZI board.
//!
//! Note that `usbd-serial` library used in this example doesn't support
//! HighSpeed mode properly at the moment. See
//! https://github.com/mvirkkunen/usbd-serial/pull/14 for a potential workaround.
#![no_std]
#![no_main]

use panic_semihosting as _;

use cortex_m_rt::entry;
use stm32f7xx_hal::{
    otg_fs::{UsbBus, USB},
    pac,
    prelude::*,
    rcc::PLL48CLK,
};
use usb_device::prelude::*;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_pll()
        .use_pll48clk(PLL48CLK::Pllq)
        .sysclk(216.MHz())
        .freeze();

    let gpioa = dp.GPIOA.split();

    let usb = USB::new(
        dp.OTG_FS_GLOBAL,
        dp.OTG_FS_DEVICE,
        dp.OTG_FS_PWRCLK,
        (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()),
        &clocks,
    );

    static mut EP_MEMORY: [u32; 1024] = [0; 1024];
    #[allow(static_mut_refs)]
    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    let mut serial = usbd_serial::SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("CubeSat Lab")
            .product("CubeSat-1 OBC")
            .serial_number("0001")])
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        .max_packet_size_0(64) // Size required for HS, and ok for FS
        .unwrap()
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 512];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
