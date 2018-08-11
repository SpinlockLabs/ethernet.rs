extern crate ethernet;

use std::mem;

use ethernet::cards::e1000::generic::*;
use ethernet::utils::linux::uio::*;

pub fn handle_device_discovered(udev: UioDevice) {
    println!("Found e1000 card: uio{} ({})", udev.dev_num(), udev.get_name().unwrap());

    let mut resource = udev.map_resource(0).unwrap();
    let driver : E1000DeviceDriver;

    unsafe {
        let rawmem = mem::transmute::<*mut u8, *mut u32>(resource.as_mut_ptr());
        driver = E1000DeviceDriver::new(rawmem);
    }

    driver.init_device().unwrap();
    let mac = driver.read_mac();
    println!("MAC Address: {:2x}:{:2x}:{:2x}:{:2x}:{:2x}:{:2x}",
        mac[0],
        mac[1],
        mac[2],
        mac[3],
        mac[4],
        mac[5]
    );
}

pub fn main() {
    for mut device in UioDevice::list_devices().unwrap() {
        let vendor_str = device.get_device_attr("vendor").unwrap();
        let device_id_str = device.get_device_attr("device").unwrap();

        let vendor = u32::from_str_radix(&vendor_str[2..].trim(), 16).unwrap();
        let device_id = u32::from_str_radix(&device_id_str[2..].trim(), 16).unwrap();

        if vendor != 0x8086 {
            continue;
        }

        if !E1000_SUPPORTED_DEVICES.iter().any(|id| *id == device_id) {
            continue;
        }

        handle_device_discovered(device);
    }
}
