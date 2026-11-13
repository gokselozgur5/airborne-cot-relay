//! Conversions from binary codec errors into [`RelayError::DecodeError`].
//!
//! The codec error types have no `Display`, so the message keeps their
//! `Debug` text, prefixed with the codec that produced it.

use crate::config::RelayError;
use sightline_cot::proto::ProtoError;

impl From<ProtoError> for RelayError {
    fn from(err: ProtoError) -> Self {
        RelayError::DecodeError(format!("TAK protobuf: {err:?}"))
    }
}
