use airborne_cot_relay::config::RelayError;
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::relay::{CotSink, DropPolicy, ManualClock};
use airborne_cot_relay::relay_loop::ReconnectingRelayLoop;
use std::sync::Arc;

struct MockSink;
impl CotSink for MockSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        Ok(payload.len())
    }
}

#[test]
fn test_reconnecting_relay_loop_type_exists() {
    let clock = ManualClock::new(0);
    let metrics = Arc::new(Metrics::new());
    let _ = ReconnectingRelayLoop::new(
        MockSink,
        || Ok(MockSink),
        10,
        DropPolicy::DropOldest,
        clock,
        metrics,
    );
}

#[test]
fn test_reconnecting_relay_loop_enqueue_and_drain() {
    let clock = ManualClock::new(100);
    let metrics = Arc::new(Metrics::new());
    let mut loop_inst = ReconnectingRelayLoop::new(
        MockSink,
        || Ok(MockSink),
        5,
        DropPolicy::DropOldest,
        clock,
        Arc::clone(&metrics),
    );
    loop_inst.enqueue(b"hello".to_vec());
    loop_inst.enqueue(b"world".to_vec());
    assert_eq!(metrics.snapshot(100).queue_depth, 2);
    loop_inst.poll().unwrap();
    let snap = metrics.snapshot(100);
    assert_eq!(snap.queue_depth, 0);
    assert_eq!(snap.sent, 2);
}

#[test]
fn test_reconnecting_relay_loop_capacity_and_drop_policy() {
    let clock = ManualClock::new(100);
    let metrics = Arc::new(Metrics::new());
    let mut loop_inst = ReconnectingRelayLoop::new(
        MockSink,
        || Ok(MockSink),
        2,
        DropPolicy::DropOldest,
        clock,
        Arc::clone(&metrics),
    );
    loop_inst.enqueue(b"a".to_vec());
    loop_inst.enqueue(b"b".to_vec());
    loop_inst.enqueue(b"c".to_vec());
    let snap = metrics.snapshot(100);
    assert_eq!(snap.queue_depth, 2);
    assert_eq!(snap.dropped, 1);
}

struct TrackingSink {
    fails: bool,
    partial: bool,
}

impl CotSink for TrackingSink {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        if self.fails {
            return Err(RelayError::IoError("send failed".into()));
        }
        if self.partial {
            return Ok(payload.len().saturating_sub(1));
        }
        Ok(payload.len())
    }
}

#[test]
fn test_failed_send_drops_payload_counted() {
    let clock = ManualClock::new(100);
    let metrics = Arc::new(Metrics::new());
    let make_sink = || {
        Ok(TrackingSink {
            fails: true,
            partial: false,
        })
    };
    let mut loop_inst = ReconnectingRelayLoop::new(
        TrackingSink {
            fails: true,
            partial: false,
        },
        make_sink,
        5,
        DropPolicy::DropOldest,
        clock,
        Arc::clone(&metrics),
    );
    loop_inst.enqueue(b"test".to_vec());
    let _ = loop_inst.poll();
    let snap = metrics.snapshot(100);
    assert_eq!(snap.dropped, 1);
    assert_eq!(snap.queue_depth, 0);
}

#[test]
fn test_short_write_fails_health() {
    let clock = ManualClock::new(100);
    let metrics = Arc::new(Metrics::new());
    let make_sink = || {
        Ok(TrackingSink {
            fails: false,
            partial: true,
        })
    };
    let mut loop_inst = ReconnectingRelayLoop::new(
        TrackingSink {
            fails: false,
            partial: true,
        },
        make_sink,
        5,
        DropPolicy::DropOldest,
        clock,
        Arc::clone(&metrics),
    );
    loop_inst.enqueue(b"abc".to_vec());
    let _ = loop_inst.poll();
    let snap = metrics.snapshot(100);
    assert_eq!(snap.sent, 0);
    assert_eq!(snap.dropped, 1);
}
