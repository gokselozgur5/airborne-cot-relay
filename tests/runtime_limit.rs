//! Tests for bounding loops by iteration count or deadline.

use airborne_cot_relay::relay::ManualClock;
use airborne_cot_relay::runtime::limit::RunLimit;

fn count_runs(mut limit: RunLimit, clock: &ManualClock, step_ms: u64) -> u32 {
    let mut runs = 0;
    while runs < 1_000 && limit.should_continue(clock) {
        runs += 1;
        clock.advance_ms(step_ms);
    }
    runs
}

#[test]
fn iteration_limit_runs_exactly_five_times() {
    let clock = ManualClock::new(0);
    assert_eq!(count_runs(RunLimit::iterations(5), &clock, 0), 5);
    assert_eq!(count_runs(RunLimit::iterations(0), &clock, 0), 0);
}

#[test]
fn deadline_limit_stops_on_the_manual_clock() {
    let clock = ManualClock::new(1_000);
    assert_eq!(count_runs(RunLimit::deadline_ms(1_500), &clock, 100), 5);
    let after = RunLimit::deadline_after(&clock, 300);
    assert_eq!(after, RunLimit::deadline_ms(1_800));
    assert_eq!(count_runs(after, &clock, 100), 3);
}

#[test]
fn unbounded_limit_never_stops() {
    let clock = ManualClock::new(0);
    assert_eq!(count_runs(RunLimit::unbounded(), &clock, 1_000), 1_000);
}
