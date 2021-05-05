pub mod pci;
pub mod perf;
pub mod profile;

use thiserror::Error;

use std::path::PathBuf;


#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not access device {device:?}")]
    DeviceAccess { source: std::io::Error, device: PathBuf },

    #[error("I/O error")]
    IoError { source: std::io::Error },

    #[error("DTX subsystem error")]
    DtxError { source: sdtx::ProtocolError },

    #[error("SysFS error")]
    SysFsError { source: pci::SysFsError },

    #[error("Invalid data")]
    InvalidData,
}

pub type Result<T> = std::result::Result<T, Error>;


impl From<sdtx::Error> for Error {
    fn from(err: sdtx::Error) -> Self {
        match err {
            sdtx::Error::IoError { source } => Error::IoError { source },
            sdtx::Error::ProtocolError { source } => Error::DtxError { source },
        }
    }
}
