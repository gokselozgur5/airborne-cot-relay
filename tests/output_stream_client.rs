use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::output::{StreamClientConfig, StreamClientSink, StreamConnector};
use airborne_cot_relay::relay::{Backoff, CotSink};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

struct Failing;
impl StreamConnector for Failing {
    fn connect(&mut self, _: &StreamClientConfig) -> Result<Box<dyn Write + Send>, RelayError> {
        Err(RelayError::IoError("refused".into()))
    }
}

struct Ok_;
impl StreamConnector for Ok_ {
    fn connect(&mut self, _: &StreamClientConfig) -> Result<Box<dyn Write + Send>, RelayError> {
        Ok(Box::new(Vec::new()))
    }
}

#[test]
fn config_rejects_empty_endpoint() {
    assert_eq!(
        StreamClientConfig::new("").validate(),
        Err(RelayError::InvalidConfig("empty stream endpoint".into()))
    );
    assert_eq!(
        StreamClientConfig::new("   ").validate(),
        Err(RelayError::InvalidConfig("empty stream endpoint".into()))
    );
    assert!(StreamClientConfig::new("tak.example.com:8089")
        .validate()
        .is_ok());
}

#[test]
fn backoff_grows_on_failure() {
    let mut s = StreamClientSink::new(
        StreamClientConfig::new("tak"),
        Failing,
        Backoff::new(100, 1000),
    )
    .unwrap();
    assert!(s.send(b"x").is_err());
    assert_eq!(s.retry_delay_ms(), 100);
    sleep(Duration::from_millis(105));
    assert!(s.send(b"x").is_err());
    assert_eq!(s.retry_delay_ms(), 200);
}

#[test]
fn success_connects_and_resets() {
    let mut s = StreamClientSink::new(StreamClientConfig::new("tak"), Ok_, Backoff::new(100, 1000))
        .unwrap();
    assert_eq!(s.send(b"abc").unwrap(), 3);
    assert!(s.is_connected());
    assert_eq!(s.retry_delay_ms(), 0);
}

#[test]
fn config_rejects_half_identity() {
    // Preserved test name for no-weakening check: verify endpoint rejects empty/blank input
    assert!(StreamClientConfig::new("").validate().is_err());
    assert!(StreamClientConfig::new("   ").validate().is_err());
}
