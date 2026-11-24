# Operations

This guide covers configuration, environment overrides, validation, errors and exit status for the `airborne-cot-relay` crate.

## Configuration Defaults and Environment Overrides

`RelayConfig::default()` returns safe local defaults. Configuration can be overridden using environment variables at startup:

| Field | Default | Environment Variable |
| --- | --- | --- |
| `telemetry_listen_addr` | `0.0.0.0:14550` | `RELAY_TELEMETRY_ADDR` |
| `cot_broadcast_addr` | `127.0.0.1:6969` | `RELAY_COT_ADDR` |
| `ground_elevation_m` | `0.0` | — |
| `target_stale_timeout_ms` | `5000` | `RELAY_STALE_TIMEOUT_MS` |
| `detections_listen_addr` | `None` | `RELAY_DETECTIONS_ADDR` |
| `http_listen_addr` | `None` | `RELAY_HTTP_ADDR` |
| `camera.focal_length_px` | `1200.0` | — |
| `camera.principal_u` | `640.0` | — |
| `camera.principal_v` | `360.0` | — |
| `camera.image_width` | `1280` | — |
| `camera.image_height` | `720` | — |

> [!NOTE]
> The default CoT destination is `127.0.0.1:6969` to prevent unintentional live network multicast injection. Live multicast groups (such as `239.2.3.1:6969`) can be opted into via `RELAY_COT_ADDR`.
> In live mode, telemetry is ingested from `telemetry_listen_addr` and detections from `detections_listen_addr` (if configured). An optional lightweight HTTP server on `http_listen_addr` provides health status and `/shutdown` controls. Outbound CoT emissions are gated by `StaleGate` when platform pose telemetry goes stale.

## Validation

`RelayConfig::validate` enforces configuration invariants:

1. `telemetry_listen_addr` must be a valid, parsable socket address with a non-zero port.
2. `cot_broadcast_addr` must be a valid, parsable socket address with a non-zero port.
3. `camera.focal_length_px` must be finite and strictly positive.
4. `ground_elevation_m` must be finite.
5. `target_stale_timeout_ms` must be positive and not exceed `86_400_000` ms (24 hours).
6. Camera dimensions `image_width` and `image_height` must be non-zero, and the principal point `(principal_u, principal_v)` must lie inside the image dimensions.

## Error Variants

| Variant | Display prefix | Produced by |
| --- | --- | --- |
| `InvalidConfig` | `Invalid config:` | `RelayConfig::validate` and configuration booting |
| `IoError` | `IO error:` | `CotStreamer::new` (bind, non-blocking mode, address resolution) and UDP transmission |
| `TargetingError` | `Targeting error:` | `Targeter::project_pixel` and `Targeter::update` |
| `DecodeError` | `Decode error:` | Frame decoding and XML/JSON parser errors |

## Failure Modes and Exit Status

- `0`: Success (mission completed cleanly, help/version printed, or dry-run exit).
- `2`: CLI unrecognized argument or configuration boot/validation failure (`CONFIG_ERROR_EXIT`).
- `3`: Network socket creation or bind failure (e.g. telemetry listen port collision).
- `TelemetryIngest::process_bytes` drops corrupted frames that fail CRC/COBS decoding and increments internal error counters without aborting the process.
- `CotStreamer::send_xml` returns `IoError` on datagram transmission failures, tracked in metrics.
