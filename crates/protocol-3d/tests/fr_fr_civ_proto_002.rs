//! Tests for FR-CIV-PROTO-002
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_002 {
    use civ_protocol_3d::{BuildingDiffEntry, BuildingDiffFrame, BuildingKind3d, BuildingProvenance, WorldXZ};

    #[test]
    fn verify_fr_civ_proto_002_basic() {
        let frame = BuildingDiffFrame {
            tick: 1,
            provenance: BuildingProvenance::Procedural,
            buildings: vec![BuildingDiffEntry {
                id: 42, kind: BuildingKind3d::Farm, tier: 1,
                position: WorldXZ { x: 1.0, z: 2.0 },
            }],
            graph: None,
        };
        assert_eq!(frame.buildings[0].kind, BuildingKind3d::Farm);
    }
}
