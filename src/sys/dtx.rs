use std::{convert::{TryFrom, TryInto}, fs::File, io::{BufReader, Read}};
use std::path::Path;
use std::os::unix::io::AsRawFd;

use smallvec::SmallVec;

use crate::sys::{Error, Result};


#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum DtxError {
    #[error("Invalid value for base state: {0:#04x}")]
    InvalidBaseState(u16),

    #[error("Invalid value for device mode: {0:#04x}")]
    InvalidDeviceMode(u16),

    #[error("Invalid value for latch status: {0:#04x}")]
    InvalidLatchStatus(u16),

    #[error("Invalid value for cancel reason: {0:#04x}")]
    InvalidCancelReason(u16),
}

pub type DtxResult<T> = std::result::Result<T, DtxError>;

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum RuntimeError {
    #[error("Detachment preconditions not fulfilled")]
    NotFeasible,

    #[error("Detach operation timed out")]
    Timeout,

    #[error("Unknown runtime error: {0:#04x}")]
    Unknown(u8),
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum HardwareError {
    #[error("Failed to open latch")]
    FailedToOpen,

    #[error("Latch failed to remain open")]
    FailedToRemainOpen,

    #[error("Failed to close latch")]
    FailedToClose,

    #[error("Unknown hardware error: {0:#04x}")]
    Unknown(u8),
}


#[derive(Debug, Clone, Copy)]
pub enum DeviceMode {
    Tablet,
    Laptop,
    Studio,
}

impl std::fmt::Display for DeviceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            DeviceMode::Tablet => "Tablet",
            DeviceMode::Laptop => "Laptop",
            DeviceMode::Studio => "Studio",
        };

        write!(f, "{}", name)
    }
}

impl TryFrom<u16> for DeviceMode {
    type Error = DtxError;

    fn try_from(value: u16) -> DtxResult<Self> {
        match value {
            uapi::SDTX_DEVICE_MODE_TABLET => Ok(DeviceMode::Tablet),
            uapi::SDTX_DEVICE_MODE_LAPTOP => Ok(DeviceMode::Laptop),
            uapi::SDTX_DEVICE_MODE_STUDIO => Ok(DeviceMode::Studio),
            v => Err(DtxError::InvalidDeviceMode(v)),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum LatchStatus {
    Closed,
    Opened,
    Error(HardwareError),
}

impl std::fmt::Display for LatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LatchStatus::Closed     => write!(f, "Closed"),
            LatchStatus::Opened     => write!(f, "Opened"),
            LatchStatus::Error(err) => write!(f, "Error: {}", err),
        }
    }
}

impl TryFrom<u16> for LatchStatus {
    type Error = DtxError;

    fn try_from(value: u16) -> DtxResult<Self> {
        match value {
            uapi::SDTX_LATCH_CLOSED => {
                Ok(LatchStatus::Closed)
            },
            uapi::SDTX_LATCH_OPENED => {
                Ok(LatchStatus::Opened)
            },
            uapi::SDTX_ERR_FAILED_TO_OPEN => {
                Ok(LatchStatus::Error(HardwareError::FailedToOpen))
            },
            uapi::SDTX_ERR_FAILED_TO_REMAIN_OPEN => {
                Ok(LatchStatus::Error(HardwareError::FailedToRemainOpen))
            },
            uapi::SDTX_ERR_FAILED_TO_CLOSE => {
                Ok(LatchStatus::Error(HardwareError::FailedToClose))
            },
            v if v & uapi::SDTX_CATEGORY_MASK == uapi::SDTX_CATEGORY_HARDWARE_ERROR => {
                Ok(LatchStatus::Error(HardwareError::Unknown(v as u8)))
            },
            v => {
                Err(DtxError::InvalidLatchStatus(v))
            },
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum BaseState {
    Detached,
    Attached,
    NotFeasible,
}

impl std::fmt::Display for BaseState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            BaseState::Detached    => "Detached",
            BaseState::Attached    => "Attached",
            BaseState::NotFeasible => "NotFeasible",
        };

        write!(f, "{}", name)
    }
}

