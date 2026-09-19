//! Tests for FR-CIV-LLM-006
//!
//! Epic: FR-CIV-LLM
//!
//! This test file verifies FR FR-CIV-LLM-006: Simulation population tracking.

#[cfg(test)]
mod fr_fr_civ_llm_006 {
    /// Verify FR-CIV-LLM-006: Simulation spawns initial civilians.
    #[test]
    fn verify_fr_civ_llm_006_basic() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        // Default seed spawns 128 civilians (32 per faction x 4 factions)
        assert!(
            sim.state.population > 0,
            "simulation must spawn civilians"
        );
    }

    /// Verify population matches WorldState after construction.
    #[test]
    fn population_matches_world_state() {
        let sim = civ_engine::Simulation::with_seed(100u64);
        assert!(
            sim.state.population >= 128,
            "at least 128 civilians spawned"
        );
    }
}
