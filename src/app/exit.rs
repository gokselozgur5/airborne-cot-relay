//! Final report and process exit status for a relay run.
use crate::config::RelayError;
use crate::metrics::Snapshot;

/// Exit status for `err`: 2 config, 3 I/O, 4 decode, 5 targeting.
pub fn status_for(err: &RelayError) -> u8 {
    match err {
        RelayError::InvalidConfig(_) => 2,
        RelayError::IoError(_) => 3,
        RelayError::DecodeError(_) => 4,
        RelayError::TargetingError(_) => 5,
    }
}
