# Airborne CoT Relay

Airborne sensor-to-TAK Cursor-on-Target relay pipeline.

## Status

The library modules are implemented and covered by unit tests.

The `airborne-cot-relay` binary runs a simulated flight path in mock mode by default, transmitting Cursor-on-Target (CoT) events over UDP to the configured broadcast address (`127.0.0.1:6969` by default). Live mode binds the telemetry listener socket and awaits sensor input.

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
cargo run --bin airborne-cot-relay -- --dry-run
cargo run --bin airborne-cot-relay -- --mock
cargo run --example mock_flight_mission
cargo test --all-targets
```

### CLI Arguments

| Argument | Behavior |
| --- | --- |
| `--mock`, `-m` | Run a simulated flight path and send CoT events to destination (default) |
| `--live`, `-l` | Listen on telemetry address for live telemetry and camera streams |
| `--dry-run` | Resolve mission mode and configuration, then exit immediately |
| `--help`, `-h` | Print usage text and exit with status 0 |
| `--version`, `-V` | Print version and exit with status 0 |

### Exit Codes

- `0`: Success (mission completed, help or version displayed, dry-run resolved)
- `2`: Invalid CLI argument or configuration error
- `3`: Socket bind failure (e.g., telemetry address already in use)

See [docs/architecture.md](docs/architecture.md) for module responsibilities and [docs/operations.md](docs/operations.md) for configuration, environment variables, failure modes and exit codes.
