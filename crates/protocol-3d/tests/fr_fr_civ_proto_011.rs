//! Tests for FR-CIV-PROTO-011
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_011 {
    use civ_protocol_3d::{encode_frame3d_binary, decode_frame3d_binary, Frame3d};

    #[test]
    fn verify_fr_civ_proto_011_basic() {
        let frame = Frame3d::VoxelDelta(civ_protocol_3d::VoxelDeltaFrame {
            tick: 99,
            deltas: vec![],
        });
        let a = encode_frame3d_binary(&frame).unwrap();
        let b = encode_frame3d_binary(&frame).unwrap();
        assert_eq!(a, b, "encoding must be deterministic");
    }
}
