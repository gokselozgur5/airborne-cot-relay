//! Tests for SIGINT and SIGTERM routing into ShutdownSignal.
#![cfg(unix)]

use airborne_cot_relay::runtime::{signal::install_handlers, ShutdownSignal};

extern "C" {
    fn raise(signum: i32) -> i32;
}

#[test]
fn sigterm_then_sigint_trigger_the_installed_signal() {
    let first = ShutdownSignal::new();
    install_handlers(&first).unwrap();
    assert!(!first.is_triggered());
    assert_eq!(unsafe { raise(15) }, 0);
    assert!(first.is_triggered());

    let second = ShutdownSignal::new();
    install_handlers(&second).unwrap();
    assert_eq!(unsafe { raise(2) }, 0);
    assert!(second.is_triggered());
}
