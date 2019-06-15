#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]

use core::convert::TryInto;
use core::mem;
use usb_device::class_prelude::*;
use usb_device::Result;

use super::reportdesc::*;
use core::ptr::null;
use core::borrow::Borrow;

// USB HID Class bInterfaceClass
const USB_CLASS_HID:                    u8 = 0x03;

// USB HID bInterfaceSubClass
const USB_SUBCLASS_HID_NONE:            u8 = 0x00;
const USB_SUBCLASS_HID_BOOT_INTERFACE:  u8 = 0x01;

// USB HID bInterfaceProtocol
const HID_PROTOCOL_NONE:                u8 = 0x00;
const HID_PROTOCOL_KEYBOARD:            u8 = 0x01;
const HID_PROTOCOL_MOUSE:               u8 = 0x02;

// Various sizes and stuff
const MAX_PACKET_SIZE:                  u16 = 64;

pub struct HidClass<'a, B: UsbBus> {
    hid_if: InterfaceNumber,
    in_report_ep: EndpointIn<'a, B>,
    out_report_ep: Option<EndpointOut<'a, B>>,
    report_desc: HidReportDescriptor<'a>,
}

impl<'a, B: UsbBus> HidClass<'a, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>, needs_out_ep: bool, desc: &'static[u8]) -> HidClass<'a,B> {
        let hid_if = alloc.interface();
        let in_report_ep = alloc.interrupt(MAX_PACKET_SIZE, 10);
        let out_report_ep = match needs_out_ep {
            true => Some(alloc.interrupt(MAX_PACKET_SIZE, 10)),
            false => None
        };

        HidClass {
            hid_if,
            in_report_ep,
            out_report_ep,
            report_desc: HidReportDescriptor { desc: desc.borrow() },
        }
    }

    pub fn max_packet_size(&self) -> u16 {
        // Both endpoints use the same size.
        self.in_report_ep.max_packet_size()
    }

    // Sends a Report
    pub fn send_report(&mut self, data: &[u8]) -> Result<usize> {
        self.in_report_ep.write(data)
    }

    // Reads a report
    pub fn read_report(&mut self, data: &mut [u8]) -> Result<usize> {
        let ep = &self.out_report_ep;
        match ep {
            None => Ok(0),
            Some(ep) => ep.read(data)
        }
    }
}

impl<B: UsbBus> UsbClass<B> for HidClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        writer.interface(self.hid_if,
                         USB_CLASS_HID,
                         USB_SUBCLASS_HID_NONE,
                         HID_PROTOCOL_NONE)?;

        writer.write(
            0x21, // HID Device descriptor
        &[
            0x11, 0x01, // bcdHID 1.11
            0x00, // bCountryCode Country code not supported.
            0x01, // bNumDescriptors One descriptor
            0x22, // bDescriptorType HID Report descriptor
            self.report_desc.desc.len() as u8, ((self.report_desc.desc.len() as u16) >> 8) as u8,  // wDescriptorLength
        ])?;

        writer.endpoint(&self.in_report_ep)?;

        let ep = &self.out_report_ep;
        match ep {
            Some(ep) => writer.endpoint(&ep),
            None => Ok(()),
        }?;

        Ok(())
    }

//    fn get_string(&self, index: StringIndex, lang_id: u16) -> Option<&str> {
//        unimplemented!()
//    }
//
//    fn reset(&mut self) {
//        unimplemented!()
//    }
//
//    fn poll(&mut self) {
//        unimplemented!()
//    }
//
    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();
        let reqt = req.request_type;

        let value = req.value;
        let length = req.length;
        let index = req.index;

//        if !((req.recipient == control::Recipient::Interface) && ((req.index & 0x00ff) == u8::from(self.hid_if) as u16)) {
//            return
//        }

        match reqt {
            control::RequestType::Standard => match req.request {
                control::Request::GET_DESCRIPTOR => {
                    let desc_type = (value >> 8) as u8;
                    let desc_index = (value & 0x00ff) as u8;

                    if desc_type == 0x22 {
                        xfer.accept(|data| {
                            let data_len = data.len();
                            data[0..length as usize].copy_from_slice(&self.report_desc.desc[0..length as usize]);
                            Ok(length as usize)
                        }).ok();
                    } else {
                        return
                    }
                },
                _ => { return }
            }
            _ => { return }
        }
    }

//    fn endpoint_setup(&mut self, addr: EndpointAddress) {
//        unimplemented!()
//    }
//
//    fn endpoint_out(&mut self, addr: EndpointAddress) {
//        unimplemented!()
//    }
//
//    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
//        unimplemented!()
//    }
}