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
