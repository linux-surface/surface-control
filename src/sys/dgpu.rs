use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::io::{Result, Error, ErrorKind, Read, Write};


#[derive(Debug, Copy, Clone)]
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

    pub fn as_str(&self) -> &'static str {
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

impl std::str::FromStr for PowerState {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        PowerState::from_str(s)
            .ok_or(format!("invalid power state: '{}'", s))
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
            Err(Error::new(ErrorKind::NotFound, "No Surface dGPU device found"))
        }
    }

    pub fn get_power(&self) -> Result<PowerState> {
        use std::ffi::CStr;

        let mut file = OpenOptions::new()
            .read(true)
            .open(self.path.as_path().join("dgpu_power"))?;

        let mut buf = [0; 4];
        let len = file.read(&mut buf)?;

        let state = CStr::from_bytes_with_nul(&buf[0..len+1])
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Device returned invalid data"))?
            .to_str()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Device returned invalid data"))?
            .trim();

        PowerState::from_str(state)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Device returned invalid data"))
    }

    pub fn set_power(&self, state: PowerState) -> Result<()> {
        let state = state.as_str().as_bytes();

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.path.as_path().join("dgpu_power"))?;

        let len = file.write(state)?;

        if len == state.len() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Failed to write to device"))
        }
    }
}
