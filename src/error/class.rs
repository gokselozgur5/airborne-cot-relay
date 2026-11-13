//! Classification of `RelayError` by how a loop should react to it.

/// How a relay loop should react to an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    /// The process cannot continue; stop and report.
    Fatal,
    /// A transient failure; retry with backoff.
    Retryable,
    /// A bad input item; drop it and keep running.
    Droppable,
}

/// Returns how a loop should react to `err`.
pub fn classify(err: &crate::config::RelayError) -> ErrorClass {
    use crate::config::RelayError;
    match err {
        RelayError::InvalidConfig(_) => ErrorClass::Fatal,
        RelayError::IoError(_) => ErrorClass::Retryable,
        RelayError::DecodeError(_) | RelayError::TargetingError(_) => ErrorClass::Droppable,
    }
}

impl ErrorClass {
    /// True when the process must stop.
    pub fn is_fatal(self) -> bool {
        self == ErrorClass::Fatal
    }

    /// True when the operation should be retried with backoff.
    pub fn is_retryable(self) -> bool {
        self == ErrorClass::Retryable
    }
}
