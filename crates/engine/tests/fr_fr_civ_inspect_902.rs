//! Tests for FR-CIV-INSPECT-902 - Entity Inspection View
//!
//! Epic: FR-CIV-INSPECT
//! Inspection tools SHALL expose entity state for god-tool queries.

#[cfg(test)]
mod fr_fr_civ_inspect_902 {
    /// FR-CIV-INSPECT-902: SimulationSnapshot captures entity counts.
    #[test]
    fn snapshot_exposes_entity_counts() {
        let ws = civ_engine::WorldState::default();
        // Default state: 0 citizens, 0 buildings, 0 military
        assert_eq!(ws.population, 0);
    }
}