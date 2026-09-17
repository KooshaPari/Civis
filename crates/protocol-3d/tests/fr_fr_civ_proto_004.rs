//! Tests for FR-CIV-PROTO-004
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_004 {
    use civ_protocol_3d::{encode_frame3d_binary, Frame3d, FRAME3D_BINARY_HEADER_LEN, FRAME3D_BINARY_MAGIC};

    #[test]
    fn verify_fr_civ_proto_004_basic() {
        let frame = Frame3d::VoxelDelta {
            tick: 0, chunk_id: 0,
            material: civ_protocol_3d::MaterialId(0),
            write_seq: 0, dirty_cells: vec![],
        };
        let encoded = encode_frame3d_binary(&frame).unwrap();
        assert!(encoded.len() >= FRAME3D_BINARY_HEADER_LEN);
        assert_eq!(&encoded[..4], FRAME3D_BINARY_MAGIC);
    }
}
