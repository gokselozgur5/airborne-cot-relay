use airborne_cot_relay::output::TcpClientSink;
use airborne_cot_relay::relay::CotSink;
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn send_to_stalled_peer_returns_within_bound() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut sink = TcpClientSink::with_defaults(listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let chunk = vec![b'x'; 64 * 1024];
        for _ in 0..1024 {
            if sink.send(&chunk).is_err() {
                break;
            }
        }
        let _ = tx.send(());
    });
    let (_peer, _) = listener.accept().unwrap();
    let done = rx.recv_timeout(Duration::from_secs(10));
    assert!(
        done.is_ok(),
        "send is still blocked on a peer that stopped reading"
    );
}
