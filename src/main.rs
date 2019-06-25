//! CDC-ACM serial port example using cortex-m-rtfm.
#![no_main]
#![no_std]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate panic_semihosting;

extern crate stm32f0;

mod hid;
mod hiddesc;

use hid::HidClass;
use usbd_serial::CdcAcmClass;

//use cortex_m::asm::delay;
use stm32f0::stm32f0x2 as stm32;
use stm32f0::stm32f0x2::{dac, Interrupt};
use stm32f0xx_hal::adc::*;
use stm32f0xx_hal::prelude::*;
use stm32f0xx_hal::stm32::interrupt;

use digital_filter::DigitalFilter;
use stm32_usbd::{UsbBus, UsbBusType};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

use core::{cell::RefCell, fmt::Write, ops::DerefMut};

/// Global peripherals
/// Each of these is a Mutex wrapping a RefCell that wraps an Option that wraps a T
/// Pretty simple design pattern to keep.  You can also wrap related things in a
/// struct and have only one mutex per group of things.
// TODO - Move to module.
static USB_DEV: Mutex<RefCell<Option<UsbDevice<UsbBusType>>>> = Mutex::new(RefCell::new(None));
static USB_SERIAL: Mutex<RefCell<Option<CdcAcmClass<UsbBusType>>>> = Mutex::new(RefCell::new(None));
static HID: Mutex<RefCell<Option<HidClass<UsbBusType>>>> = Mutex::new(RefCell::new(None));

/**
 * #[entry] macro designates this function as the "main" of your program.
 * It does not have to be named main(), you can name it whatever you like.
 */
#[entry]
fn fuz() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        cortex_m::interrupt::free(|cs| {
            /// This is static because it must not be allowed
            /// to 'go away' after this function ends.
            /// If it did the USB connection would disappear!
            static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
            let mut flash = p.FLASH;
            let mut rcc = p
                .RCC
                .configure()
                .sysclk(48.mhz())
                .enable_crs(p.CRS)
                .freeze(&mut flash);

            let i2c= p.I2C1;

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

            /// Get the CHIP_ID to use as a serial #.
            /// The CHIP_ID is 96 bits or 12 bytes long.
            const CHIP_ID_PTR: *const u8 = (0x1FFF_F7AC) as *const u8;
            let chip_id: &[u8] = unsafe { core::slice::from_raw_parts(CHIP_ID_PTR, 12) };

            // Static mut is unsafe.  Since we kow what we are doing here it is ok.
            unsafe {
                USB_BUS = Some(UsbBus::new(p.USB, (usb_dm, usb_dp)));
            }

            // Static mut is unsafe.  Since we kow what we are doing here it is ok.
            let hid = HidClass::new(unsafe { USB_BUS.as_ref().unwrap() }, true, &hiddesc::DESC);
            let serial = CdcAcmClass::new(unsafe { USB_BUS.as_ref().unwrap() }, 64);
            let usb_dev = UsbDeviceBuilder::new(
                unsafe { USB_BUS.as_ref().unwrap() },
                UsbVidPid(0x5824, 0x27dd),
            )
            .manufacturer("Fake company")
            .product("Composite Device")
            .serial_number(unsafe { core::str::from_utf8_unchecked(chip_id) }) // Pretty sure this won't convert without an issue...
            .device_class(0x00) // 0x00 means set at interface level.
            .max_packet_size_0(64)
            .self_powered(false)
            .max_power(500)
            .supports_remote_wakeup(false)
            .build();

            // The * at the beginning is dereferencing the RefMut that is returned by the borrow_mut();
            *USB_DEV.borrow(cs).borrow_mut() = Some(usb_dev);
            *USB_SERIAL.borrow(cs).borrow_mut() = Some(serial);
            *HID.borrow(cs).borrow_mut() = Some(hid);

            let mut nvic: cortex_m::peripheral::NVIC = cp.NVIC;

            nvic.enable(Interrupt::USB);
            //nvic.enable(Interrupt::ADC_COMP);
        });
    }

    loop {
        cortex_m::asm::wfi(); // Power down, don't waste power idling.
    }
}

/// The #[interrupt] macro tells the system the following function is
/// to be executed by that ISR.  The name is important, it must be the
/// same as the interrupt name.
#[interrupt]
fn USB() {
    usb_interrupt();
}

#[interrupt]
fn ADC_COMP() {
    // Do nothing for now.
}

fn usb_interrupt() {
    cortex_m::interrupt::free(|cs| {
        if let (Some(ref mut usb_dev), Some(ref mut serial), Some(ref mut hid)) = (
            USB_DEV.borrow(cs).borrow_mut().deref_mut(),
            USB_SERIAL.borrow(cs).borrow_mut().deref_mut(),
            HID.borrow(cs).borrow_mut().deref_mut(),
        ) {
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
