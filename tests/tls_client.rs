use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::output::{TlsClientConfig, TlsClientSink, TlsConnector};
use airborne_cot_relay::relay::{Backoff, CotSink};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

struct DroppingWrite;
impl Write for DroppingWrite {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "write failed",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct ConnectOkWriteFails;
impl TlsConnector for ConnectOkWriteFails {
    fn connect(&mut self, _: &TlsClientConfig) -> Result<Box<dyn Write + Send>, RelayError> {
        Ok(Box::new(DroppingWrite))
    }
}

#[test]
fn backoff_grows_when_connect_succeeds_but_write_fails() {
    let mut s = TlsClientSink::new(
        TlsClientConfig::new("tak"),
        ConnectOkWriteFails,
        Backoff::new(100, 1000),
    )
    .unwrap();
    assert!(s.send(b"x").is_err());
    assert_eq!(s.retry_delay_ms(), 100);
    sleep(Duration::from_millis(105));
    assert!(s.send(b"x").is_err());
    assert_eq!(s.retry_delay_ms(), 200);
}
