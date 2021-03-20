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

    #[error(transparent)]
    Protocol { #[from] details: ProtocolError, }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum DtxRuntimeError {
    #[error("Detachment preconditions not fulfilled")]
    NotFeasible,

    #[error("Detach operation timed out")]
    Timeout,

    #[error("Unknown error: {0:#04x}")]
    Unknown(u8),
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
    Unknown(u8),
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum ProtocolError {
    #[error("Invalid value for base state: {0:#04x}")]
    BaseState(u8),

    #[error("Invalid value for device mode: {0:#04x}")]
    DeviceMode(u8),

    #[error("Invalid value for latch status: {0:#04x}")]
    LatchStatus(u8),
}

pub type DtxResult<T> = std::result::Result<T, DtxError>;

fn translate_status_code(value: u16) -> DtxResult<u16> {
    match value & uapi::SDTX_CATEGORY_MASK {
        uapi::SDTX_CATEGORY_STATUS => Ok(value),
        uapi::SDTX_CATEGORY_RUNTIME_ERROR => Err(DtxError::RuntimeError {
            details: match value {
                uapi::SDTX_DETACH_NOT_FEASIBLE        => DtxRuntimeError::NotFeasible,
                uapi::SDTX_DETACH_TIMEOUT             => DtxRuntimeError::Timeout,
                v => DtxRuntimeError::Unknown((v & uapi::SDTX_VALUE_MASK) as u8)
            },
        }),
        uapi::SDTX_CATEGORY_HARDWARE_ERROR => Err(DtxError::HardwareError {
            details: match value {
                uapi::SDTX_ERR_FAILED_TO_OPEN         => DtxHardwareError::FailedToOpen,
                uapi::SDTX_ERR_FAILED_TO_REMAIN_OPEN  => DtxHardwareError::FailedToRemainOpen,
                uapi::SDTX_ERR_FAILED_TO_CLOSE        => DtxHardwareError::FailedToClose,
                v => DtxHardwareError::Unknown((v & uapi::SDTX_VALUE_MASK) as u8)
            },
        }),
        uapi::SDTX_CATEGORY_UNKNOWN => Err(DtxError::Unknown(value)),
        _                           => Err(DtxError::Unsupported(value))
    }
}


#[derive(Debug, Clone, Copy)]
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
            v => Err(ProtocolError::DeviceMode(v as u8).into()),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum LatchStatus {
    Closed,
    Opened,
}

impl LatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LatchStatus::Closed => "Closed",
            LatchStatus::Opened => "Opened",
        }
    }
}

impl std::fmt::Display for LatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<u16> for LatchStatus {
    type Error = DtxError;

    fn try_from(value: u16) -> DtxResult<Self> {
        match translate_status_code(value)? {
            uapi::SDTX_LATCH_CLOSED => Ok(LatchStatus::Closed),
            uapi::SDTX_LATCH_OPENED => Ok(LatchStatus::Opened),
            v => Err(ProtocolError::LatchStatus(v as u8).into()),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum BaseState {
    Detached,
    Attached,
}

impl BaseState {
    pub fn as_str(&self) -> &'static str {
        match self {
            BaseState::Detached => "Detached",
            BaseState::Attached => "Attached",
        }
    }
}

impl std::fmt::Display for BaseState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<u16> for BaseState {
    type Error = DtxError;

    fn try_from(value: u16) -> DtxResult<Self> {
        match translate_status_code(value)? {
            uapi::SDTX_BASE_DETACHED => Ok(BaseState::Detached),
            uapi::SDTX_BASE_ATTACHED => Ok(BaseState::Attached),
            v => Err(ProtocolError::BaseState(v as u8).into()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    Hid,
    Ssh,
    Unknown(u8),
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DeviceType::Hid        => write!(f, "HID"),
            DeviceType::Ssh        => write!(f, "SSH"),
            DeviceType::Unknown(v) => write!(f, "{:#02x}", v),
        }
    }
}

impl From<u16> for DeviceType {
    fn from(value: u16) -> Self {
        match value & uapi::SDTX_DEVICE_TYPE_MASK {
            uapi::SDTX_DEVICE_TYPE_HID => DeviceType::Hid,
            uapi::SDTX_DEVICE_TYPE_SSH => DeviceType::Ssh,
            v                          => DeviceType::Unknown((v >> 8) as u8)
        }
    }
}


#[derive(Debug)]
pub struct BaseInfo {
    pub state:       BaseState,
    pub device_type: DeviceType,
    pub id:          u8,
}

impl TryFrom<uapi::BaseInfo> for BaseInfo {
    type Error = DtxError;

    fn try_from(value: uapi::BaseInfo) -> DtxResult<Self> {
        let state = BaseState::try_from(value.state)?;
        let device_type = DeviceType::from(value.base_id);
        let id = (value.base_id & 0xff) as u8;

        Ok(BaseInfo { state, device_type, id })
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

    pub fn latch_confirm(&self) -> Result<()> {
        unsafe { uapi::dtx_latch_confirm(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn latch_heartbeat(&self) -> Result<()> {
        unsafe { uapi::dtx_latch_heartbeat(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn latch_cancel(&self) -> Result<()> {
        unsafe { uapi::dtx_latch_cancel(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    pub fn get_base_info(&self) -> Result<BaseInfo> {
        let mut info = uapi::BaseInfo { state: 0, base_id: 0 };

        unsafe { uapi::dtx_get_base_info(self.file.as_raw_fd(), &mut info as *mut uapi::BaseInfo) }
            .map_err(|source| Error::IoctlError { source })?;

        BaseInfo::try_from(info)
            .map_err(|source| Error::DtxError { source })
    }

    pub fn get_device_mode(&self) -> Result<DeviceMode> {
        let mut mode: u16 = 0;

        unsafe { uapi::dtx_get_device_mode(self.file.as_raw_fd(), &mut mode as *mut u16) }
            .map_err(|source| Error::IoctlError { source })?;

        DeviceMode::try_from(mode)
            .map_err(|source| Error::DtxError { source })
    }

    pub fn get_latch_status(&self) -> Result<LatchStatus> {
        let mut status: u16 = 0;

        unsafe { uapi::dtx_get_latch_status(self.file.as_raw_fd(), &mut status as *mut u16) }
            .map_err(|source| Error::IoctlError { source })?;

        LatchStatus::try_from(status)
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

    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct EventHeader {
        pub length: u16,
        pub code: u16,
    }

    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct BaseInfo {
        pub state: u16,
        pub base_id: u16,
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
