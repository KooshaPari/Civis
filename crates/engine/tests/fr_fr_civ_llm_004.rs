//! Tests for FR-CIV-LLM-004
//!
//! Epic: FR-CIV-LLM
//!
//! This test file verifies FR FR-CIV-LLM-004: Simulation tick advances.

#[cfg(test)]
mod fr_fr_civ_llm_004 {
    /// Verify FR-CIV-LLM-004: Simulation::tick() advances state.
    #[test]
    fn verify_fr_civ_llm_004_basic() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        let tick_before = sim.state.tick;
        sim.tick();
        assert_eq!(sim.state.tick, tick_before + 1, "tick must increment");
    }

    /// Verify multiple ticks advance monotonically.
    #[test]
    fn multiple_ticks_advance() {
        let mut sim = civ_engine::Simulation::with_seed(7u64);
        for expected in 1..=10 {
            sim.tick();
            assert_eq!(sim.state.tick, expected);
        }
    }
}
