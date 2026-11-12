use crate::config::RelayError;
use stm32_navcore::protocol::NavTelemetry;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformPose {
    pub timestamp_ms: u64,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub hae_m: f64,
    pub roll_rad: f64,
    pub pitch_rad: f64,
    pub yaw_rad: f64,
}

impl PlatformPose {
    pub const fn new(
        timestamp_ms: u64,
        lat_deg: f64,
        lon_deg: f64,
        hae_m: f64,
        roll_rad: f64,
        pitch_rad: f64,
        yaw_rad: f64,
    ) -> Self {
        Self {
            timestamp_ms,
            lat_deg,
            lon_deg,
            hae_m,
            roll_rad,
            pitch_rad,
            yaw_rad,
        }
    }
}

impl PlatformPose {
    pub fn is_stale(&self, now_ms: u64, max_age_ms: u64) -> bool {
        now_ms.saturating_sub(self.timestamp_ms) > max_age_ms
    }
}

fn lerp(a: f64, b: f64, f: f64) -> f64 {
    a + (b - a) * f
}

/// Interpolates between two samples, clamping outside their time range.
pub fn interpolate_pose(a: &PlatformPose, b: &PlatformPose, t_ms: u64) -> PlatformPose {
    if t_ms <= a.timestamp_ms || b.timestamp_ms <= a.timestamp_ms {
        return *a;
    }
    if t_ms >= b.timestamp_ms {
        return *b;
    }
    let f = (t_ms - a.timestamp_ms) as f64 / (b.timestamp_ms - a.timestamp_ms) as f64;
    PlatformPose::new(
        t_ms,
        lerp(a.lat_deg, b.lat_deg, f),
        lerp(a.lon_deg, b.lon_deg, f),
        lerp(a.hae_m, b.hae_m, f),
        lerp(a.roll_rad, b.roll_rad, f),
        lerp(a.pitch_rad, b.pitch_rad, f),
        a.yaw_rad + shortest_arc(a.yaw_rad, b.yaw_rad) * f,
    )
}

fn shortest_arc(from: f64, to: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    let d = (to - from).rem_euclid(tau);
    if d > std::f64::consts::PI {
        d - tau
    } else {
        d
    }
}

pub struct TelemetryIngest {
    rx_buf: Vec<u8>,
    origin_lat_deg: f64,
    origin_lon_deg: f64,
    origin_alt_m: f64,
    max_age_ms: u64,
    reset_threshold_ms: u64,
    latest_pose: Option<PlatformPose>,
    decode_errors: u64,
    last_decode_error: Option<RelayError>,
}

impl TelemetryIngest {
    pub fn new(
        origin_lat_deg: f64,
        origin_lon_deg: f64,
        origin_alt_m: f64,
        max_age_ms: u64,
    ) -> Self {
        Self {
            rx_buf: Vec::with_capacity(256),
            origin_lat_deg,
            origin_lon_deg,
            origin_alt_m,
            max_age_ms,
            reset_threshold_ms: Self::DEFAULT_RESET_THRESHOLD_MS,
            latest_pose: None,
            decode_errors: 0,
            last_decode_error: None,
        }
    }
}

impl TelemetryIngest {
    /// Backward jumps larger than this are treated as a clock reset.
    pub const DEFAULT_RESET_THRESHOLD_MS: u64 = 5000;

    pub fn with_reset_threshold_ms(mut self, threshold_ms: u64) -> Self {
        self.reset_threshold_ms = threshold_ms;
        self
    }

    pub fn latest_pose(&self) -> Option<PlatformPose> {
        self.latest_pose
    }

    pub fn is_stale(&self, now_ms: u64) -> bool {
        match self.latest_pose {
            Some(pose) => pose.is_stale(now_ms, self.max_age_ms),
            None => true,
        }
    }
}

impl TelemetryIngest {
    pub fn ned_to_geodetic(&self, pos_ned: [f32; 3]) -> (f64, f64, f64) {
        const METERS_PER_DEG_LAT: f64 = 111_132.92;
        let lat = self.origin_lat_deg + (pos_ned[0] as f64) / METERS_PER_DEG_LAT;
        let lat_rad = self.origin_lat_deg.to_radians();
        let meters_per_deg_lon = METERS_PER_DEG_LAT * lat_rad.cos();
        let lon = if meters_per_deg_lon.abs() > 1e-6 {
            self.origin_lon_deg + (pos_ned[1] as f64) / meters_per_deg_lon
        } else {
            self.origin_lon_deg
        };
        let hae = self.origin_alt_m - (pos_ned[2] as f64);
        (lat, lon, hae)
    }
}

