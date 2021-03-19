use std::fs::File;
use std::path::Path;
use std::os::unix::io::AsRawFd;

use nix::{ioctl_none, ioctl_read};

use crate::error::{Error, ErrorKind, Result, ResultExt};


#[derive(Debug)]
pub enum OpMode {
    Tablet,
    Laptop,
    Studio,
}

impl OpMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpMode::Tablet => "Tablet",
            OpMode::Laptop => "Laptop",
            OpMode::Studio => "Studio",
        }
    }
}

impl std::fmt::Display for OpMode {
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
        let result = File::open(path);

        match result {
            Ok(file) => Ok(Device { file }),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(failure::err_msg("Surface DTX device not found"))
                    .context(ErrorKind::DeviceAccess)
                    .map_err(Into::into)
            },
            Err(e) => {
                Err(e).context(ErrorKind::DeviceAccess)
                    .map_err(Into::into)
            },
        }

    }

    pub fn latch_lock(&self) -> Result<()> {
        unsafe { dtx_latch_lock(self.file.as_raw_fd()).context(ErrorKind::Io)?; }
        Ok(())
    }

    pub fn latch_unlock(&self) -> Result<()> {
        unsafe { dtx_latch_unlock(self.file.as_raw_fd()).context(ErrorKind::Io)?; }
        Ok(())
    }

    pub fn latch_request(&self) -> Result<()> {
        unsafe { dtx_latch_request(self.file.as_raw_fd()).context(ErrorKind::Io)?; }
        Ok(())
    }

    pub fn get_opmode(&self) -> Result<OpMode> {
        let mut opmode: u16 = 0;
        unsafe {
            dtx_get_opmode(self.file.as_raw_fd(), &mut opmode as *mut u16)
                .context(ErrorKind::Io)?;
        }

        match opmode {
            0 => Ok(OpMode::Tablet),
            1 => Ok(OpMode::Laptop),
            2 => Ok(OpMode::Studio),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        }
    }
}


ioctl_none!(dtx_latch_lock,    0xa5, 0x23);
ioctl_none!(dtx_latch_unlock,  0xa5, 0x24);
ioctl_none!(dtx_latch_request, 0xa5, 0x25);
ioctl_read!(dtx_get_opmode,    0xa5, 0x2a, u16);
