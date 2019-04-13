use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::io::{Result, Error, ErrorKind, Read, Write};


#[derive(Debug, Copy, Clone)]
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

    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Normal  => "1",
            Mode::Battery => "2",
            Mode::Perf1   => "3",
            Mode::Perf2   => "4",
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl std::str::FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Mode::from_str(s)
            .ok_or(format!("invalid performance mode: '{}'", s))
    }
}


pub struct Device {
    path: PathBuf,
}

impl Device {
    pub fn open() -> Result<Self> {
        Device::open_path("/sys/devices/platform/MSHW0107:00")
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().is_dir() {
            Ok(Device { path: path.as_ref().to_owned() })
        } else {
            Err(Error::new(ErrorKind::NotFound, "No Surface performance-mode device found"))
        }
    }

    pub fn get_mode(&self) -> Result<Mode> {
        use std::ffi::CStr;

        let mut file = OpenOptions::new()
            .read(true)
            .open(self.path.as_path().join("perf_mode"))?;

        let mut buf = [0; 4];
        let len = file.read(&mut buf)?;

        let state = CStr::from_bytes_with_nul(&buf[0..len+1])
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Device returned invalid data"))?
            .to_str()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Device returned invalid data"))?
            .trim();

        Mode::from_str(state)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Device returned invalid data"))
    }

    pub fn set_mode(&self, mode: Mode) -> Result<()> {
        let mode = mode.as_str().as_bytes();

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.path.as_path().join("perf_mode"))?;

        let len = file.write(mode)?;

        if len == mode.len() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Failed to write to device"))
        }
    }
}
