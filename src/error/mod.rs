//! Conversions and helpers for [`RelayError`].
//!
//! Each submodule adds one family of conversions or helpers so that call
//! sites can use `?` instead of mapping errors by hand.

use crate::config::RelayError;

impl From<std::io::Error> for RelayError {
    /// Maps any I/O failure to `IoError`, keeping the `ErrorKind` name so
    /// that `ConnectionRefused` and `AddrInUse` stay distinguishable.
    fn from(err: std::io::Error) -> Self {
        RelayError::IoError(format!("{:?}: {err}", err.kind()))
    }
}

impl From<&std::io::Error> for RelayError {
    /// Maps a borrowed I/O failure to `IoError`, keeping the `ErrorKind` name.
    fn from(err: &std::io::Error) -> Self {
        RelayError::IoError(format!("{:?}: {err}", err.kind()))
    }
}

impl From<std::io::ErrorKind> for RelayError {
    /// Maps an `ErrorKind` directly to `IoError`, keeping the `ErrorKind` name.
    fn from(kind: std::io::ErrorKind) -> Self {
        RelayError::from(std::io::Error::from(kind))
    }
}

pub mod class;
mod codec;
pub mod context;
mod cot;
pub mod exit;
