//! Tests for FR-CIV-PROTO-004
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED
//!
//! The binary `F3D0` envelope always carries a fixed magic header. The internal
//! header length is not part of the public API, so this asserts the public
//! invariant instead: every encoded frame starts with the magic and exposes the
//! public `is_frame3d_binary` predicate.

#[cfg(test)]
mod fr_fr_civ_proto_004 {
    use civ_protocol_3d::{
        encode_frame3d_binary, is_frame3d_binary, Frame3d, VoxelDeltaFrame, FRAME3D_BINARY_MAGIC,
    };

    #[test]
    fn verify_fr_civ_proto_004_basic() {
        let frame = Frame3d::VoxelDelta(VoxelDeltaFrame {
            tick: 0,
            deltas: vec![],
        });
        let encoded = encode_frame3d_binary(&frame).expect("encode");

        assert!(
            encoded.len() >= FRAME3D_BINARY_MAGIC.len(),
            "an encoded frame is at least as long as its magic"
        );
        assert_eq!(&encoded[..4], FRAME3D_BINARY_MAGIC, "magic leads the frame");
        assert!(
            is_frame3d_binary(&encoded),
            "the public predicate recognises an encoded frame"
        );

        // A payload that does not start with the magic is rejected.
        assert!(!is_frame3d_binary(b"XXXX"));
        assert!(!is_frame3d_binary(&[]), "an empty payload is not a frame");
    }
}
