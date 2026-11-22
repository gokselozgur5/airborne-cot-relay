use airborne_cot_relay::{DualBufferSampler, PlatformPose};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn pose(x: f64) -> PlatformPose {
    PlatformPose::new(x as u64, x, x, x, x, x, x)
}

#[test]
fn concurrent_updates_never_expose_a_torn_snapshot() {
    let sampler = Arc::new(DualBufferSampler::new(pose(0.0), 0));
    let stop = Arc::new(AtomicBool::new(false));
    let writers: Vec<_> = (0..2u64)
        .map(|w| {
            let (s, flag) = (Arc::clone(&sampler), Arc::clone(&stop));
            std::thread::spawn(move || {
                let mut i = w;
                while !flag.load(Ordering::Relaxed) {
                    i += 2;
                    s.update(pose(i as f64), i);
                }
            })
        })
        .collect();
    let is_torn = |p: PlatformPose| p.lat_deg != p.yaw_rad || p.lat_deg != p.timestamp_ms as f64;
    let torn = (0..2_000_000u64)
        .filter(|_| is_torn(sampler.sample().pose))
        .count();
    stop.store(true, Ordering::Relaxed);
    writers.into_iter().for_each(|w| w.join().unwrap());
    assert_eq!(torn, 0, "snapshots mixing fields of two different updates");
}

#[test]
fn concurrent_multi_writer_reads_preserve_pose_and_sequence_invariants() {
    let p0 = PlatformPose::new(1000, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let sampler = Arc::new(DualBufferSampler::new(p0, 0));
    let stop = Arc::new(AtomicBool::new(false));
    let writers: Vec<_> = (0..4u64)
        .map(|w_id| {
            let s = Arc::clone(&sampler);
            std::thread::spawn(move || {
                for i in 1..=500 {
                    let ts_ns = w_id * 1_000_000 + i;
                    let p = PlatformPose::new(ts_ns, ts_ns as f64, 0.0, 0.0, 0.0, 0.0, 0.0);
                    s.update(p, ts_ns);
                }
            })
        })
        .collect();
    let readers: Vec<_> = (0..2)
        .map(|_| {
            let (s, flag) = (Arc::clone(&sampler), Arc::clone(&stop));
            std::thread::spawn(move || {
                let mut last_seq = 0;
                while !flag.load(Ordering::Relaxed) {
                    let snap = s.sample();
                    assert!(snap.is_valid);
                    assert!(snap.sequence >= last_seq);
                    last_seq = snap.sequence;
                    if snap.sequence > 0 {
                        assert_eq!(snap.pose.timestamp_ms, snap.timestamp_ns);
                        assert_eq!(snap.pose.lat_deg, snap.timestamp_ns as f64);
                    }
                }
            })
        })
        .collect();
    writers.into_iter().for_each(|w| w.join().unwrap());
    stop.store(true, Ordering::Relaxed);
    readers.into_iter().for_each(|r| r.join().unwrap());
}
