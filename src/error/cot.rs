//! Conversion from `sightline_cot::cot::CotError` into [`RelayError`].
//!
//! Network failures become `IoError`; every parsing or validation failure
//! becomes `DecodeError`. The original `CotError` text is kept.

use crate::config::RelayError;
use sightline_cot::cot::CotError;

impl From<CotError> for RelayError {
    fn from(err: CotError) -> Self {
        match err {
            CotError::NetworkError(_) => RelayError::IoError(err.to_string()),
            _ => RelayError::DecodeError(err.to_string()),
        }
    }
}
