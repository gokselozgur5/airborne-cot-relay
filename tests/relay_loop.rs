use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::relay::{CotSink, DropPolicy, ManualClock};
use airborne_cot_relay::relay_loop::ReconnectingRelayLoop;
use std::sync::{Arc, Mutex};
struct BusyOnce(bool, Arc<Mutex<Vec<u8>>>);
impl CotSink for BusyOnce {
    fn send(&mut self, buf: &[u8]) -> Result<usize, RelayError> {
        if std::mem::take(&mut self.0) {
            Err(std::io::Error::from(std::io::ErrorKind::WouldBlock).into())
        } else {
            self.1.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
    }
}
#[test]
fn would_block_retry_keeps_fifo_order() {
    let sent = Arc::new(Mutex::new(Vec::new()));
    let sink = BusyOnce(true, Arc::clone(&sent));
    let f = || Err::<BusyOnce, _>(RelayError::IoError("unused".into()));
    let (clk, m) = (ManualClock::new(0), Arc::new(Metrics::new()));
    let mut r = ReconnectingRelayLoop::new(sink, f, 8, DropPolicy::DropOldest, clk, m);
    for p in [b"A".to_vec(), b"B".to_vec(), b"C".to_vec()] {
        let _ = r.enqueue(p);
    }
    let _ = (r.poll(), r.poll());
    assert_eq!(String::from_utf8_lossy(&sent.lock().unwrap()), "ABC");
}
