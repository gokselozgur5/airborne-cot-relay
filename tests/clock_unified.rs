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
