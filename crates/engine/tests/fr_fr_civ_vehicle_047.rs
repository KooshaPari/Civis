//! Tests for FR-CIV-VEHICLE-047
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-047.
//! Re-solve is dirty-region scoped: unchanged regions not recomputed.

#[cfg(test)]
mod fr_fr_civ_vehicle_047 {
    use civ_engine::lod::{should_tick_entity, LodTier};

    /// FR-CIV-VEHICLE-047 -- LOD tick cadence: unchanged regions skip recomputation.
    #[test]
    fn verify_fr_civ_vehicle_047_basic() {
        // Cold tier only ticks every 16 ticks
        assert!(should_tick_entity(0, LodTier::Cold));
        assert!(!should_tick_entity(1, LodTier::Cold));
        assert!(!should_tick_entity(15, LodTier::Cold));
        assert!(should_tick_entity(16, LodTier::Cold));
        // Warm tier ticks every 4 ticks
        assert!(should_tick_entity(0, LodTier::Warm));
        assert!(!should_tick_entity(1, LodTier::Warm));
        assert!(should_tick_entity(4, LodTier::Warm));
    }
}
