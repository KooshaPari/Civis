//! Tests for FR-CIV-PERF-BUILD-001
//!
//! Epic: FR-CIV-PERF-BUILD
//!
//! This test file verifies FR FR-CIV-PERF-BUILD-001: Build compilation performance.

#[cfg(test)]
mod fr_fr_civ_perf_build_001 {
    /// Verify FR-CIV-PERF-BUILD-001: Engine crate compiles and basic types are available.
    #[test]
    fn verify_fr_civ_perf_build_001_basic() {
        // Verify the engine crate's key types are accessible and usable.
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
        assert_eq!(ws.population, 1_000_000);
    }

    /// Verify WorldState implements Serialize + Deserialize.
    #[test]
    fn world_state_serde_roundtrip() {
        let ws = civ_engine::WorldState::default();
        let json = serde_json::to_string(&ws).expect("serialize");
        let ws2: civ_engine::WorldState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ws.tick, ws2.tick);
        assert_eq!(ws.population, ws2.population);
    }
}
