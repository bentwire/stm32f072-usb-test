//! CDC-ACM serial port example using cortex-m-rtfm.
#![no_main]
#![no_std]
#![allow(non_snake_case)]

extern crate panic_semihosting;

use usbd_serial;

use cortex_m::asm::delay;
use rtfm::app;
use stm32f0xx_hal::prelude::*;

use stm32_usbd::{UsbBus, UsbBusType};
use usb_device::bus;
use usb_device::prelude::*;

#[app(device = stm32f0xx_hal::stm32 )]
const APP: () = {
    static mut USB_DEV: UsbDevice<'static, UsbBusType> = ();
    static mut SERIAL: usbd_serial::CdcAcmClass<'static, UsbBusType> = ();

    #[init]
    fn init() {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        let mut flash = device.FLASH;
        let mut rcc = device.RCC;

        let mut clocks = rcc.configure().sysclk(48.mhz()).enable_crs(device.CRS).freeze(&mut flash);

        let mut gpioa = device.GPIOA.split(&mut clocks);

        let usb_dm = gpioa.pa11;
        let usb_dp = gpioa.pa12;

        *USB_BUS = Some(UsbBus::new(device.USB, (usb_dm, usb_dp)));

        let serial = usbd_serial::CdcAcmClass::new(USB_BUS.as_ref().unwrap(), 64);

        let usb_dev =
            UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x5824, 0x27dd))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(usbd_serial::USB_CLASS_CDC)
                .build();

        USB_DEV = usb_dev;
        SERIAL = serial;
    }

    #[interrupt(resources = [USB_DEV, SERIAL])]
    fn USB() {
        usb_poll(&mut resources.USB_DEV, &mut resources.SERIAL);
    }
};

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    serial: &mut usbd_serial::CdcAcmClass<'static, B>,
) {
    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 8];

    match serial.read_packet(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back in upper case
            for c in buf[0..count].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }

            serial.write_packet(&buf[0..count]).ok();
        }
        _ => {}
    }
}
