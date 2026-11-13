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

#[cfg(unix)]
mod imp {
    use super::{RelayError, ShutdownSignal};
    use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
    use std::sync::Arc;

    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    const SIG_ERR: usize = usize::MAX;

    extern "C" {
        fn signal(signum: i32, handler: extern "C" fn(i32)) -> usize;
    }

    /// Flag set by the handler; leaked on purpose so it outlives any handler run.
    static TARGET: AtomicPtr<AtomicBool> = AtomicPtr::new(std::ptr::null_mut());

    extern "C" fn on_signal(_signum: i32) {
        let target = TARGET.load(Ordering::SeqCst);
        if !target.is_null() {
            // SAFETY: TARGET only holds pointers leaked from Arc::into_raw.
            unsafe { (*target).store(true, Ordering::SeqCst) };
        }
    }
}
