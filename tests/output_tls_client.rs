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
