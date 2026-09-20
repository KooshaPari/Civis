//! Tests for FR-CIV-ROAD-910
//!
//! Upgraded from stub to real assertions.
//!
//! This test file verifies FR FR-CIV-ROAD-910.

#[cfg(test)]
mod fr_fr_civ_road_910 {
    /// Verify FR-CIV-ROAD-910 type existence and basic behavior.
    #[test]
    fn verify_fr_civ_road_910_basic() {
        use civ_traffic::{InfraProvenance, RoadKind, SCHEMA_VERSION};
        assert!(!SCHEMA_VERSION.is_empty());
        assert_eq!(RoadKind::None.speed_multiplier(), 1.0);
    }
}
