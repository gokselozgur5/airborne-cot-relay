use crate::config::RelayError;

pub trait ResultExt<T> {
    fn context(self, prefix: &str) -> Result<T, RelayError>;
}
