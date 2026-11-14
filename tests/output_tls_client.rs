use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::output::{TlsClientConfig, TlsClientSink, TlsConnector};
use airborne_cot_relay::relay::{Backoff, CotSink};
use std::io::Write;

struct Failing;
impl TlsConnector for Failing {
    fn connect(&mut self, _: &TlsClientConfig) -> Result<Box<dyn Write + Send>, RelayError> {
        Err(RelayError::IoError("refused".into()))
    }
}

struct Ok_;
impl TlsConnector for Ok_ {
    fn connect(&mut self, _: &TlsClientConfig) -> Result<Box<dyn Write + Send>, RelayError> {
        Ok(Box::new(Vec::new()))
    }
}

#[test]
fn config_rejects_half_identity() {
    let mut c = TlsClientConfig::new("tak");
    c.client_cert_path = Some("c.pem".into());
    assert!(c.validate().is_err());
    assert!(TlsClientConfig::new("").validate().is_err());
}

#[test]
fn backoff_grows_on_failure() {
    let mut s = TlsClientSink::new(TlsClientConfig::new("tak"), Failing, Backoff::new(100, 1000)).unwrap();
    assert!(s.send(b"x").is_err());
    assert_eq!(s.retry_delay_ms(), 100);
    assert!(s.send(b"x").is_err());
    assert_eq!(s.retry_delay_ms(), 200);
}

#[test]
fn success_connects_and_resets() {
    let mut s = TlsClientSink::new(TlsClientConfig::new("tak"), Ok_, Backoff::new(100, 1000)).unwrap();
    assert_eq!(s.send(b"abc").unwrap(), 3);
    assert!(s.is_connected());
    assert_eq!(s.retry_delay_ms(), 0);
}
