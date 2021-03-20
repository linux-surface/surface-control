use std::fs::File;
use std::path::Path;
use std::os::unix::io::AsRawFd;

use nix::{ioctl_none, ioctl_read};

use crate::sys::{Error, Result};


#[derive(Debug)]
pub enum DeviceMode {
    Tablet,
    Laptop,
    Studio,
}

impl DeviceMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceMode::Tablet => "Tablet",
            DeviceMode::Laptop => "Laptop",
            DeviceMode::Studio => "Studio",
        }
    }
}

impl std::fmt::Display for DeviceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


#[derive(Debug)]
pub struct Device {
    file: File,
}

impl Device {
    pub fn open() -> Result<Self> {
        Device::open_path("/dev/surface/dtx")
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        File::open(path.as_ref())
            .map_err(|source| Error::DeviceAccess { source, device: path.as_ref().to_owned() })
            .map(|file| Device { file })
    }

    pub fn latch_lock(&self) -> Result<()> {
        unsafe { dtx_latch_lock(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn latch_unlock(&self) -> Result<()> {
        unsafe { dtx_latch_unlock(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn latch_request(&self) -> Result<()> {
        unsafe { dtx_latch_request(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn get_device_mode(&self) -> Result<DeviceMode> {
        let mut mode: u16 = 0;

        unsafe { dtx_get_opmode(self.file.as_raw_fd(), &mut mode as *mut u16) }
            .map_err(|source| Error::IoctlError { source })?;

        match mode {
            0 => Ok(DeviceMode::Tablet),
            1 => Ok(DeviceMode::Laptop),
            2 => Ok(DeviceMode::Studio),
            _ => Err(Error::InvalidData),
        }
    }
}


ioctl_none!(dtx_latch_lock,    0xa5, 0x23);
ioctl_none!(dtx_latch_unlock,  0xa5, 0x24);
ioctl_none!(dtx_latch_request, 0xa5, 0x25);
ioctl_read!(dtx_get_opmode,    0xa5, 0x2a, u16);
