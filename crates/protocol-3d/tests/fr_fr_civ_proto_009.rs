//! Tests for FR-CIV-PROTO-009
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_009 {
    use civ_protocol_3d::WorldXZ;

    #[test]
    fn verify_fr_civ_proto_009_basic() {
        let wxz = WorldXZ { x: 1.0, z: -2.5 };
        assert_eq!(wxz.x, 1.0);
        assert_eq!(wxz.z, -2.5);
    }
}
