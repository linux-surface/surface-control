use std::{fs::OpenOptions, io::{Read, Write}};
use std::path::{Path, PathBuf};

use crate::sys::{Error, Result};


pub struct Device {
    path: PathBuf,
}

impl Device {
    pub fn open() -> Result<Self> {
        Device::open_path("/sys/firmware/acpi/")
    }

    pub fn open_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        use std::io;

        if path.as_ref().is_dir() && path.as_ref().join("platform_profile").is_file()
            && path.as_ref().join("platform_profile_choices").is_file()
        {
            Ok(Device {
                path: path.as_ref().to_owned(),
            })
        } else {
            Err(Error::DeviceAccess {
                source: io::Error::new(io::ErrorKind::NotFound, "No platform profile support found"),
                device: path.as_ref().to_owned(),
            })
        }
    }

    pub fn get(&self) -> Result<String> {
        let attribute = "platform_profile";

        let mut file = OpenOptions::new()
            .read(true)
            .open(self.path.as_path().join(attribute))
            .map_err(|source| Error::DeviceAccess {
                source,
                device: self.path.as_path().join(attribute),
            })?;

        let mut profile = String::new();
        file.read_to_string(&mut profile)
            .map_err(|e| Error::IoError { source: e })?;

        Ok(profile.trim().to_owned())
    }

    pub fn set(&self, profile: &str) -> Result<()> {
        let attribute = "platform_profile";

        let mut file = OpenOptions::new()
            .write(true)
            .open(self.path.as_path().join(attribute))
            .map_err(|source| Error::DeviceAccess {
                source,
                device: self.path.as_path().join(attribute),
            })?;

        file.write(profile.as_bytes())
            .map_err(|e| Error::IoError { source: e })?;

        Ok(())
    }

    pub fn get_supported(&self) -> Result<Vec<String>> {
        let attribute = "platform_profile_choices";

        let mut file = OpenOptions::new()
            .read(true)
            .open(self.path.as_path().join(attribute))
            .map_err(|source| Error::DeviceAccess {
                source,
                device: self.path.as_path().join(attribute),
            })?;

        let mut supported = String::new();
        file.read_to_string(&mut supported)
            .map_err(|e| Error::IoError { source: e })?;

        let supported = supported.split_ascii_whitespace()
            .map(|s| s.trim().to_owned())
            .collect();

        Ok(supported)
    }
}