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

