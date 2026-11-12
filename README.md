# Airborne CoT Relay

Airborne sensor-to-TAK Cursor-on-Target relay pipeline.

## Status

The library modules are implemented and covered by unit tests.

The `airborne-cot-relay` binary is still a placeholder. It prints fixed messages for each mode and does not open sockets or read telemetry yet.

## System Architecture

```mermaid
graph TD
    A[Telemetry Stream] --> B[TelemetryIngest]
    C[Camera Detections] --> D[Targeter Engine]
    B --> D
    D --> E[MultiTargetTracker]
    E --> F[CotStreamer]
    F --> G[ATAK / TAK Server]
```

## Mathematical Formulation

1. **Ray Back-Projection:**
   Inverts pinhole intrinsics to camera frame unit vector.

2. **Ground Intersection:**
   Transforms ray to ENU and computes ground plane intercept.

3. **Covariance Estimation:**
   Projects angular uncertainty into CE90/LE90 error ellipses.

## Usage Instructions

```bash
cargo run --bin airborne-cot-relay -- --help
cargo run --example mock_flight_mission
cargo test --all-targets
```
