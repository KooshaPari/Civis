//! Tests for FR-CIV-ROAD-920
//!
//! Epic: auto-generated
//! Upgraded from stub to real assertions.
//!
//! This test file verifies FR FR-CIV-ROAD-920.

#[cfg(test)]
mod fr_fr_civ_road_920 {
    /// Verify FR-CIV-ROAD-920 type existence and basic behavior.
    #[test]
    fn verify_fr_civ_road_920_basic() {
        use civ_traffic::{InfraProvenance, RoadKind, SCHEMA_VERSION};
        assert!(!SCHEMA_VERSION.is_empty());
        assert_eq!(RoadKind::None.speed_multiplier(), 1.0);
    }
}
