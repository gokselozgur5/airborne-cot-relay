//! Non-blocking UDP socket source for target detections.

use crate::config::RelayError;

fn parse_f64(s: &str, field: &str) -> Result<f64, RelayError> {
    s.parse()
        .map_err(|e| RelayError::DecodeError(format!("{field}: {e}")))
}

fn parse_f32(s: &str, field: &str) -> Result<f32, RelayError> {
    s.parse()
        .map_err(|e| RelayError::DecodeError(format!("{field}: {e}")))
}

pub struct UdpDetectionSource;
