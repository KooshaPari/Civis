//! Tests for FR-CIV-ROAD-901
//!
//! Epic: auto-generated
//! Stub: TDD-red — replace with real FR assertions
//! Upgraded from stub to real assertions.
//!
//! This test file verifies FR FR-CIV-ROAD-901.

#[cfg(test)]
mod fr_fr_civ_road_901 {
    /// Verify FR-CIV-ROAD-901 type existence and basic behavior.
    #[test]
    fn verify_fr_civ_road_901_basic() {
        use civ_traffic::{InfraProvenance, RoadKind, SCHEMA_VERSION};
        assert!(!SCHEMA_VERSION.is_empty());
        assert_eq!(RoadKind::None.speed_multiplier(), 1.0);
    }
}
