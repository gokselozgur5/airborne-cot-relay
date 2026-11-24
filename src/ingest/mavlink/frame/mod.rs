//! MAVLink v2 frame encoding and decoding.

pub mod decode;
pub mod encode;

pub use decode::decode_mavlink_v2_frame;
pub use encode::{
    build_mavlink_v2_frame, encode_mavlink_v2_attitude, encode_mavlink_v2_global_position,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::mavlink::types::*;

    #[test]
    fn test_encode_decode_attitude() {
        let frame = encode_mavlink_v2_attitude(42, 1, 1, 1000, 0.1, -0.2, 1.57);
        let pkt = decode_mavlink_v2_frame(&frame).expect("decodes attitude frame");
        assert_eq!(pkt.seq, 42);
        assert_eq!(pkt.sysid, 1);
        assert_eq!(pkt.compid, 1);
        assert_eq!(pkt.msgid, MSG_ID_ATTITUDE);
    }

    #[test]
    fn test_encode_decode_global_position() {
        let frame = encode_mavlink_v2_global_position(
            10,
            1,
            1,
            2500,
            377749000,
            -1224194000,
            150000,
            50000,
            100,
            -50,
            0,
            9000,
        );
        let pkt = decode_mavlink_v2_frame(&frame).expect("decodes global position frame");
        assert_eq!(pkt.seq, 10);
        assert_eq!(pkt.msgid, MSG_ID_GLOBAL_POSITION_INT);
    }

    #[test]
    fn test_zero_truncated_attitude_and_global_position() {
        let mut att_payload = [0u8; 16];
        att_payload[0..4].copy_from_slice(&1000u32.to_le_bytes());
        att_payload[4..8].copy_from_slice(&0.1f32.to_le_bytes());
        att_payload[8..12].copy_from_slice(&(-0.2f32).to_le_bytes());
        att_payload[12..16].copy_from_slice(&1.57f32.to_le_bytes());
        let att_frame =
            build_mavlink_v2_frame(1, 1, 1, MSG_ID_ATTITUDE, CRC_EXTRA_ATTITUDE, &att_payload);
        let pkt_att = decode_mavlink_v2_frame(&att_frame).expect("decodes zero-truncated attitude");
        if let MavlinkMessage::Attitude {
            time_boot_ms,
            roll,
            pitch,
            yaw,
        } = pkt_att.message
        {
            assert_eq!(time_boot_ms, 1000);
            assert!((roll - 0.1).abs() < 1e-5);
            assert!((pitch - (-0.2)).abs() < 1e-5);
            assert!((yaw - 1.57).abs() < 1e-5);
        } else {
            panic!("expected Attitude message");
        }

        let mut pos_payload = [0u8; 20];
        pos_payload[0..4].copy_from_slice(&2000u32.to_le_bytes());
        pos_payload[4..8].copy_from_slice(&377749000i32.to_le_bytes());
        pos_payload[8..12].copy_from_slice(&(-1224194000i32).to_le_bytes());
        pos_payload[12..16].copy_from_slice(&150000i32.to_le_bytes());
        pos_payload[16..20].copy_from_slice(&50000i32.to_le_bytes());
        let pos_frame = build_mavlink_v2_frame(
            2,
            1,
            1,
            MSG_ID_GLOBAL_POSITION_INT,
            CRC_EXTRA_GLOBAL_POSITION_INT,
            &pos_payload,
        );
        let pkt_pos = decode_mavlink_v2_frame(&pos_frame).expect("decodes zero-truncated pos");
        if let MavlinkMessage::GlobalPositionInt {
            time_boot_ms,
            lat,
            lon,
            alt,
            relative_alt,
            vx,
            vy,
            vz,
            hdg,
        } = pkt_pos.message
        {
            assert_eq!(time_boot_ms, 2000);
            assert_eq!(lat, 377749000);
            assert_eq!(lon, -1224194000);
            assert_eq!(alt, 150000);
            assert_eq!(relative_alt, 50000);
            assert_eq!(vx, 0);
            assert_eq!(vy, 0);
            assert_eq!(vz, 0);
            assert_eq!(hdg, 0);
        } else {
            panic!("expected GlobalPositionInt message");
        }
    }
}
