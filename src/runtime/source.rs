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

/// In-memory telemetry source that returns queued chunks in order.
#[derive(Debug, Clone, Default)]
pub struct VecTelemetrySource {
    chunks: VecDeque<Vec<u8>>,
}

impl VecTelemetrySource {
    /// Creates a source that yields `chunks` in order.
    pub fn new(chunks: Vec<Vec<u8>>) -> Self {
        let chunks = chunks.into();
        Self { chunks }
    }

    /// Queues one more chunk behind the existing ones.
    pub fn push(&mut self, chunk: Vec<u8>) {
        self.chunks.push_back(chunk);
    }
}

impl TelemetrySource for VecTelemetrySource {
    fn poll_chunk(&mut self) -> Result<Option<Vec<u8>>, RelayError> {
        Ok(self.chunks.pop_front())
    }
}
