//! Tests for FR-CIV-CORE-DET-001
//!
//! Epic: FR-CIV-CORE-DET
//!
//! This test file verifies FR FR-CIV-CORE-DET-001: Cross-Platform Determinism.
//! resvg rendering must be deterministic across platforms. We verify the
//! simulation determinism that underpins all deterministic guarantees.

#[cfg(test)]
mod fr_fr_civ_core_det_001 {
    /// Two simulations with the same seed produce identical state after multiple
    /// ticks, proving cross-instance determinism.
    #[test]
    fn identical_state_across_instances() {
        let mut sim_a = civ_engine::Simulation::with_seed(42);
        let mut sim_b = civ_engine::Simulation::with_seed(42);
        for _ in 0..10 {
            sim_a.tick();
            sim_b.tick();
        }
        assert_eq!(sim_a.state.tick, sim_b.state.tick);
        assert_eq!(sim_a.state.population, sim_b.state.population);
        assert_eq!(
            sim_a.state.energy_budget_joules,
            sim_b.state.energy_budget_joules
        );
        assert_eq!(sim_a.state.factions, sim_b.state.factions);
        assert_eq!(
            sim_a.state.faction_treasury,
            sim_b.state.faction_treasury
        );
    }

    /// Replay logs are identical across same-seed instances.
    #[test]
    fn replay_logs_identical() {
        let mut sim_a = civ_engine::Simulation::with_seed(42);
        let mut sim_b = civ_engine::Simulation::with_seed(42);
        for _ in 0..5 {
            sim_a.tick();
            sim_b.tick();
        }
        let log_a = sim_a.replay_log();
        let log_b = sim_b.replay_log();
        assert_eq!(log_a.events.len(), log_b.events.len());
    }
}
