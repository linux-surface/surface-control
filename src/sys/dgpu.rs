use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};

use crate::error::{Error, ErrorKind, Result, ResultExt};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PowerState {
    On,
    Off,
}

impl PowerState {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "0" | "off" => Some(PowerState::Off),
            "1" | "on"  => Some(PowerState::On),
            _           => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PowerState::Off => "off",
            PowerState::On  => "on",
        }
    }
}

impl std::fmt::Display for PowerState {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}


#[derive(Debug)]
pub struct InvalidPowerStateError;

impl std::str::FromStr for PowerState {
    type Err = InvalidPowerStateError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        PowerState::from_str(s).ok_or(InvalidPowerStateError)
    }
}


pub struct Device {
    path: PathBuf,
}

impl Device {
    pub fn open() -> Result<Self> {
        Device::open_path("/sys/devices/platform/MSHW0153:00")
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().is_dir() {
            Ok(Device { path: path.as_ref().to_owned() })
        } else {
            Err(failure::err_msg("Surface dGPU hot-plug device not found"))
                .context(ErrorKind::DeviceAccess)
                .map_err(Into::into)
        }
    }

    pub fn get_power(&self) -> Result<PowerState> {
        use std::ffi::CStr;

        let mut file = OpenOptions::new()
            .read(true)
            .open(self.path.as_path().join("dgpu_power"))
            .context(ErrorKind::DeviceAccess)?;

        let mut buf = [0; 5];
        let len = file.read(&mut buf).context(ErrorKind::Io)?;
        let len = std::cmp::min(len + 1, buf.len());

        let state = CStr::from_bytes_with_nul(&buf[0..len])
            .context(ErrorKind::InvalidData)?
            .to_str().context(ErrorKind::InvalidData)?
            .trim();

        PowerState::from_str(state)
            .ok_or_else(|| Error::from(ErrorKind::InvalidData))
    }

    pub fn set_power(&self, state: PowerState) -> Result<()> {
        let state = state.as_str().as_bytes();

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.path.as_path().join("dgpu_power"))
            .context(ErrorKind::DeviceAccess)?;

        let len = file.write(state).context(ErrorKind::Io)?;

        if len == state.len() {
            Ok(())
        } else {
            Err(Error::from(ErrorKind::Io))
        }
    }
}
