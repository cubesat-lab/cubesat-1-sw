use static_cell::StaticCell;
use stm32f7xx_hal::{
    gpio::Pin,
    otg_fs::{UsbBus, USB},
    pac::{OTG_FS_DEVICE, OTG_FS_GLOBAL, OTG_FS_PWRCLK},
    rcc::Clocks,
};
use usb_device::{bus::UsbBusAllocator, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

static EP_MEMORY: StaticCell<[u32; 1024]> = StaticCell::new();
static USB_BUS: StaticCell<UsbBusAllocator<UsbBus<USB>>> = StaticCell::new();

pub struct UsbParameters<'a> {
    pub global: OTG_FS_GLOBAL,
    pub device: OTG_FS_DEVICE,
    pub pwrclk: OTG_FS_PWRCLK,
    pub clocks: &'a Clocks,
    pub pin_dm: Pin<'A', 11>,
    pub pin_dp: Pin<'A', 12>,
}

#[allow(dead_code)]
pub struct Usb<'a> {
    bus: &'a UsbBusAllocator<UsbBus<USB>>,
    device: UsbDevice<'static, UsbBus<USB>>,
    serial: SerialPort<'static, UsbBus<USB>>,
}

impl<'a> Usb<'a> {
    pub fn new(usb_parameters: UsbParameters) -> Self {
        // Initialize the USB peripheral
        let usb = USB::new(
            usb_parameters.global,
            usb_parameters.device,
            usb_parameters.pwrclk,
            (
                usb_parameters.pin_dm.into_alternate(),
                usb_parameters.pin_dp.into_alternate(),
            ),
            usb_parameters.clocks,
        );

        let ep_memory = EP_MEMORY.init([0; 1024]);

        // Create the USB bus allocator
        let bus = USB_BUS.init(UsbBus::new(usb, ep_memory));

        let serial = SerialPort::new(bus);

        let device = UsbDeviceBuilder::new(bus, UsbVidPid(0x16c0, 0x27dd))
            .strings(&[StringDescriptors::default()
                .manufacturer("CubeSat Lab")
                .product("CubeSat-1 OBC")
                .serial_number("0001")])
            .unwrap()
            .device_class(USB_CLASS_CDC)
            .max_packet_size_0(64) // Size required for HS, and ok for FS
            .unwrap()
            .build();

        Self {
            bus,
            device,
            serial,
        }
    }

    /// Poll the USB device
    pub fn poll(&mut self) -> bool {
        self.device.poll(&mut [&mut self.serial])
    }

    pub fn read(&mut self, data: &mut [u8]) -> Result<usize, UsbError> {
        self.serial.read(data)
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, UsbError> {
        self.serial.write(data)
    }
}
