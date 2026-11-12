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
