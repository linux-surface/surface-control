use std::{convert::TryFrom, fs::File};
use std::path::Path;
use std::os::unix::io::AsRawFd;

use crate::sys::{Error, Result};


#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum DtxError {
    #[error("Runtime error: {details}")]
    RuntimeError { details: DtxRuntimeError },

    #[error("Hardware error: {details}")]
    HardwareError { details: DtxHardwareError },

    #[error("Unknown firmware status code: {0:#04x}")]
    Unknown(u16),

    #[error("Unknown error: {0:#04x}")]
    Unsupported(u16),

    #[error("Invalid value: {0:#04x}")]
    Invalid(u16),
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum DtxRuntimeError {
    #[error("Detachment preconditions not fulfilled")]
    NotFeasible,

    #[error("Detach operation timed out")]
    Timeout,

    #[error("Unknown error: {0:#04x}")]
    Unknown(u16),
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum DtxHardwareError {
    #[error("Failed to open latch")]
    FailedToOpen,

    #[error("Latch failed to remain open")]
    FailedToRemainOpen,

    #[error("Failed to close latch")]
    FailedToClose,

    #[error("Unknown error: {0:#04x}")]
    Unknown(u16),
}

pub type DtxResult<T> = std::result::Result<T, DtxError>;

fn translate_status_code(value: u16) -> DtxResult<u16> {
    match value & uapi::SDTX_CATEGORY_MASK {
        uapi::SDTX_CATEGORY_STATUS => Ok(value),
        uapi::SDTX_CATEGORY_RUNTIME_ERROR => Err(DtxError::RuntimeError {
            details: match value {
                uapi::SDTX_DETACH_NOT_FEASIBLE        => DtxRuntimeError::NotFeasible,
                uapi::SDTX_DETACH_TIMEOUT             => DtxRuntimeError::Timeout,
                v                                     => DtxRuntimeError::Unknown(v)
            },
        }),
        uapi::SDTX_CATEGORY_HARDWARE_ERROR => Err(DtxError::HardwareError {
            details: match value {
                uapi::SDTX_ERR_FAILED_TO_OPEN         => DtxHardwareError::FailedToOpen,
                uapi::SDTX_ERR_FAILED_TO_REMAIN_OPEN  => DtxHardwareError::FailedToRemainOpen,
                uapi::SDTX_ERR_FAILED_TO_CLOSE        => DtxHardwareError::FailedToClose,
                v                                     => DtxHardwareError::Unknown(v)
            },
        }),
        uapi::SDTX_CATEGORY_UNKNOWN => Err(DtxError::Unknown(value)),
        _                           => Err(DtxError::Unsupported(value))
    }
}


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

impl TryFrom<u16> for DeviceMode {
    type Error = DtxError;

    fn try_from(value: u16) -> DtxResult<Self> {
        match translate_status_code(value)? {
            uapi::SDTX_DEVICE_MODE_TABLET => Ok(DeviceMode::Tablet),
            uapi::SDTX_DEVICE_MODE_LAPTOP => Ok(DeviceMode::Laptop),
            uapi::SDTX_DEVICE_MODE_STUDIO => Ok(DeviceMode::Studio),
            v                             => Err(DtxError::Invalid(v)),     // TODO: add info about type of value
        }
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
        unsafe { uapi::dtx_latch_lock(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn latch_unlock(&self) -> Result<()> {
        unsafe { uapi::dtx_latch_unlock(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn latch_request(&self) -> Result<()> {
        unsafe { uapi::dtx_latch_request(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn get_device_mode(&self) -> Result<DeviceMode> {
        let mut mode: u16 = 0;

        unsafe { uapi::dtx_get_device_mode(self.file.as_raw_fd(), &mut mode as *mut u16) }
            .map_err(|source| Error::IoctlError { source })?;

        DeviceMode::try_from(mode)
            .map_err(|source| Error::DtxError { source })
    }
}


#[allow(unused)]
mod uapi {
    use nix::{ioctl_none, ioctl_read};

    // status/error categories
    pub const SDTX_CATEGORY_STATUS: u16           = 0x0000;
    pub const SDTX_CATEGORY_RUNTIME_ERROR: u16    = 0x1000;
    pub const SDTX_CATEGORY_HARDWARE_ERROR: u16   = 0x2000;
    pub const SDTX_CATEGORY_UNKNOWN: u16          = 0xf000;

    pub const SDTX_CATEGORY_MASK: u16             = 0xf000;
    pub const SDTX_VALUE_MASK: u16                = 0x0fff;

    // latch status values
    pub const SDTX_LATCH_CLOSED: u16              = SDTX_CATEGORY_STATUS | 0x00;
    pub const SDTX_LATCH_OPENED: u16              = SDTX_CATEGORY_STATUS | 0x01;

    // base status values
    pub const SDTX_BASE_DETACHED: u16             = SDTX_CATEGORY_STATUS | 0x00;
    pub const SDTX_BASE_ATTACHED: u16             = SDTX_CATEGORY_STATUS | 0x01;

    // runtime errors (non-critical)
    pub const SDTX_DETACH_NOT_FEASIBLE: u16       = SDTX_CATEGORY_RUNTIME_ERROR | 0x01;
    pub const SDTX_DETACH_TIMEOUT: u16            = SDTX_CATEGORY_RUNTIME_ERROR | 0x02;

    // hardware errors (critical)
    pub const SDTX_ERR_FAILED_TO_OPEN: u16        = SDTX_CATEGORY_HARDWARE_ERROR | 0x01;
    pub const SDTX_ERR_FAILED_TO_REMAIN_OPEN: u16 = SDTX_CATEGORY_HARDWARE_ERROR | 0x02;
    pub const SDTX_ERR_FAILED_TO_CLOSE: u16       = SDTX_CATEGORY_HARDWARE_ERROR | 0x03;

    // base types
    pub const SDTX_DEVICE_TYPE_HID: u16           = 0x0100;
    pub const SDTX_DEVICE_TYPE_SSH: u16           = 0x0200;

    pub const SDTX_DEVICE_TYPE_MASK: u16          = 0x0f00;

    // device mode
    pub const SDTX_DEVICE_MODE_TABLET: u16        = 0x00;
    pub const SDTX_DEVICE_MODE_LAPTOP: u16        = 0x01;
    pub const SDTX_DEVICE_MODE_STUDIO: u16        = 0x02;

    // event code
    pub const SDTX_EVENT_REQUEST: u16             = 1;
    pub const SDTX_EVENT_CANCEL: u16              = 2;
    pub const SDTX_EVENT_BASE_CONNECTION: u16     = 3;
    pub const SDTX_EVENT_LATCH_STATUS: u16        = 4;
    pub const SDTX_EVENT_DEVICE_MODE: u16         = 5;

    #[derive(Debug)]
    #[repr(C)]
    pub struct EventHeader {
        length: u16,
        code: u16,
    }

    #[derive(Debug)]
    #[repr(C)]
    pub struct BaseInfo {
        state: u16,
        base_id: u16,
    }

    ioctl_none!(dtx_events_enable,    0xa5, 0x21);
    ioctl_none!(dtx_events_disable,   0xa5, 0x22);

    ioctl_none!(dtx_latch_lock,       0xa5, 0x23);
    ioctl_none!(dtx_latch_unlock,     0xa5, 0x24);

    ioctl_none!(dtx_latch_request,    0xa5, 0x25);
    ioctl_none!(dtx_latch_confirm,    0xa5, 0x26);
    ioctl_none!(dtx_latch_heartbeat,  0xa5, 0x27);
    ioctl_none!(dtx_latch_cancel,     0xa5, 0x28);

    ioctl_read!(dtx_get_base_info,    0xa5, 0x29, BaseInfo);
    ioctl_read!(dtx_get_device_mode,  0xa5, 0x2a, u16);
    ioctl_read!(dtx_get_latch_status, 0xa5, 0x2b, u16);
}
