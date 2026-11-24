//! End-to-end loopback integration test for airborne-cot-relay live mode (#398).

use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use airborne_cot_relay::app::run_live::run_live;
use airborne_cot_relay::metrics::Metrics;
use airborne_cot_relay::runtime::limit::RunLimit;
use airborne_cot_relay::runtime::ShutdownSignal;
use airborne_cot_relay::RelayConfig;
use stm32_navcore::protocol::NavTelemetry;

fn frame(ts: u32, lat: f32, lon: f32) -> Vec<u8> {
    let t = NavTelemetry {
        timestamp_ms: ts,
        pos_ned_m: [lat, lon, -100.0],
        vel_ned_mps: [15.0, 0.0, 0.0],
        euler_rad: [0.0, 0.0, 0.0],
        status: 0,
    };
    let mut b = [0u8; 64];
    let n = t.pack_uart(&mut b).unwrap();
    b[..n].to_vec()
}

#[test]
fn test_live_mode_loopback_e2e() {
    let telem_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let telem_addr = telem_sock.local_addr().unwrap();
    drop(telem_sock);

    let det_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    let det_addr = det_sock.local_addr().unwrap();
    drop(det_sock);

    let cot_receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
    cot_receiver
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .unwrap();
    let cot_addr = cot_receiver.local_addr().unwrap();

    let cfg = RelayConfig {
        telemetry_listen_addr: telem_addr.to_string(),
        cot_broadcast_addr: cot_addr.to_string(),
        detections_listen_addr: Some(det_addr.to_string()),
        http_listen_addr: None,
        target_stale_timeout_ms: 1000,
        ..Default::default()
    };

    let metrics = Arc::new(Metrics::new());
    let shutdown = ShutdownSignal::new();
    let shutdown_worker = shutdown.clone();
    let metrics_worker = Arc::clone(&metrics);

    let handle = thread::spawn(move || {
        run_live(
            &cfg,
            &metrics_worker,
            shutdown_worker,
            RunLimit::unbounded(),
        )
    });

    thread::sleep(Duration::from_millis(50));

    let telem_sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    let det_sender = UdpSocket::bind("127.0.0.1:0").unwrap();

    let det_line = b"640.0,360.0,630.0,350.0,20.0,20.0,vehicle,0.95\n";

    // Send consecutive frames + detections so track reaches confirmed status (min_hits=3)
    for i in 0..10 {
        let ts = 1_000 + i * 100;
        telem_sender
            .send_to(&frame(ts, 0.0, 0.0), telem_addr)
            .unwrap();
        det_sender.send_to(det_line, det_addr).unwrap();
        thread::sleep(Duration::from_millis(30));
    }

    let mut buf = [0u8; 2048];
    let n = cot_receiver.recv(&mut buf).expect("must receive CoT event");
    let cot_xml = String::from_utf8_lossy(&buf[..n]);

    assert!(cot_xml.contains("<event version=\"2.0\""));
    assert!(cot_xml.contains("type=\"a-u-"));
    assert!(cot_xml.contains("<point"));

    shutdown.trigger();
    let res = handle.join().expect("thread join ok");
    assert!(res.is_ok());
    assert!(res.unwrap() >= 1);
}
