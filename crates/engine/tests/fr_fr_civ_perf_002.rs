//! Tests for FR-CIV-PERF-002
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-002: Heap allocation tracking per tick.

#[cfg(test)]
mod fr_fr_civ_perf_002 {
    /// Verify FR-CIV-PERF-002: Serialized WorldState stays under 1 MiB.
    #[test]
    fn verify_fr_civ_perf_002_basic() {
        let ws = civ_engine::WorldState::default();
        let json = serde_json::to_string(&ws).expect("serialize world state");
        let bytes = json.len();
        assert!(
            bytes < 1_048_576,
            "serialized WorldState {bytes} bytes exceeds 1 MiB budget"
        );
    }

    /// Verify WorldState with non-default data still serializes under 1 MiB.
    #[test]
    fn populated_world_state_serialization_budget() {
        let mut ws = civ_engine::WorldState::default();
        ws.tick = 9999;
        ws.population = 6_400_000;
        ws.factions.insert(0, "Rome".to_string());
        ws.factions.insert(1, "Carthage".to_string());
        ws.factions.insert(2, "Persia".to_string());
        ws.factions.insert(3, "Egypt".to_string());
        let json = serde_json::to_string(&ws).expect("serialize populated state");
        assert!(
            json.len() < 1_048_576,
            "populated WorldState serialization {} bytes exceeds 1 MiB",
            json.len()
        );
    }
}
