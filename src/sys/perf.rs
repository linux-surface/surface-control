use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};

use crate::error::{Error, ErrorKind, Result, ResultExt};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Battery,
    Perf1,
    Perf2,
}

impl Mode {
    pub fn from_str(s: &str) -> Option<Self> {
        // TODO: handle other strings?

        match s {
            "1" => Some(Mode::Normal),
            "2" => Some(Mode::Battery),
            "3" => Some(Mode::Perf1),
            "4" => Some(Mode::Perf2),
            _   => None,
        }
    }

    pub fn short_str(self) -> &'static str {
        match self {
            Mode::Normal  => "1",
            Mode::Battery => "2",
            Mode::Perf1   => "3",
            Mode::Perf2   => "4",
        }
    }

    pub fn long_str(self) -> &'static str {
        match self {
            Mode::Normal  => "Normal",
            Mode::Battery => "Battery-Saver",
            Mode::Perf1   => "Better Perormance",
            Mode::Perf2   => "Best Performance",
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
        Device::open_path("/sys/bus/platform/devices/surface_sam_sid_perfmode")
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().is_dir() {
            Ok(Device { path: path.as_ref().to_owned() })
        } else {
            Err(failure::err_msg("Surface performance-mode device not found"))
                .context(ErrorKind::DeviceAccess)
                .map_err(Into::into)
        }
    }

    pub fn get_mode(&self) -> Result<Mode> {
        use std::ffi::CStr;

        let mut file = OpenOptions::new()
            .read(true)
            .open(self.path.as_path().join("perf_mode"))
            .context(ErrorKind::DeviceAccess)?;

        let mut buf = [0; 4];
        let len = file.read(&mut buf).context(ErrorKind::Io)?;
        let len = std::cmp::min(len + 1, buf.len());

        let state = CStr::from_bytes_with_nul(&buf[0..len])
            .context(ErrorKind::InvalidData)?
            .to_str().context(ErrorKind::InvalidData)?
            .trim();

        Mode::from_str(state)
            .ok_or_else(|| Error::from(ErrorKind::InvalidData))
    }

    pub fn set_mode(&self, mode: Mode) -> Result<()> {
        let mode = mode.short_str().as_bytes();

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.path.as_path().join("perf_mode"))
            .context(ErrorKind::DeviceAccess)?;

        let len = file.write(mode).context(ErrorKind::Io)?;

        if len == mode.len() {
            Ok(())
        } else {
            Err(Error::from(ErrorKind::Io))
        }
    }
}
