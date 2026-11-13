use airborne_cot_relay::clock::adapters::HealthTimeAdapter;
use airborne_cot_relay::clock::ManualClock;
use airborne_cot_relay::health::{HealthMonitor, HealthStatus, HealthThresholds};
use std::sync::Arc;

#[test]
fn monitor_degrades_when_adapted_clock_advances() {
    let clock = ManualClock::new(5_000);
    let mut monitor = HealthMonitor::new(HealthTimeAdapter(&clock), HealthThresholds::default());
    monitor.record_telemetry();
    assert_eq!(monitor.status(), HealthStatus::Healthy);
    clock.advance_ms(1_999);
    assert_eq!(monitor.status(), HealthStatus::Healthy);
    clock.advance_ms(1);
    assert_eq!(monitor.status(), HealthStatus::Degraded);
}

#[test]
fn monitor_goes_offline_with_shared_clock() {
    let clock = Arc::new(ManualClock::new(0));
    let adapter = HealthTimeAdapter(Arc::clone(&clock));
    let mut monitor = HealthMonitor::new(adapter, HealthThresholds::default());
    monitor.record_telemetry();
    clock.advance_ms(10_000);
    assert_eq!(monitor.status(), HealthStatus::Offline);
}
