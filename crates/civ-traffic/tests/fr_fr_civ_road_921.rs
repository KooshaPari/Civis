//! Tests for FR-CIV-ROAD-921
//!
//! Upgraded from stub to real assertions.
//!
//! This test file verifies FR FR-CIV-ROAD-921.

#[cfg(test)]
mod fr_fr_civ_road_921 {
    /// Verify FR-CIV-ROAD-921 type existence and basic behavior.
    #[test]
    fn verify_fr_civ_road_921_basic() {
        use civ_traffic::{InfraProvenance, RoadKind, SCHEMA_VERSION};
        assert!(!SCHEMA_VERSION.is_empty());
        assert_eq!(RoadKind::None.speed_multiplier(), 1.0);
    }
}
