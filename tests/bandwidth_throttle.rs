use airborne_cot_relay::bandwidth_throttle::{BandwidthThrottle, ThrottleConfig, ThrottleDecision};
use airborne_cot_relay::priority_scheduler::CotPriority;
use std::time::Duration;

#[test]
fn payload_above_burst_is_not_throttled_forever() {
    let mut throttle = BandwidthThrottle::new(ThrottleConfig::default());
    let first = throttle.admit(CotPriority::High, 5000);
    let ThrottleDecision::Throttled { wait_ms } = first else {
        return;
    };
    std::thread::sleep(Duration::from_millis(wait_ms + 100));
    let second = throttle.admit(CotPriority::High, 5000);
    assert!(
        !matches!(second, ThrottleDecision::Throttled { .. }),
        "waited the advertised {wait_ms} ms, got {second:?} again"
    );
}
