//! Synthesizes PlatformPose from MAVLink attitude and global position messages.

use super::types::{MavlinkMessage, MavlinkPacket};
use crate::telemetry::PlatformPose;

#[derive(Debug, Default)]
pub struct MavlinkPoseSynthesizer {
    pub lat_deg: Option<f64>,
    pub lon_deg: Option<f64>,
    pub hae_m: Option<f64>,
    pub roll_rad: f64,
    pub pitch_rad: f64,
    pub yaw_rad: f64,
    pub last_timestamp_ms: u64,
}

impl MavlinkPoseSynthesizer {
    pub fn update(&mut self, pkt: &MavlinkPacket) -> Option<PlatformPose> {
        match pkt.message {
            MavlinkMessage::Attitude {
                time_boot_ms,
                roll,
                pitch,
                yaw,
            } => {
                self.roll_rad = roll as f64;
                self.pitch_rad = pitch as f64;
                self.yaw_rad = yaw as f64;
                self.last_timestamp_ms = time_boot_ms as u64;
            }
            MavlinkMessage::GlobalPositionInt {
                time_boot_ms,
                lat,
                lon,
                alt,
                ..
            } => {
                self.lat_deg = Some(lat as f64 * 1e-7);
                self.lon_deg = Some(lon as f64 * 1e-7);
                self.hae_m = Some(alt as f64 * 1e-3);
                self.last_timestamp_ms = time_boot_ms as u64;
            }
        }

        if let (Some(lat), Some(lon), Some(hae)) = (self.lat_deg, self.lon_deg, self.hae_m) {
            Some(PlatformPose::new(
                self.last_timestamp_ms,
                lat,
                lon,
                hae,
                self.roll_rad,
                self.pitch_rad,
                self.yaw_rad,
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::mavlink::frame::*;

    #[test]
    fn test_pose_synthesis() {
        let mut synth = MavlinkPoseSynthesizer::default();
        let att_frame = encode_mavlink_v2_attitude(1, 1, 1, 1000, 0.05, 0.02, 1.2);
        let pos_frame = encode_mavlink_v2_global_position(
            2, 1, 1, 1050, 410000000, 290000000, 350000, 200000, 0, 0, 0, 0,
        );

        let att_pkt = decode_mavlink_v2_frame(&att_frame).unwrap();
        let pos_pkt = decode_mavlink_v2_frame(&pos_frame).unwrap();

        assert!(synth.update(&att_pkt).is_none());
        let pose = synth
            .update(&pos_pkt)
            .expect("complete pose after position");
        assert_eq!(pose.timestamp_ms, 1050);
        assert!((pose.lat_deg - 41.0).abs() < 1e-6);
        assert!((pose.lon_deg - 29.0).abs() < 1e-6);
        assert!((pose.hae_m - 350.0).abs() < 1e-3);
        assert!((pose.yaw_rad - 1.2).abs() < 1e-4);
    }
}
