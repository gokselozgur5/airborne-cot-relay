//! SIGINT and SIGTERM handling that sets a `ShutdownSignal`.

use super::ShutdownSignal;
use crate::config::RelayError;

/// Routes SIGINT and SIGTERM to `shutdown`; a no-op on non-unix targets.
///
/// A later call redirects both signals to the newer `ShutdownSignal`.
pub fn install_handlers(shutdown: &ShutdownSignal) -> Result<(), RelayError> {
    imp::install(shutdown)
}

#[cfg(not(unix))]
mod imp {
    use super::{RelayError, ShutdownSignal};

    pub fn install(_shutdown: &ShutdownSignal) -> Result<(), RelayError> {
        Ok(())
    }
}
