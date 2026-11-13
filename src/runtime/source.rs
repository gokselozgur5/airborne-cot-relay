//! Polled input sources for telemetry bytes and detection batches.

use crate::config::RelayError;
use crate::targeter::TargetDetection;
use std::collections::VecDeque;

/// Non-blocking source of raw telemetry bytes.
pub trait TelemetrySource {
    /// Returns the next chunk, or `Ok(None)` when nothing is ready.
    fn poll_chunk(&mut self) -> Result<Option<Vec<u8>>, RelayError>;
}

/// Non-blocking source of detection batches, one batch per video frame.
pub trait DetectionSource {
    /// Returns the next batch, or `Ok(None)` when nothing is ready.
    fn poll_batch(&mut self) -> Result<Option<Vec<TargetDetection>>, RelayError>;
}
