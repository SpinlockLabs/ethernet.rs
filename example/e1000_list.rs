extern crate ethernet;

use std::mem;

use ethernet::cards::e1000::generic::*;
use ethernet::utils::linux::uio::*;

pub fn main() {
    let udev = UioDevice::new(0).unwrap();
    let mut resource = udev.map_resource(0).unwrap();

    let driver : E1000DeviceDriver;

    unsafe {
        let rawmem = mem::transmute::<*mut u8, *mut u32>(resource.as_mut_ptr());
        driver = E1000DeviceDriver::new(rawmem);
    }

    driver.init_device().unwrap();
    let mac = driver.read_mac();
    println!("{:2x}:{:2x}:{:2x}:{:2x}:{:2x}:{:2x}",
        mac[0],
        mac[1],
        mac[2],
        mac[3],
        mac[4],
        mac[5]
    );
}
