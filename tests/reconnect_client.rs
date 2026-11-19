use airborne_cot_relay::ReconnectPolicy;
use std::time::Duration;

#[test]
fn backoff_stays_capped_after_many_failures() {
    let policy = ReconnectPolicy::default();
    for failures in [10usize, 67, 68, 500, 5000] {
        assert_eq!(policy.compute_backoff(failures), Duration::from_secs(10));
    }
}
