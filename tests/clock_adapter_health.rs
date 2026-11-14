use airborne_cot_relay::clock::adapters::HealthTimeAdapter;
use airborne_cot_relay::clock::{Clock, ManualClock};
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

#[test]
fn health_time_adapter_new_and_inner_accessors() {
    let clock = ManualClock::new(1_234);
    let adapter = HealthTimeAdapter::new(clock);
    assert_eq!(adapter.inner().now_ms(), 1_234);
    let inner = adapter.into_inner();
    assert_eq!(inner.now_ms(), 1_234);
}

#[test]
fn health_time_adapter_implements_clock() {
    let clock = ManualClock::new(9_876);
    let adapter = HealthTimeAdapter::new(clock);
    assert_eq!(adapter.now_ms(), 9_876);
}
