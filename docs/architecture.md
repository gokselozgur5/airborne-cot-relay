# Architecture

This document describes how the modules of the `airborne-cot-relay` crate fit together.

## Module responsibilities

| Module | File | Responsibility |
| --- | --- | --- |
| `config` | `src/config.rs` | `RelayError`, `CameraIntrinsics` and `RelayConfig`, including defaults and validation |
| `telemetry` | `src/telemetry.rs` | Parses the navigation telemetry byte stream into `PlatformPose` values |
| `targeter` | `src/targeter.rs` | Back-projects camera pixels to the ground and tracks multiple targets |
| `relay` | `src/relay.rs` | Converts tracks into CoT XML and sends them over UDP |
| `main` | `src/main.rs` | Command line entry point that selects a run mode |

## Data flow

1. Raw telemetry bytes enter `TelemetryIngest::process_bytes`, which buffers the stream, checks frame CRCs, and returns the decoded `PlatformPose` values. `latest_pose` returns the newest pose and `is_stale` compares it with the configured maximum age.

2. A pose and a list of `TargetDetection` values go to `Targeter::update`. The targeter projects each pixel onto the ground plane at the configured ground elevation and updates a multi-target tracker. It returns one `TrackedObject` per live track.

3. `Targeter::project_pixel` performs the single-pixel projection without tracking. It returns a `GroundIntersection` with CE90 and LE90 estimates.

