use airborne_cot_relay::{ReconnectClient, ReconnectPolicy};
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn zero_message_limit_does_not_hang_enqueue() {
    let policy = ReconnectPolicy::new().with_queue_limits(0, 1024);
    let mut client = ReconnectClient::new("127.0.0.1:9".parse().unwrap(), policy);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = client.enqueue(vec![1, 2, 3]);
        let _ = tx.send(());
    });
    let done = rx.recv_timeout(Duration::from_secs(2));
    assert!(done.is_ok(), "enqueue did not return within 2 s");
}
