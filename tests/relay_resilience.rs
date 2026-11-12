use airborne_cot_relay::relay::{send_tracked, CotSink, SendHealth};
use airborne_cot_relay::RelayError;
use std::collections::VecDeque;

/// Sink that replays a scripted sequence of send results.
struct ScriptedSink(VecDeque<Result<usize, RelayError>>);

impl CotSink for ScriptedSink {
    fn send(&mut self, _payload: &[u8]) -> Result<usize, RelayError> {
        self.0.pop_front().unwrap_or(Ok(0))
    }
}

fn io_err() -> Result<usize, RelayError> {
    Err(RelayError::IoError("link down".to_string()))
}
