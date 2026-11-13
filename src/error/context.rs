//! Adding operation context to a `RelayError` without changing its variant.

use crate::config::RelayError;

/// Returns `err` with `ctx: ` prepended to its message, same variant.
fn prefix(err: RelayError, ctx: &str) -> RelayError {
    let add = |msg: String| format!("{ctx}: {msg}");
    match err {
        RelayError::InvalidConfig(m) => RelayError::InvalidConfig(add(m)),
        RelayError::DecodeError(m) => RelayError::DecodeError(add(m)),
        RelayError::TargetingError(m) => RelayError::TargetingError(add(m)),
        RelayError::IoError(m) => RelayError::IoError(add(m)),
    }
}

/// Extension for `Result<T, RelayError>` that names the failed operation.
pub trait ResultExt<T> {
    /// Prefixes the error message with `ctx`, keeping the variant.
    fn context(self, ctx: &str) -> Result<T, RelayError>;
}

impl<T> ResultExt<T> for Result<T, RelayError> {
    fn context(self, ctx: &str) -> Result<T, RelayError> {
        self.map_err(|err| prefix(err, ctx))
    }
}
