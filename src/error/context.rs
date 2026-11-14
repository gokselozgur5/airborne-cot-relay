use std::fmt;

pub trait ResultExt<T> {
    fn context(self, msg: &str) -> Self;
}

impl<T> ResultExt<T> for Result<T, RelayError> {
    fn context(mut self, msg: &str) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(e) => Err(RelayError::IoError(format!("{}: {}", msg, e))),
        }
    }
}

#[derive(Debug)]
pub struct RelayError {
    kind: RelayErrorKind,
}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            RelayErrorKind::IoError(ref e) => write!(f, "IO error: {}", e),
            // Add other variants as needed
        }
    }
}

#[derive(Debug)]
pub enum RelayErrorKind {
    IoError(String),
    // Add other variants as needed
}
