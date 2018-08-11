use std::io;
use std::io::prelude::*;
use std::fs;
use std::fs::{File, OpenOptions};
use std::num::ParseIntError;
use std::str::FromStr;
use std::usize;

use memmap;
pub use memmap::MmapMut;

const PAGESIZE: usize = 4096;

#[derive(Debug)]
pub enum UioError {
    Address,
    Io(io::Error),
    Parse
}

impl From<io::Error> for UioError {
    fn from(e: io::Error) -> Self {
        UioError::Io(e)
    }
}

impl From<ParseIntError> for UioError {
    fn from(_: ParseIntError) -> Self {
        UioError::Parse
    }
}

pub struct UioDevice {
    uio_num: usize,
    devfile: File,
}

impl UioDevice {
    /// Creates a new UIO device for Linux.
    ///
    /// # Arguments
    ///  * uio_num - UIO index of device (i.e., 1 for /dev/uio1)
    pub fn new(uio_num: usize) -> io::Result<UioDevice> {
        let path = format!("/dev/uio{}", uio_num);
        let f = File::open(path)?;
        Ok(UioDevice { uio_num, devfile: f })
    }

    /// Creates a new UIO device for Linux.
    ///
    /// # Arguments
    ///  * uio_name - UIO name of device (uio1 for /dev/uio1)
    pub fn new_by_name(uio_name: &str) -> io::Result<UioDevice> {
        let path = format!("/dev/{}", uio_name);
        let uio_num = usize::from_str(&uio_name[3..]).unwrap();
        let f = File::open(path)?;
        Ok(UioDevice { uio_num, devfile: f })
    }

    /// Return a vector of mappable resources (i.e., PCI bars) including their size.
    pub fn get_resource_info(&mut self) -> Result<Vec<(String, u64)>, UioError> {
        let paths = fs::read_dir(format!("/sys/class/uio/uio{}/device/", self.uio_num))?;

        let mut bars = Vec::new();
        for p in paths {
            let path = p?;
            let file_name = path.file_name().into_string().expect("Is valid UTF-8 string.");

            if file_name.starts_with("resource") && file_name.len() > "resource".len() {
                let metadata = fs::metadata(path.path())?;
                bars.push((file_name, metadata.len()));
            }
        }

        Ok(bars)
    }

    /// Return the value of the given device attribute.
    ///
    /// # Example Attributes
    ///  * irq: IRQ number
    ///  * vendor: Vendor ID in hex
    pub fn get_device_attr(&mut self, attr_name: &str) -> Result<String, UioError> {
        Ok(fs::read_to_string(format!("/sys/class/uio/uio{}/device/{}", self.uio_num, attr_name))?)
    }

    /// Maps a given resource into the virtual address space of the process.
    ///
    /// # Arguments
    ///   * bar_nr: The index to the given resource (i.e., 1 for /sys/class/uio/uioX/device/resource1)
    pub fn map_resource(&self, bar_nr: usize) -> Result<MmapMut, UioError> {
        let filename = format!("/sys/class/uio/uio{}/device/resource{}", self.uio_num, bar_nr);
        let f = OpenOptions::new().read(true).write(true).open(filename.to_string())?;

        let mut mopts = memmap::MmapOptions::new();
        mopts.offset(0);

        unsafe {
            Ok(mopts.map_mut(&f)?)
        }
    }

    fn read_file(&self, path: String) -> Result<String, UioError> {
        let mut file = File::open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(buffer.trim().to_string())
    }

    fn parse_from<T>(&self, path: String) -> Result<T, UioError>
        where T: FromStr {
        let buffer = self.read_file(path)?;

        match buffer.parse::<T>() {
            Err(_) => { Err(UioError::Parse) }
            Ok(addr) => Ok(addr)
        }
    }

    /// The amount of events.
    pub fn get_event_count(&self) -> Result<u32, UioError> {
        let filename = format!("/sys/class/uio/uio{}/event", self.uio_num);
        self.parse_from::<u32>(filename)
    }

    /// The name of the UIO device.
    pub fn get_name(&self) -> Result<String, UioError> {
        let filename = format!("/sys/class/uio/uio{}/name", self.uio_num);
        self.read_file(filename)
    }

    /// The version of the UIO driver.
    pub fn get_version(&self) -> Result<String, UioError> {
        let filename = format!("/sys/class/uio/uio{}/version", self.uio_num);
        self.read_file(filename)
    }

    /// The size of a given mapping.
    ///
    /// # Arguments
    ///  * mapping: The given index of the mapping (i.e., 1 for /sys/class/uio/uioX/maps/map1)
    pub fn map_size(&self, mapping: usize) -> Result<usize, UioError> {
        let filename = format!("/sys/class/uio/uio{}/maps/map{}/size", self.uio_num, mapping);
        self.parse_from::<usize>(filename)
    }

    /// The address of a given mapping.
    ///
    /// # Arguments
    ///  * mapping: The given index of the mapping (i.e., 1 for /sys/class/uio/uioX/maps/map1)
    pub fn map_addr(&self, mapping: usize) -> Result<usize, UioError> {
        let filename = format!("/sys/class/uio/uio{}/maps/map{}/addr", self.uio_num, mapping);
        self.parse_from::<usize>(filename)
    }

    /// Return a list of all possible memory mappings.
    pub fn get_map_info(&mut self) -> Result<Vec<String>, UioError> {
        let paths = fs::read_dir(format!("/sys/class/uio/uio{}/maps/", self.uio_num))?;

        let mut map = Vec::new();
        for p in paths {
            let path = p?;
            let file_name = path.file_name().into_string().expect("Is valid UTF-8 string.");

            if file_name.starts_with("map") && file_name.len() > "map".len() {
                map.push(file_name);
            }
        }

        Ok(map)
    }

    /// Map an available memory mapping.
    ///
    /// # Arguments
    ///  * mapping: The given index of the mapping (i.e., 1 for /sys/class/uio/uioX/maps/map1)
    pub fn map_mapping(&self, mapping: usize) -> Result<MmapMut, UioError> {
        let offset = mapping * PAGESIZE;
        // let map_size = self.map_size(mapping).unwrap(); // TODO

        let mut mopts = memmap::MmapOptions::new();
        mopts.offset(offset);
        unsafe {
            Ok(mopts.map_mut(&self.devfile)?)
        }
    }
}
