//! Tests for the in-memory telemetry and detection sources.

use airborne_cot_relay::runtime::source::{
    DetectionSource, TelemetrySource, VecDetectionSource, VecTelemetrySource,
};
use airborne_cot_relay::TargetDetection;

#[test]
fn telemetry_source_returns_chunks_then_none() {
    let mut source = VecTelemetrySource::new(vec![vec![1, 2], vec![3]]);
    assert_eq!(source.poll_chunk(), Ok(Some(vec![1, 2])));
    assert_eq!(source.poll_chunk(), Ok(Some(vec![3])));
    assert_eq!(source.poll_chunk(), Ok(None));
    assert_eq!(source.poll_chunk(), Ok(None));
    source.push(vec![9]);
    assert_eq!(source.poll_chunk(), Ok(Some(vec![9])));
}

fn detection(classification: &str) -> TargetDetection {
    TargetDetection {
        pixel_u: 640.0,
        pixel_v: 360.0,
        bbox: (600.0, 320.0, 80.0, 80.0),
        classification: classification.into(),
        confidence: 0.9,
        embedding: None,
    }
}

#[test]
fn detection_source_returns_batches_then_none() {
    let first = vec![detection("car"), detection("person")];
    let mut source = VecDetectionSource::new(vec![first.clone(), Vec::new()]);
    assert_eq!(source.poll_batch(), Ok(Some(first)));
    assert_eq!(source.poll_batch(), Ok(Some(Vec::new())));
    assert_eq!(source.poll_batch(), Ok(None));
    source.push(vec![detection("truck")]);
    assert_eq!(source.poll_batch(), Ok(Some(vec![detection("truck")])));
}
