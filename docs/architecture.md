# Architecture

This document describes how the modules of the `airborne-cot-relay` crate fit together.

## Core Modules and Pipeline Wiring

The binary and library use modular stages to ingest telemetry, project sensor detections, filter duplicate or jittery tracks, and stream Cursor-on-Target (CoT) XML:

| Subsystem / Module | Path | Responsibility |
| --- | --- | --- |
| `app::config_boot` | `src/app/config_boot.rs` | Boots configuration from defaults and environment overrides |
| `app::run_mock` | `src/app/run_mock.rs` | Executes mock mission connecting ingest, targeter, track filter, and CoT streamer |
| `app::mock_mission` | `src/app/mock_mission.rs` | Deterministic synthetic flight telemetry and target detection generator |
| `config` | `src/config.rs` | `RelayError`, `CameraIntrinsics`, and `RelayConfig` definitions and validation |
| `telemetry` | `src/telemetry.rs` | Decodes navigation telemetry frames into `PlatformPose` records |
| `targeter` | `src/targeter.rs` | Back-projects camera pixels to ground plane and tracks multi-target associations |
| `track_filter` | `src/track_filter.rs` | Deduplicates and rate-limits track updates based on position delta and time |
| `relay` | `src/relay.rs` | Converts track summaries to CoT XML datagrams and manages UDP streaming |
| `metrics` | `src/metrics.rs` | Tracks operational counters (packets in, tracks created, sent, dropped) |
| `main` | `src/main.rs` | Command-line runner supporting `--mock`, `--live`, `--dry-run`, `--help`, and `--version` |

## Data Flow

1. Raw telemetry bytes enter `TelemetryIngest::process_bytes`, which buffers the stream, checks frame CRCs, and returns the decoded `PlatformPose` values. `latest_pose` returns the newest pose and `is_stale` compares it with the configured maximum age.

2. A pose and a list of `TargetDetection` values go to `Targeter::update`. The targeter projects each pixel onto the ground plane at the configured ground elevation and updates a multi-target tracker. It returns one `TrackedObject` per live track.

3. `Targeter::project_pixel` performs the single-pixel projection without tracking. It returns a `GroundIntersection` with CE90 and LE90 estimates.

## CoT Output

1. `TrackSummary::from_track` converts a track into geodetic coordinates, speed, course, and an error radius. Its LE90 value is fixed at 10 m.

2. `classification_to_cot_type` maps a classification string to a standard CoT type code:
   - Aircraft / Drone / UAV: `a-u-A-M-F-Q` (unknown UAS)
   - Vehicle / Car / Truck: `a-u-G-E-V-C` (unknown ground vehicle)
   - Person / Pedestrian / Human: `a-u-G-U-C-I` (unknown ground civilian)
   - Unrecognized classifications default to `a-u-G` (unknown ground).

3. `CotStreamer::build_cot_xml` renders an `<event>` element with the UID `TRACK-<prefix>-<id>`. The stale time is the event time plus the configured stale timeout.

4. `CotStreamer::send_track` sends the XML as one UDP datagram to the target destination address.
