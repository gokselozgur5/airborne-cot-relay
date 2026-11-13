use airborne_cot_relay::relay::{send_tracked, BoundedQueue, CotSink, DropPolicy, SendHealth};
use airborne_cot_relay::{
    CameraIntrinsics, CotStreamer, RelayError, TargetDetection, Targeter, TelemetryIngest,
    TrackSummary, TrackedObject,
};
use sightline_cot::cot::CotEvent;
use stm32_navcore::protocol::NavTelemetry;

fn frame(timestamp_ms: u32) -> Vec<u8> {
    let telem = NavTelemetry {
        timestamp_ms,
        pos_ned_m: [0.0, 0.0, -100.0],
        vel_ned_mps: [0.0; 3],
        euler_rad: [0.0; 3],
        status: 0,
    };
    let mut buf = [0u8; 64];
    let len = telem.pack_uart(&mut buf).unwrap();
    buf[..len].to_vec()
}

fn detection(u: f64, v: f64) -> TargetDetection {
    TargetDetection {
        pixel_u: u,
        pixel_v: v,
        bbox: (u - 10.0, v - 10.0, u + 10.0, v + 10.0),
        classification: "vehicle".into(),
        confidence: 0.9,
        embedding: None,
    }
}

#[derive(Default)]
struct Recorder(Vec<String>);

impl CotSink for Recorder {
    fn send(&mut self, payload: &[u8]) -> Result<usize, RelayError> {
        self.0.push(String::from_utf8(payload.to_vec()).unwrap());
        Ok(payload.len())
    }
}

fn summary(o: &TrackedObject) -> TrackSummary {
    TrackSummary {
        id: o.id,
        classification: o.classification.clone(),
        lat_deg: o.lat_deg,
        lon_deg: o.lon_deg,
        hae_m: o.hae_m,
        ce90_m: o.ce90_m,
        le90_m: 10.0,
        course_deg: o.course_deg,
        speed_mps: o.speed_mps,
    }
}

/// Feeds 20 frame and detection batches and queues every confirmed track.
fn run_batches(queue: &mut BoundedQueue<TrackSummary>) {
    let mut ingest = TelemetryIngest::new(35.0, 139.0, 0.0, 5_000);
    let mut targeter = Targeter::with_params(&CameraIntrinsics::default(), 0.0, 2, 5, 1.0);
    for i in 0..20u32 {
        let pose = *ingest
            .process_bytes(&frame(1_000 + i * 100))
            .last()
            .unwrap();
        let tracks = targeter.update(&pose, &[detection(640.0, 360.0)]).unwrap();
        tracks
            .iter()
            .filter(|o| o.is_confirmed)
            .for_each(|o| drop(queue.push(summary(o))));
    }
}

#[test]
fn frames_and_detections_reach_the_sink_as_parsable_cot() {
    let mut queue = BoundedQueue::new(64, DropPolicy::DropOldest);
    run_batches(&mut queue);
    let streamer = CotStreamer::new("127.0.0.1:0", "127.0.0.1:9", 5_000).unwrap();
    let (mut sink, mut health) = (Recorder::default(), SendHealth::new(3));
    while let Some(track) = queue.pop() {
        let xml = streamer.build_cot_xml(&track, 1_700_000_000_000);
        send_tracked(&mut sink, &mut health, xml.as_bytes()).unwrap();
    }
    let events: Vec<CotEvent> = sink
        .0
        .iter()
        .map(|x| CotEvent::from_xml(x).unwrap())
        .collect();
    assert!(!events.is_empty());
    assert!(events.iter().all(|e| e.uid.starts_with("TRACK-")));
}
