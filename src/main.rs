//! CDC-ACM serial port example using cortex-m-rtfm.
#![no_main]
#![no_std]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate panic_semihosting;

extern crate stm32f0;

mod hid;
mod hiddesc;

use usbd_serial::CdcAcmClass;
use hid::HidClass;

//use cortex_m::asm::delay;
use stm32f0xx_hal::prelude::*;
use stm32f0xx_hal::stm32::{interrupt, Interrupt};
use stm32f0xx_hal::adc::*;
use stm32f0::stm32f0x2 as stm32;
use stm32f0::stm32f0x2::dac;

use stm32_usbd::{UsbBus, UsbBusType};
use usb_device::bus::{UsbBusAllocator};
use usb_device::prelude::*;
use digital_filter::DigitalFilter;

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

use core::{ cell::RefCell, fmt::Write, ops::DerefMut };

// Global peripherals
// TODO - Move to module.
static USB_DEV: Mutex<RefCell<Option<UsbDevice<UsbBusType>>>> = Mutex::new(RefCell::new(None));
static USB_SERIAL: Mutex<RefCell<Option<CdcAcmClass<UsbBusType>>>> = Mutex::new(RefCell::new(None));
static HID: Mutex<RefCell<Option<HidClass<UsbBusType>>>> = Mutex::new(RefCell::new(None));


#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        cortex_m::interrupt::free(|cs| {
            static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure()
                                    .sysclk(48.mhz())
                                    .enable_crs(p.CRS)
                                    .freeze(&mut flash);

            let i2c: stm32::I2C1 = p.I2C1;

            let gpioa = p.GPIOA.split(&mut rcc);

            // Setup ain0
            let adcA0 = gpioa.pa0.into_analog(cs);

            // Setup ADC.
            let mut adc = Adc::new(p.ADC, &mut rcc);
            adc.set_sample_time(AdcSampleTime::T_7);
            adc.set_precision(AdcPrecision::B_12);
            adc.set_align(AdcAlign::Right);

            let usb_dm = gpioa.pa11;
            let usb_dp = gpioa.pa12;

            // Static mut is unsafe.
            unsafe  { USB_BUS = Some(UsbBus::new(p.USB, (usb_dm, usb_dp))); }

            // Static mut is unsafe.
            let hid = unsafe { HidClass::new(USB_BUS.as_ref().unwrap(), true, &hiddesc::DESC) };
            let serial = unsafe { CdcAcmClass::new(USB_BUS.as_ref().unwrap(), 64) };
            let usb_dev =
                UsbDeviceBuilder::new(unsafe { USB_BUS.as_ref().unwrap() }, UsbVidPid(0x5824, 0x27dd))
                    .manufacturer("Fake company")
                    .product("Composite Device")
                    .serial_number("TEST")
                    .device_class(0x00)
                    .max_packet_size_0(64)
                    .self_powered(false)
                    .max_power(500)
                    .supports_remote_wakeup(false)
                    .build();


            *USB_DEV.borrow(cs).borrow_mut() = Some(usb_dev);
            *USB_SERIAL.borrow(cs).borrow_mut() = Some(serial);
            *HID    .borrow(cs).borrow_mut() = Some(hid);

            let mut nvic: cortex_m::peripheral::NVIC = cp.NVIC;

            nvic.enable(interrupt::USB);

        });
    }

    loop {
        cortex_m::asm::wfi(); // Power down, don't waste power idling.
    }
}

#[interrupt]
fn USB() {
    usb_interrupt();
}

fn usb_interrupt() {
    cortex_m::interrupt::free(|cs| {
        if let (Some(ref mut usb_dev), Some(ref mut serial), Some(ref mut hid)) = (USB_DEV.borrow(cs).borrow_mut().deref_mut(),
                                                                                   USB_SERIAL.borrow(cs).borrow_mut().deref_mut(),
                                                                                   HID.borrow(cs).borrow_mut().deref_mut())
        {
            if !usb_dev.poll(&mut [hid, serial]) {
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
    });
}
