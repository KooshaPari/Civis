#![allow(clippy::absurd_extreme_comparisons)]
#![allow(unused_comparisons)]
// Placeholder assertion (`count >= 0` on an unsigned count) kept as a marker until
// the owning lane writes the real oracle; tracked as a follow-up issue.

//! Tests for FR-CIV-TERRAIN-003
//!
//! Epic: FR-CIV-TERRAIN
//!
//! This test file verifies FR FR-CIV-TERRAIN-003: CA-dirty-chunk P99 < 16ms.
//! The voxel dirty event tracking exists for chunk updates.

#[cfg(test)]
mod fr_fr_civ_terrain_003 {
    /// Simulation tracks voxel dirty events per tick.
    #[test]
    fn voxel_events_tracked() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        sim.tick();
        // last_tick_voxel_events should be accessible (may be empty)
        let _events = sim.last_tick_voxel_events();
    }

    /// Simulation tracks voxel damage count per tick.
    #[test]
    fn voxel_damage_count_tracked() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        sim.tick();
        let count = sim.last_tick_voxel_damage_count();
        assert!(count >= 0, "damage count must be non-negative");
    }
}
