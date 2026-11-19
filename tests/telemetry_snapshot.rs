use airborne_cot_relay::{DualBufferSampler, PlatformPose};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

fn pose(x: f64) -> PlatformPose {
    PlatformPose::new(x as u64, x, x, x, x, x, x)
}

#[test]
fn concurrent_updates_never_expose_a_torn_snapshot() {
    let sampler = Arc::new(DualBufferSampler::new(pose(0.0), 0));
    let stop = Arc::new(AtomicBool::new(false));
    let writers: Vec<_> = (0..2u64).map(|w| {
        let (s, flag) = (Arc::clone(&sampler), Arc::clone(&stop));
        std::thread::spawn(move || {
            let mut i = w;
            while !flag.load(Ordering::Relaxed) {
                i += 2;
                s.update(pose(i as f64), i);
            }
        })
    }).collect();
    let is_torn = |p: PlatformPose| p.lat_deg != p.yaw_rad || p.lat_deg != p.timestamp_ms as f64;
    let torn = (0..2_000_000u64).filter(|_| is_torn(sampler.sample().pose)).count();
    stop.store(true, Ordering::Relaxed);
    writers.into_iter().for_each(|w| w.join().unwrap());
    assert_eq!(torn, 0, "snapshots mixing fields of two different updates");
}
