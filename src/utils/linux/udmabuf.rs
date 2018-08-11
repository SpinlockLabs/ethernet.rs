use std::fs;
use std::path;

use std::io::{Read, Error as IoError};
use std::num::ParseIntError;

pub use memmap::MmapMut;

error_chain! {
    foreign_links {
        ParseIntErr(ParseIntError);
        IoErr(IoError);
    }
}

pub struct Udmabuf {
    name: String
}

impl Udmabuf {
    pub fn supported() -> bool {
        path::Path::new("/sys/class/udmabuf").exists()
    }

    pub fn list() -> Result<Vec<Udmabuf>> {
        let mut out = Vec::new();
        for r in fs::read_dir("/sys/class/udmabuf")? {
            let entry = r?;
            let maybe_file_name = entry.file_name().into_string();

            out.push(Udmabuf {
                name: maybe_file_name.unwrap()
            });
        }
        Ok(out)
    }

    pub fn named<T>(name: T) -> Result<Udmabuf> where T: Into<String> {
        Ok(Udmabuf {
            name: name.into()
        })
    }

    pub fn size(&self) -> Result<usize> {
        let mut path : String = "/sys/class/udmabuf/".into();
        path.push_str(&self.name);
        path.push_str("/size");
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let size = usize::from_str_radix(&contents, 10)?;
        Ok(size)
    }

    pub fn phys(&self) -> Result<u64> {
        let mut path : String = "/sys/class/udmabuf/".into();
        path.push_str(&self.name);
        path.push_str("/phys");
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let phys = u64::from_str_radix(&contents[2..], 16)?;
        Ok(phys)
    }

    pub fn map(&self) -> Result<MmapMut> {
        let mut path : String = "/dev/".into();
        path.push_str(&self.name);
        let file = fs::File::open(path)?;

        unsafe {
            Ok(MmapMut::map_mut(&file)?)
        }
    }
}
