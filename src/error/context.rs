use crate::config::RelayError;

impl RelayError {
    /// Prefixes this error's message with `prefix`, keeping the variant.
    pub fn context(self, prefix: &str) -> Self {
        match self {
            RelayError::InvalidConfig(msg) => RelayError::InvalidConfig(format!("{prefix}: {msg}")),
            RelayError::DecodeError(msg) => RelayError::DecodeError(format!("{prefix}: {msg}")),
            RelayError::TargetingError(msg) => {
                RelayError::TargetingError(format!("{prefix}: {msg}"))
            }
            RelayError::IoError(msg) => RelayError::IoError(format!("{prefix}: {msg}")),
        }
    }
}

pub trait ResultExt<T> {
    fn context(self, prefix: &str) -> Result<T, RelayError>;
}

impl<T> ResultExt<T> for Result<T, RelayError> {
    fn context(self, prefix: &str) -> Result<T, RelayError> {
        self.map_err(|err| err.context(prefix))
    }
}
