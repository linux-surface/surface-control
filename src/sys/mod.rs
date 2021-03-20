pub mod dtx;
pub mod perf;

use thiserror::Error;

use std::path::PathBuf;


#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not access device {device:?}")]
    DeviceAccess { source: std::io::Error, device: PathBuf },

    #[error("I/O error")]
    IoError { source: std::io::Error },

    #[error("I/O error")]
    IoctlError { source: nix::Error },

    #[error("DTX subsystem error")]
    DtxError { source: dtx::DtxError },

    #[error("Invalid data")]
    InvalidData,
}

pub type Result<T> = std::result::Result<T, Error>;
