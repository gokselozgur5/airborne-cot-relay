/// Error classification for relay errors.
#[derive(Debug)]
pub enum ErrorClass {
    Fatal,
    Retryable,
    Droppable,
}

impl ErrorClass {
    /// Checks if the error is fatal.
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal)
    }

    /// Checks if the error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Retryable)
    }
}

impl From<std::io::Error> for ErrorClass {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::ConnectionRefused
            || err.kind() == std::io::ErrorKind::TimedOut
        {
            Self::Retryable
        } else {
            Self::Fatal
        }
    }
}