impl TryFrom<u16> for BaseState {
    type Error = DtxError;

    fn try_from(value: u16) -> DtxResult<Self> {
        match value {
            uapi::SDTX_BASE_DETACHED       => Ok(BaseState::Detached),
            uapi::SDTX_BASE_ATTACHED       => Ok(BaseState::Attached),
            uapi::SDTX_DETACH_NOT_FEASIBLE => Ok(BaseState::NotFeasible),
            v => Err(DtxError::InvalidBaseState(v)),
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


#[derive(Debug, Clone, Copy)]
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

pub enum CancelReason {
    Runtime(RuntimeError),
    Hardware(HardwareError),
}

#[derive(Debug, Clone)]
pub enum Event {
    Request,
    Cancel { reason: event::CancelReason },
    BaseConnection { state: event::BaseState, device_type: DeviceType, id: u8 },
    LatchStatus { status: event::LatchStatus },
    DeviceMode { mode: event::DeviceMode },
    Unknown { code: u16, data: Vec<u8> },
}

mod event {
    use super::*;

    use std::convert::TryInto;


    #[derive(Debug, Clone, Copy)]
    pub enum CancelReason {
        Runtime(RuntimeError),
        Hardware(HardwareError),
        Unknown(u16),
    }

    impl From<u16> for CancelReason {
        fn from(value: u16) -> Self {
            match value & uapi::SDTX_CATEGORY_MASK {
                uapi::SDTX_CATEGORY_RUNTIME_ERROR => match value {
                    uapi::SDTX_DETACH_NOT_FEASIBLE => Self::Runtime(RuntimeError::NotFeasible),
                    uapi::SDTX_DETACH_TIMEOUT      => Self::Runtime(RuntimeError::Timeout),
                    x                              => Self::Runtime(RuntimeError::Unknown(x as u8)),
                },
                uapi::SDTX_CATEGORY_HARDWARE_ERROR => match value {
                    uapi::SDTX_ERR_FAILED_TO_OPEN        => Self::Hardware(HardwareError::FailedToOpen),
                    uapi::SDTX_ERR_FAILED_TO_REMAIN_OPEN => Self::Hardware(HardwareError::FailedToRemainOpen),
                    uapi::SDTX_ERR_FAILED_TO_CLOSE       => Self::Hardware(HardwareError::FailedToClose),
                    x                                    => Self::Hardware(HardwareError::Unknown(x as u8)),
                },
                x => Self::Unknown(x),
            }
        }
    }

    impl TryInto<super::CancelReason> for CancelReason {
        type Error = DtxError;

        fn try_into(self) -> DtxResult<super::CancelReason> {
            match self {
                Self::Runtime(err)  => Ok(super::CancelReason::Runtime(err)),
                Self::Hardware(err) => Ok(super::CancelReason::Hardware(err)),
                Self::Unknown(err)  => Err(DtxError::InvalidCancelReason(err)),
            }
        }
    }


    #[derive(Debug, Clone, Copy)]
    pub enum BaseState {
        Detached,
        Attached,
        NotFeasible,
        Unknown(u16),
    }

    impl From<u16> for BaseState {
        fn from(value: u16) -> Self {
            match value {
                uapi::SDTX_BASE_DETACHED       => Self::Detached,
                uapi::SDTX_BASE_ATTACHED       => Self::Attached,
                uapi::SDTX_DETACH_NOT_FEASIBLE => Self::NotFeasible,
                x                              => Self::Unknown(x),
            }
        }
    }

    impl TryInto<super::BaseState> for BaseState {
        type Error = DtxError;

        fn try_into(self) -> DtxResult<super::BaseState> {
            match self {
                Self::Detached     => Ok(super::BaseState::Detached),
                Self::Attached     => Ok(super::BaseState::Attached),
                Self::NotFeasible  => Ok(super::BaseState::NotFeasible),
                Self::Unknown(err) => Err(DtxError::InvalidBaseState(err)),
            }
        }
    }


    #[derive(Debug, Clone, Copy)]
    pub enum LatchStatus {
        Closed,
        Opened,
        Error(HardwareError),
        Unknown(u16),
    }

    impl From<u16> for LatchStatus {
        fn from(value: u16) -> Self {
            match value & uapi::SDTX_CATEGORY_MASK {
                uapi::SDTX_CATEGORY_HARDWARE_ERROR => match value {
                    uapi::SDTX_ERR_FAILED_TO_OPEN        => Self::Error(HardwareError::FailedToOpen),
                    uapi::SDTX_ERR_FAILED_TO_REMAIN_OPEN => Self::Error(HardwareError::FailedToRemainOpen),
                    uapi::SDTX_ERR_FAILED_TO_CLOSE       => Self::Error(HardwareError::FailedToClose),
                    x                                    => Self::Error(HardwareError::Unknown(x as u8)),
                },
                uapi::SDTX_CATEGORY_STATUS => match value {
                    uapi::SDTX_LATCH_CLOSED => Self::Closed,
                    uapi::SDTX_LATCH_OPENED => Self::Opened,
                    x                       => Self::Unknown(x),
                },
                x => Self::Unknown(x),
            }
        }
    }

    impl TryInto<super::LatchStatus> for LatchStatus {
        type Error = DtxError;

        fn try_into(self) -> DtxResult<super::LatchStatus> {
            match self {
                Self::Closed       => Ok(super::LatchStatus::Closed),
                Self::Opened       => Ok(super::LatchStatus::Opened),
                Self::Error(err)   => Ok(super::LatchStatus::Error(err)),
                Self::Unknown(err) => Err(DtxError::InvalidLatchStatus(err)),
            }
        }
    }


    #[derive(Debug, Clone, Copy)]
    pub enum DeviceMode {
        Tablet,
        Laptop,
        Studio,
        Unknown(u16),
    }

    impl From<u16> for DeviceMode {
        fn from(value: u16) -> Self {
            match value {
                uapi::SDTX_DEVICE_MODE_TABLET => Self::Tablet,
                uapi::SDTX_DEVICE_MODE_LAPTOP => Self::Laptop,
                uapi::SDTX_DEVICE_MODE_STUDIO => Self::Studio,
                x                             => Self::Unknown(x),
            }
        }
    }

    impl TryInto<super::DeviceMode> for DeviceMode {
        type Error = DtxError;

        fn try_into(self) -> DtxResult<super::DeviceMode> {
            match self {
                Self::Tablet       => Ok(super::DeviceMode::Tablet),
                Self::Laptop       => Ok(super::DeviceMode::Laptop),
                Self::Studio       => Ok(super::DeviceMode::Studio),
                Self::Unknown(err) => Err(DtxError::InvalidDeviceMode(err)),
            }
        }
    }
}


#[derive(Debug)]
pub struct Device {
    file: File,
}

impl Device {
    const DEFAULT_DEVICE_FILE_PATH: &'static str = "/dev/surface/dtx";

    pub fn open() -> Result<Self> {
        Device::open_path(Device::DEFAULT_DEVICE_FILE_PATH)
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

    pub fn events(&mut self) -> Result<EventStream> {
        EventStream::from_device(self)
    }

    fn events_enable(&self) -> Result<()> {
        unsafe { uapi::dtx_events_enable(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }

    fn events_disable(&self) -> Result<()> {
        unsafe { uapi::dtx_events_disable(self.file.as_raw_fd()) }
            .map_err(|source| Error::IoctlError { source })
            .map(|_| ())
    }
}


#[derive(Debug)]
pub struct EventStream<'a> {
    device: &'a mut Device,
    reader: BufReader<File>,
}

impl<'a> EventStream<'a> {
    fn from_device(device: &'a mut Device) -> Result<Self> {
        let file = device.file.try_clone().unwrap();

        device.events_enable()?;

        Ok(EventStream { device, reader: BufReader::new(file) })
    }
}

impl<'a> Drop for EventStream<'a> {
    fn drop(&mut self) {
        let _ = self.device.events_disable();
    }
}

impl<'a> EventStream<'a> {
    fn read_next_blocking(&mut self) -> std::io::Result<Event> {
        let mut buf_hdr = [0; std::mem::size_of::<uapi::EventHeader>()];
        let mut buf_data = SmallVec::<[u8; 32]>::new();

        self.reader.read_exact(&mut buf_hdr)?;

        let hdr: uapi::EventHeader = unsafe { std::mem::transmute_copy(&buf_hdr) };

        buf_data.resize(hdr.length as usize, 0);
        self.reader.read_exact(&mut buf_data)?;

        Ok(self.translate(hdr.code, &buf_data))
    }

    fn translate(&self, code: u16, data: &[u8]) -> Event {
        match code {
            uapi::SDTX_EVENT_REQUEST => {
                if !data.is_empty() {
                    return Event::Unknown { code, data: data.into() };
                }

                Event::Request
            },

            uapi::SDTX_EVENT_CANCEL => {
                if data.len() != std::mem::size_of::<u16>() {
                    return Event::Unknown { code, data: data.into() };
                }

                let reason = &data[0..std::mem::size_of::<u16>()];
                let reason = u16::from_ne_bytes(reason.try_into().unwrap());
                let reason = event::CancelReason::from(reason);

                Event::Cancel { reason }
            },

            uapi::SDTX_EVENT_BASE_CONNECTION => {
                if data.len() != 2 * std::mem::size_of::<u16>() {
                    return Event::Unknown { code, data: data.into() };
                }

                let state = &data[0..std::mem::size_of::<u16>()];
                let state = u16::from_ne_bytes(state.try_into().unwrap());
                let state = event::BaseState::from(state);

                let base = &data[std::mem::size_of::<u16>()..2*std::mem::size_of::<u16>()];
                let base = u16::from_ne_bytes(base.try_into().unwrap());

                let device_type = DeviceType::from(base);
                let id = (base & 0xff) as u8;

                Event::BaseConnection { state, device_type, id }
            },

            uapi::SDTX_EVENT_LATCH_STATUS => {
                if data.len() != std::mem::size_of::<u16>() {
                    return Event::Unknown { code, data: data.into() };
                }

                let status = &data[0..std::mem::size_of::<u16>()];
                let status = u16::from_ne_bytes(status.try_into().unwrap());
                let status = event::LatchStatus::from(status);

                Event::LatchStatus { status }
            },

            uapi::SDTX_EVENT_DEVICE_MODE => {
                if data.len() != std::mem::size_of::<u16>() {
                    return Event::Unknown { code, data: data.into() };
                }

                let mode = &data[0..std::mem::size_of::<u16>()];
                let mode = u16::from_ne_bytes(mode.try_into().unwrap());
                let mode = event::DeviceMode::from(mode);

                Event::DeviceMode { mode }
            },

            code => {
                Event::Unknown { code, data: data.into() }
            },
        }
    }
}

impl<'a> Iterator for EventStream<'a> {
    type Item = std::io::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.read_next_blocking())
    }
}


#[allow(clippy::identity_op)]
mod uapi {
    use nix::{ioctl_none, ioctl_read};

    // status/error categories
    pub const SDTX_CATEGORY_STATUS: u16           = 0x0000;
    pub const SDTX_CATEGORY_RUNTIME_ERROR: u16    = 0x1000;
    pub const SDTX_CATEGORY_HARDWARE_ERROR: u16   = 0x2000;

    pub const SDTX_CATEGORY_MASK: u16             = 0xf000;

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
    #[repr(C, packed)]
    pub struct EventHeader {
        pub length: u16,
        pub code: u16,
    }

    #[derive(Debug, Clone, Copy)]
    #[repr(C, packed)]
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
