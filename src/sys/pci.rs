use std::{convert::TryFrom, ffi::{OsStr, OsString}, str::FromStr};

use crate::sys::{Error, Result};


#[derive(thiserror::Error, Debug)]
pub enum SysFsError {
    #[error("Attribute \"{attribute}\" not avalable")]
    MissingAttribute { attribute: &'static str },

    #[error("Invalid value {value:?} for attribute \"{attribute}\"")]
    InvalidAttributeValue { attribute: &'static str, value: OsString },
}

pub type SysFsResult<T> = std::result::Result<T, SysFsError>;


pub const VENDOR_ID_NVIDIA: u16 = 0x10de;

pub const BASE_CLASS_DISPLAY: u8 = 0x03;


#[derive(Debug, Clone, Copy)]
pub struct Class {
    pub base:  u8,
    pub sub:   u8,
    pub iface: u8,
}


#[derive(Debug, Clone, Copy)]
pub enum PowerState {
    Unknown,
    Error,
    D0,
    D1,
    D2,
    D3hot,
    D3cold,
}

impl PowerState {
    fn from_sysfs(value: &OsStr) -> SysFsResult<PowerState> {
        let attribute = "power_state";

        let value = value.to_str()
            .ok_or_else(|| SysFsError::InvalidAttributeValue { attribute, value: value.into() })?;

        match value {
            "unknown" => Ok(PowerState::Unknown),
            "error"   => Ok(PowerState::Error),
            "D0"      => Ok(PowerState::D0),
            "D1"      => Ok(PowerState::D1),
            "D2"      => Ok(PowerState::D2),
            "D3hot"   => Ok(PowerState::D3hot),
            "D3cold"  => Ok(PowerState::D3cold),
            v => Err(SysFsError::InvalidAttributeValue { attribute, value: v.into() }),
        }
    }
}

impl std::fmt::Display for PowerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Error   => write!(f, "Error"),
            Self::D0      => write!(f, "D0"),
            Self::D1      => write!(f, "D1"),
            Self::D2      => write!(f, "D2"),
            Self::D3hot   => write!(f, "D3hot"),
            Self::D3cold  => write!(f, "D3cold"),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum RuntimePowerManagement {
    On,
    Off
}

impl std::fmt::Display for RuntimePowerManagement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::On  => write!(f, "On"),
            Self::Off => write!(f, "Off"),
        }
    }
}

impl RuntimePowerManagement {
    fn from_sysfs(value: &OsStr) -> SysFsResult<RuntimePowerManagement> {
        let attribute = "power/control";

        let value = value.to_str()
            .ok_or_else(|| SysFsError::InvalidAttributeValue { attribute, value: value.into() })?;

        match value {
            "auto" => Ok(RuntimePowerManagement::On),
            "on"   => Ok(RuntimePowerManagement::Off),
            v => Err(SysFsError::InvalidAttributeValue { attribute, value: v.into() }),
        }
    }

    fn as_sysfs(&self) -> &'static str {
        match self {
            Self::On  => "auto",
            Self::Off => "on",
        }
    }
}

#[derive(Debug)]
pub struct InvalidRuntimePmError;

impl FromStr for RuntimePowerManagement {
    type Err = InvalidRuntimePmError;

    fn from_str(s: &str) -> std::result::Result<Self, InvalidRuntimePmError> {
        match s {
            "on" | "On" | "ON"    => Ok(Self::On),
            "off" | "Off" | "OFF" => Ok(Self::Off),
            _ => Err(InvalidRuntimePmError),
        }
    }
}


pub struct PciDevice {
    base: udev::Device,
}

impl PciDevice {
    #[allow(unused)]
    pub fn base(&self) -> &udev::Device {
        &self.base
    }

    pub fn vendor_id(&self) -> SysFsResult<u16> {
        let attribute = "vendor";

        let id = self.base.attribute_value(attribute)
            .ok_or(SysFsError::MissingAttribute { attribute })?;

        let id = id.to_str()
            .ok_or_else(|| SysFsError::InvalidAttributeValue { attribute, value: id.into() })?
            .trim_start_matches("0x");

        u16::from_str_radix(id, 16)
            .map_err(|_| SysFsError::InvalidAttributeValue { attribute, value: id.into() })
    }

    pub fn device_id(&self) -> SysFsResult<u16> {
        let attribute = "device";

        let id = self.base.attribute_value(attribute)
            .ok_or(SysFsError::MissingAttribute { attribute })?;

        let id = id.to_str()
            .ok_or_else(|| SysFsError::InvalidAttributeValue { attribute, value: id.into() })?
            .trim_start_matches("0x");

        u16::from_str_radix(id, 16)
            .map_err(|_| SysFsError::InvalidAttributeValue { attribute, value: id.into() })
    }

    pub fn class(&self) -> SysFsResult<Class> {
        let attribute = "class";

        let id = self.base.attribute_value(attribute)
            .ok_or(SysFsError::MissingAttribute { attribute })?;

        let id = id.to_str()
            .ok_or_else(|| SysFsError::InvalidAttributeValue { attribute, value: id.into() })?
            .trim_start_matches("0x");

        let id = u32::from_str_radix(id, 16)
            .map_err(|_| SysFsError::InvalidAttributeValue { attribute, value: id.into() })?;

        Ok(Class {
            base:  ((id >> 16) & 0xff) as u8,
            sub:   ((id >> 8) & 0xff) as u8,
            iface: (id & 0xff) as u8,
        })
    }

    pub fn get_power_state(&self) -> SysFsResult<PowerState> {
        let attribute = "power_state";

        PowerState::from_sysfs(self.base.attribute_value(attribute)
            .ok_or(SysFsError::MissingAttribute { attribute })?)
    }

    pub fn get_runtime_pm(&self) -> SysFsResult<RuntimePowerManagement> {
        let attribute = "power/control";

        RuntimePowerManagement::from_sysfs(self.base.attribute_value(attribute)
            .ok_or(SysFsError::MissingAttribute { attribute })?)
    }

    pub fn set_runtime_pm(&mut self, state: RuntimePowerManagement) -> Result<()> {
        self.base.set_attribute_value("power/control", state.as_sysfs())
            .map_err(|source| {
                // WORKAROUND: Someone didn't invert the errno in the udev crate...
                if let Some(errno) = source.raw_os_error()  {
                    Error::Io { source: std::io::Error::from_raw_os_error(-errno) }
                } else {
                    Error::Io { source }
                }
            })
    }
}

impl TryFrom<udev::Device> for PciDevice {
    type Error = SysFsError;

    fn try_from(base: udev::Device) -> SysFsResult<Self> {
        match base.subsystem() {
            Some(x) if x == "pci" => Ok(PciDevice { base }),
            Some(x) => Err(SysFsError::InvalidAttributeValue { attribute: "pci", value: x.into() }),
            None    => Err(SysFsError::InvalidAttributeValue { attribute: "pci", value: "".into() }),
        }
    }
}
