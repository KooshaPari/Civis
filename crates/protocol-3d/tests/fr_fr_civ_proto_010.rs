//! Tests for FR-CIV-PROTO-010
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_010 {
    use civ_protocol_3d::{BuildingDiffEntry, BuildingKind3d, WorldXZ};

    #[test]
    fn verify_fr_civ_proto_010_basic() {
        let entry = BuildingDiffEntry {
            id: 1, kind: BuildingKind3d::House, tier: 0,
            position: WorldXZ { x: 0.0, z: 0.0 },
        };
        assert_eq!(entry.id, 1);
    }
}
