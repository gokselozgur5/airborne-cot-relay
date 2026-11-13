//! Adding operation context to a `RelayError` without changing its variant.

use crate::config::RelayError;

/// Returns `err` with `ctx: ` prepended to its message, same variant.
#[allow(dead_code)]
fn prefix(err: RelayError, ctx: &str) -> RelayError {
    let add = |msg: String| format!("{ctx}: {msg}");
    match err {
        RelayError::InvalidConfig(m) => RelayError::InvalidConfig(add(m)),
        RelayError::DecodeError(m) => RelayError::DecodeError(add(m)),
        RelayError::TargetingError(m) => RelayError::TargetingError(add(m)),
        RelayError::IoError(m) => RelayError::IoError(add(m)),
    }
}
