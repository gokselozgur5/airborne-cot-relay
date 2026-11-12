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

#[test]
fn test_consecutive_failures_trigger_reconnect() {
    let mut sink = ScriptedSink(vec![io_err(), Ok(3), io_err(), io_err()].into());
    let mut health = SendHealth::new(2);
    assert!(send_tracked(&mut sink, &mut health, b"cot").is_err());
    assert!(!health.needs_reconnect());
    assert_eq!(send_tracked(&mut sink, &mut health, b"cot").unwrap(), 3);
    assert_eq!(health.failures(), (0, 1));
    assert!(send_tracked(&mut sink, &mut health, b"cot").is_err());
    assert!(send_tracked(&mut sink, &mut health, b"cot").is_err());
    assert!(health.needs_reconnect());
    assert_eq!(health.failures(), (2, 3));
}

#[test]
fn test_short_write_counts_as_failure() {
    let mut sink = ScriptedSink(vec![Ok(2)].into());
    let mut health = SendHealth::new(1);
    let err = send_tracked(&mut sink, &mut health, b"cot").unwrap_err();
    assert!(err.to_string().contains("short write: 2 of 3 bytes"));
    assert!(health.needs_reconnect());
}
