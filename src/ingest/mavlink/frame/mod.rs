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
}
