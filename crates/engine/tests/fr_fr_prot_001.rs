//! Tests for FR-PROT-001 — JSON-RPC 2.0 API State Model
//!
//! Epic: FR-PROT
//! The engine SHALL expose state compatible with JSON-RPC 2.0 transport.
//! WorldState fields must be serializable for wire transport.

#[cfg(test)]
mod fr_fr_prot_001 {
    /// FR-PROT-001: WorldState is serializable via serde_json.
    #[test]
    fn world_state_json_serializable() {
        let ws = civ_engine::WorldState::default();
        let json = serde_json::to_string(&ws).expect("WorldState must serialize to JSON");
        assert!(json.contains("tick"), "JSON must contain tick field");
        assert!(json.contains("population"), "JSON must contain population field");
    }

    /// FR-PROT-001: WorldState round-trips through JSON.
    #[test]
    fn world_state_json_roundtrip() {
        let ws = civ_engine::WorldState {
            tick: 42,
            population: 1000,
            rng_seed: 555,
            ..civ_engine::WorldState::default()
        };
        let json = serde_json::to_string(&ws).unwrap();
        let ws2: civ_engine::WorldState = serde_json::from_str(&json).unwrap();
        assert_eq!(ws2.tick, 42);
        assert_eq!(ws2.population, 1000);
        assert_eq!(ws2.rng_seed, 555);
    }
}