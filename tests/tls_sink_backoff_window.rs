use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::output::{TlsClientConfig, TlsClientSink, TlsConnector};
use airborne_cot_relay::relay::{Backoff, CotSink};
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct Refusing(Arc<AtomicU32>);
impl TlsConnector for Refusing {
    fn connect(&mut self, _: &TlsClientConfig) -> Result<Box<dyn Write + Send>, RelayError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(RelayError::IoError("ConnectionRefused".into()))
    }
}

#[test]
fn sends_inside_backoff_window_do_not_dial_again() {
    let dials = Arc::new(AtomicU32::new(0));
    let connector = Refusing(Arc::clone(&dials));
    let config = TlsClientConfig::new("tak.example");
    let mut sink = TlsClientSink::new(config, connector, Backoff::new(60_000, 60_000)).unwrap();
    for _ in 0..100 {
        assert!(sink.send(b"<event/>").is_err());
    }
    assert_eq!(sink.retry_delay_ms(), 60_000);
    assert_eq!(
        dials.load(Ordering::SeqCst),
        1,
        "connect attempts inside one 60 s backoff window"
    );
}
