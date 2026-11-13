//! Tests for the injectable sleepers.

use airborne_cot_relay::relay::{Clock, ManualClock};
use airborne_cot_relay::runtime::sleep::{ManualSleeper, Sleeper, ThreadSleeper};
use std::time::{Duration, Instant};

#[test]
fn manual_sleeper_records_durations_in_order() {
    let mut sleeper = ManualSleeper::new();
    sleeper.sleep(Duration::from_millis(300));
    sleeper.sleep(Duration::from_millis(100));
    let expected = [Duration::from_millis(300), Duration::from_millis(100)];
    assert_eq!(sleeper.slept(), expected);
}

#[test]
fn manual_sleeper_advances_attached_clock() {
    let clock = ManualClock::new(1_000);
    let mut sleeper = ManualSleeper::with_clock(&clock);
    sleeper.sleep(Duration::from_millis(250));
    sleeper.sleep(Duration::from_secs(2));
    assert_eq!(clock.now_ms(), 3_250);
}
