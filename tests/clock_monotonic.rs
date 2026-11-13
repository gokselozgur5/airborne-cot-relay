use airborne_cot_relay::clock::monotonic::MonotonicClock;
use airborne_cot_relay::clock::{Clock, ManualClock};

#[test]
fn backward_step_holds_last_value_and_counts() {
    let clock = MonotonicClock::new(ManualClock::new(60_000));
    assert_eq!(clock.now_ms(), 60_000);
    clock.inner().set_ms(50_000);
    assert_eq!(clock.now_ms(), 60_000);
    assert_eq!(clock.backward_steps(), 1);
}

#[test]
fn forward_progress_resumes_without_counting() {
    let clock = MonotonicClock::new(ManualClock::new(60_000));
    clock.now_ms();
    clock.inner().set_ms(50_000);
    clock.now_ms();
    clock.inner().set_ms(60_500);
    assert_eq!(clock.now_ms(), 60_500);
    clock.inner().advance_ms(0);
    assert_eq!(clock.now_ms(), 60_500);
    assert_eq!(clock.backward_steps(), 1);
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn monotonic_clock_is_send_and_sync() {
    assert_send_sync::<MonotonicClock<ManualClock>>();
}
