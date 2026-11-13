use airborne_cot_relay::clock::{Clock, ManualClock, SystemClock};
use std::sync::Arc;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn clocks_are_send_and_sync() {
    assert_send_sync::<ManualClock>();
    assert_send_sync::<SystemClock>();
}

#[test]
fn manual_clock_observes_advance_and_set() {
    let clock = ManualClock::new(1_000);
    clock.advance_ms(250);
    assert_eq!(clock.now_ms(), 1_250);
    clock.set_ms(400);
    assert_eq!(clock.now_ms(), 400);
}

#[test]
fn manual_clock_advance_is_seen_across_threads() {
    let clock = Arc::new(ManualClock::new(0));
    let other = Arc::clone(&clock);
    std::thread::spawn(move || other.advance_ms(5))
        .join()
        .unwrap();
    assert_eq!(clock.now_ms(), 5);
}

#[test]
fn system_clock_never_decreases() {
    let clock = SystemClock::default();
    let mut prev = clock.now_ms();
    for _ in 0..1000 {
        let now = clock.now_ms();
        assert!(now >= prev);
        prev = now;
    }
    assert!(prev > 1_700_000_000_000);
}