impl TelemetryIngest {
    pub fn update_from_nav(&mut self, telem: &NavTelemetry) -> PlatformPose {
        if let Some(current) = self.latest_pose {
            let ts = telem.timestamp_ms as u64;
            let is_reset = current.timestamp_ms.saturating_sub(ts) > self.reset_threshold_ms;
            if ts <= current.timestamp_ms && !is_reset {
                return current;
            }
        }
        let (lat, lon, hae) = self.ned_to_geodetic(telem.pos_ned_m);
        let pose = PlatformPose::new(
            telem.timestamp_ms as u64,
            lat,
            lon,
            hae,
            telem.euler_rad[0] as f64,
            telem.euler_rad[1] as f64,
            telem.euler_rad[2] as f64,
        );
        self.latest_pose = Some(pose);
        pose
    }
}

impl TelemetryIngest {
    pub fn process_bytes(&mut self, incoming: &[u8]) -> Vec<PlatformPose> {
        self.rx_buf.extend_from_slice(incoming);
        let mut poses = Vec::new();
        while let Some(pos) = self.rx_buf.iter().position(|&b| b == 0x00) {
            let frame: Vec<u8> = self.rx_buf.drain(..=pos).collect();
            if frame.len() > 1 {
                match NavTelemetry::unpack_uart(&frame) {
                    Ok(telem) => poses.push(self.update_from_nav(&telem)),
                    Err(e) => self.record_decode_error(format!("{e:?}")),
                }
            }
        }
        poses
    }
}

impl TelemetryIngest {
    pub fn decode_errors(&self) -> u64 {
        self.decode_errors
    }

    pub fn last_decode_error(&self) -> Option<&RelayError> {
        self.last_decode_error.as_ref()
    }

    fn record_decode_error(&mut self, msg: String) {
        self.decode_errors = self.decode_errors.saturating_add(1);
        self.last_decode_error = Some(RelayError::DecodeError(msg));
    }
}

impl TelemetryIngest {
    pub fn clear_buffer(&mut self) {
        self.rx_buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_pose_staleness() {
        let pose = PlatformPose::new(1000, 37.77, -122.41, 500.0, 0.0, 0.0, 1.57);
        assert!(!pose.is_stale(1500, 1000));
        assert!(pose.is_stale(2500, 1000));
        assert_eq!(pose.timestamp_ms, 1000);
    }
}

#[cfg(test)]
mod test_geodetic {
    use super::*;

    #[test]
    fn test_ned_to_geodetic_conversion() {
        let ingest = TelemetryIngest::new(0.0, 0.0, 100.0, 5000);
        let (lat, lon, hae) = ingest.ned_to_geodetic([111_132.92, 0.0, -50.0]);
        assert!((lat - 1.0).abs() < 1e-4);
        assert!((lon - 0.0).abs() < 1e-4);
        assert!((hae - 150.0).abs() < 1e-4);
    }
}

#[cfg(test)]
mod test_stream {
    use super::*;

    #[test]
    fn test_process_bytes_valid_frame() {
        let mut ingest = TelemetryIngest::new(35.0, 139.0, 200.0, 5000);
        let telem = NavTelemetry {
            timestamp_ms: 5000,
            pos_ned_m: [100.0, -50.0, -30.0],
            vel_ned_mps: [15.0, 0.0, 0.0],
            euler_rad: [0.05, -0.02, 0.8],
            status: 1,
        };
        let mut buf = [0u8; 64];
        let len = telem.pack_uart(&mut buf).unwrap();
        let poses = ingest.process_bytes(&buf[..len]);
        assert_eq!(poses.len(), 1);
        assert_eq!(poses[0].timestamp_ms, 5000);
        assert!(ingest.latest_pose().is_some());
        assert!(!ingest.is_stale(6000));
    }
}

#[cfg(test)]
mod test_crc {
    use super::*;

    #[test]
    fn test_crc_error_rejection() {
        let mut ingest = TelemetryIngest::new(35.0, 139.0, 200.0, 1000);
        let telem = NavTelemetry {
            timestamp_ms: 1000,
            pos_ned_m: [0.0, 0.0, 0.0],
            vel_ned_mps: [0.0, 0.0, 0.0],
            euler_rad: [0.0, 0.0, 0.0],
            status: 0,
        };
        let mut buf = [0u8; 64];
        let len = telem.pack_uart(&mut buf).unwrap();
        let mut bad = buf[..len].to_vec();
        bad[5] ^= 0xFF;
        assert!(ingest.process_bytes(&bad).is_empty());
    }
}

#[cfg(test)]
mod test_chunking {
    use super::*;

    #[test]
    fn test_stream_boundary_recovery() {
        let mut ingest = TelemetryIngest::new(35.0, 139.0, 200.0, 1000);
        let telem = NavTelemetry {
            timestamp_ms: 2000,
            pos_ned_m: [0.0, 0.0, 0.0],
            vel_ned_mps: [0.0, 0.0, 0.0],
            euler_rad: [0.0, 0.0, 0.0],
            status: 0,
        };
        let mut buf = [0u8; 64];
        let len = telem.pack_uart(&mut buf).unwrap();
        assert!(ingest.process_bytes(&buf[..len / 2]).is_empty());
        let res = ingest.process_bytes(&buf[len / 2..len]);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].timestamp_ms, 2000);
    }
}
