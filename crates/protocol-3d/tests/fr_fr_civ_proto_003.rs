//! Tests for FR-CIV-PROTO-003
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_003 {
    use civ_protocol_3d::{encode_frame3d_binary, decode_frame3d_binary, Frame3d, FRAME3D_BINARY_MAGIC};

    #[test]
    fn verify_fr_civ_proto_003_basic() {
        let frame = Frame3d::VoxelDelta {
            tick: 42, chunk_id: 0,
            material: civ_protocol_3d::MaterialId(1),
            write_seq: 0, dirty_cells: vec![],
        };
        let encoded = encode_frame3d_binary(&frame).expect("encode");
        assert_eq!(&encoded[0..4], FRAME3D_BINARY_MAGIC);
        let decoded = decode_frame3d_binary(&encoded).expect("decode");
        match decoded {
            Frame3d::VoxelDelta { tick, .. } => assert_eq!(tick, 42),
            _ => panic!("expected VoxelDelta"),
        }
    }
}
