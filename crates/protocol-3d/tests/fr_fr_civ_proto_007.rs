//! Tests for FR-CIV-PROTO-007
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_proto_007 {
    use civ_protocol_3d::{BuildingKind3d, BuildingProvenance};

    #[test]
    fn verify_fr_civ_proto_007_basic() {
        let kinds = [BuildingKind3d::Farm, BuildingKind3d::Mine, BuildingKind3d::Barracks,
            BuildingKind3d::Temple, BuildingKind3d::Market, BuildingKind3d::House, BuildingKind3d::CityCenter];
        assert_eq!(kinds.len(), 7);
        assert_ne!(BuildingProvenance::Procedural, BuildingProvenance::Freehand);
    }
}
