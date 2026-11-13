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

/// What the binary prints and returns when a run ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub stdout: String,
    pub stderr: Option<String>,
    pub status: u8,
}

/// Builds the outcome: the metrics snapshot always, the error when present.
pub fn finish<T>(result: &Result<T, RelayError>, snapshot: &Snapshot) -> RunOutcome {
    let stdout = snapshot.render();
    match result {
        Ok(_) => RunOutcome {
            stdout,
            stderr: None,
            status: 0,
        },
        Err(e) => RunOutcome {
            stdout,
            stderr: Some(format!("error: {e}")),
            status: status_for(e),
        },
    }
}
