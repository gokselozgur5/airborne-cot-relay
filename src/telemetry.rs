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

pub struct TelemetryIngest {
    rx_buf: Vec<u8>,
    origin_lat_deg: f64,
    origin_lon_deg: f64,
    origin_alt_m: f64,
    max_age_ms: u64,
    latest_pose: Option<PlatformPose>,
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
            latest_pose: None,
        }
    }
}

impl TelemetryIngest {
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
                if let Ok(telem) = NavTelemetry::unpack_uart(&frame) {
                    poses.push(self.update_from_nav(&telem));
                }
            }
        }
        poses
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
