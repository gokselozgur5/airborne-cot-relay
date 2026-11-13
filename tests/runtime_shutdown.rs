//! Tests for the shared shutdown flag.

use airborne_cot_relay::runtime::ShutdownSignal;
use std::thread;

#[test]
fn new_signal_is_not_triggered() {
    assert!(!ShutdownSignal::new().is_triggered());
    assert!(!ShutdownSignal::default().is_triggered());
}

#[test]
fn clone_triggered_on_another_thread_is_seen_by_original() {
    let signal = ShutdownSignal::new();
    let remote = signal.clone();
    thread::spawn(move || remote.trigger()).join().unwrap();
    assert!(signal.is_triggered());
}

#[test]
fn shared_flag_store_is_seen_by_signal() {
    use std::sync::atomic::Ordering;
    let signal = ShutdownSignal::new();
    signal.shared_flag().store(true, Ordering::SeqCst);
    assert!(signal.is_triggered());
}
