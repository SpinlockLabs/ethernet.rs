use std::mem;

use std::thread::sleep;
use std::time::Duration;

error_chain! {
}

#[allow(unused)]
mod constants {
    pub const E1000_REG_CTRL : usize = 0x0000;
    pub const E1000_REG_STATUS : usize = 0x0008;
    pub const E1000_REG_EEPROM : usize = 0x0014;
    pub const E1000_REG_EXT : usize = 0x0018;

    pub const E1000_REG_ICR : usize = 0x000C0;
    pub const E1000_REG_IMS : usize = 0x000D0;
    pub const E1000_REG_IMC : usize = 0x000D8;
}

pub const E1000_SUPPORTED_DEVICES : [u32; 1] = [
    0x15b8
];

#[repr(packed)]
pub struct E1000RxDesc {
    pub addr: u64,
    pub length: u64,
    pub checksum: u16,
    pub status: u8,
    pub errors: u8,
    pub special: u16
}

#[repr(packed)]
pub struct E1000TxDesc {
    pub addr: u64,
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
    pub status: u8,
    pub css: u8,
    pub special: u8
}

pub struct E1000Buffer {
    pub buffer: *mut u8,
    pub phys: u64,
    pub length: usize
}

pub struct E1000DeviceDriver {
    mem: *mut u32
}

impl E1000DeviceDriver {
    pub fn new(mem: *mut u32) -> E1000DeviceDriver {
        E1000DeviceDriver {
            mem
        }
    }

    unsafe fn read_cmd(&self, offset: usize) -> u32 {
        return *self.mem.offset(offset as isize);
    }

    unsafe fn read_byte(&self, offset: usize) -> u8 {
        let bytes = mem::transmute::<*mut u32, *mut u8>(self.mem);
        return *bytes.offset(offset as isize);
    }

    unsafe fn write_cmd(&self, offset: usize, value: u32) {
        *self.mem.offset(offset as isize) = value;
    }

    pub fn read_status(&self) -> u32 {
        unsafe {
            return self.read_cmd(constants::E1000_REG_STATUS);
        }
    }

    pub fn read_mac(&self) -> Vec<u8> {
        unsafe {
            let mut data = vec![0u8; 6];

            for i in 0..6 {
                data[i] = self.read_byte(i + 0x5400);
            }

            data
        }
    }

    pub fn init_device(&self) -> Result<()> {
        unsafe {
            self.write_cmd(constants::E1000_REG_CTRL, 1 << 26);
            sleep(Duration::from_millis(1));
            let mut status = self.read_cmd(constants::E1000_REG_CTRL);
            status |= 1 << 5; // Auto Speed Detection
            status |= 1 << 6; // Link Up
            status &= !(1 << 3); // Disable Reset Link
            status &= !(1 << 31); // Disable Reset Phy
            status &= !(1 << 7); // Unset Invert Loss-of-Signal
            self.write_cmd(constants::E1000_REG_CTRL, status);
            sleep(Duration::from_millis(1));
            status = self.read_cmd(constants::E1000_REG_CTRL);
            status &= !(1 << 30);
            self.write_cmd(constants::E1000_REG_CTRL, status);
            sleep(Duration::from_millis(1));

            for i in 0..128 {
                self.write_cmd(0x5200 + i * 4, 0);
            }

            for i in 0..64 {
                self.write_cmd(0x4000 + i * 4, 0);
            }

            self.read_status();
        }
        Ok(())
    }
}
