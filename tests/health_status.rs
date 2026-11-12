use airborne_cot_relay::health::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone)]
struct Clock(Rc<Cell<u64>>);
impl TimeSource for Clock {
    fn now(&self) -> Duration {
        Duration::from_secs(self.0.get())
    }
}

fn mon() -> (HealthMonitor<Clock>, Rc<Cell<u64>>) {
    let t = Rc::new(Cell::new(100));
    let m = HealthMonitor::new(Clock(t.clone()), HealthThresholds::default());
    (m, t)
}

#[test]
fn starting_until_telemetry() {
    assert_eq!(mon().0.status(), HealthStatus::Starting);
}

#[test]
fn degrades_then_offline_by_telemetry_age() {
    let (mut m, t) = mon();
    m.record_telemetry();
    assert_eq!(m.status(), HealthStatus::Healthy);
    t.set(103);
    assert_eq!(m.status(), HealthStatus::Degraded);
    t.set(110);
    assert_eq!(m.status(), HealthStatus::Offline);
}
