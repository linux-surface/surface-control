use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::sys::{Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Battery,
    Perf1,
    Perf2,
}

impl Mode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1" => Some(Mode::Normal),
            "2" => Some(Mode::Battery),
            "3" => Some(Mode::Perf1),
            "4" => Some(Mode::Perf2),
            _ => None,
        }
    }

    pub fn short_str(self) -> &'static str {
        match self {
            Mode::Normal => "1",
            Mode::Battery => "2",
            Mode::Perf1 => "3",
            Mode::Perf2 => "4",
        }
    }

    pub fn long_str(self) -> &'static str {
        match self {
            Mode::Normal => "Normal",
            Mode::Battery => "Battery-Saver",
            Mode::Perf1 => "Better Performance",
            Mode::Perf2 => "Best Performance",
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.long_str())
    }
}

#[derive(Debug)]
pub struct InvalidPerformanceModeError;

impl std::str::FromStr for Mode {
    type Err = InvalidPerformanceModeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Mode::from_str(s).ok_or(InvalidPerformanceModeError)
    }
}

pub struct Device {
    path: PathBuf,
}

impl Device {
    pub fn open() -> Result<Self> {
        Device::open_path("/sys/bus/surface_aggregator/devices/01:03:01:00:01")
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().is_dir() {
            Ok(Device {
                path: path.as_ref().to_owned(),
            })
        } else {
            use std::io;

            Err(Error::DeviceAccess {
                source: io::Error::new(io::ErrorKind::NotFound, "No such file or directory"),
                device: path.as_ref().to_owned(),
            })
        }
    }

    pub fn get_mode(&self) -> Result<Mode> {
        let attribute = "perf_mode";

        let mut file = OpenOptions::new()
            .read(true)
            .open(self.path.as_path().join(attribute))
            .map_err(|source| Error::DeviceAccess {
                source,
                device: self.path.as_path().join(attribute),
            })?;

        let mut buf = [0; 4];
        let len = file
            .read(&mut buf)
            .map_err(|source| Error::IoError { source })?;
        let len = std::cmp::min(len + 1, buf.len());

        let state = std::ffi::CStr::from_bytes_with_nul(&buf[0..len])
            .map_err(|_| Error::InvalidData)?
            .to_str()
            .map_err(|_| Error::InvalidData)?
            .trim();

        Mode::from_str(state).ok_or_else(|| Error::InvalidData)
    }

    pub fn set_mode(&self, mode: Mode) -> Result<()> {
        let attribute = "perf_mode";

        let mode = mode.short_str().as_bytes();

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.path.as_path().join(attribute))
            .map_err(|source| Error::DeviceAccess {
                source,
                device: self.path.as_path().join(attribute),
            })?;

        let len = file
            .write(mode)
            .map_err(|source| Error::IoError { source })?;

        if len == mode.len() {
            Ok(())
        } else {
            use std::io;

            Err(Error::IoError {
                source: io::Error::new(io::ErrorKind::WriteZero, "Write failed"),
            })
        }
    }
}
