# Operations

This guide covers configuration, validation, errors and exit status for the `airborne-cot-relay` crate.

## Configuration defaults

`RelayConfig::default()` returns the values below. The binary does not read a configuration file or environment variables, so these values apply only when a caller builds the config in code.

| Field | Default |
| --- | --- |
| `telemetry_listen_addr` | `0.0.0.0:14550` |
| `cot_broadcast_addr` | `239.2.3.1:6969` |
| `ground_elevation_m` | `0.0` |
| `target_stale_timeout_ms` | `5000` |
| `camera.focal_length_px` | `1200.0` |
| `camera.principal_u` | `640.0` |
| `camera.principal_v` | `360.0` |
| `camera.image_width` | `1280` |
| `camera.image_height` | `720` |

## Validation

`RelayConfig::validate` checks three rules and returns `RelayError::InvalidConfig` for the first one that fails:

1. `telemetry_listen_addr` must not be empty.
2. `cot_broadcast_addr` must not be empty.
3. `camera.focal_length_px` must be greater than zero.

The check does not inspect the principal point, the image size, or the ground elevation. A focal length of `NaN` also passes, because `NaN <= 0.0` is false.

