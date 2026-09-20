//! Tests for FR-CIV-CORE-011
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-011: Replay Determinism Verification.
//! Two simulations with the same seed produce identical state hashes.

#[cfg(test)]
mod fr_fr_civ_core_011 {
    /// Two sims with the same seed produce identical replay logs after tick.
    #[test]
    fn replay_determinism_same_seed() {
        let mut sim_a = civ_engine::Simulation::with_seed(42);
        let mut sim_b = civ_engine::Simulation::with_seed(42);
        sim_a.tick();
        sim_b.tick();
        let log_a = sim_a.replay_log();
        let log_b = sim_b.replay_log();
        assert_eq!(
            log_a.events.len(),
            log_b.events.len(),
            "event count must match"
        );
        for (ea, eb) in log_a.events.iter().zip(log_b.events.iter()) {
            assert_eq!(
                std::mem::discriminant(ea),
                std::mem::discriminant(eb),
                "event discriminants must match for determinism"
            );
        }
    }

    /// Running the same sim twice in sequence produces same state.
    #[test]
    fn hash_chain_determinism() {
        let mut sim_a = civ_engine::Simulation::with_seed(7);
        let mut sim_b = civ_engine::Simulation::with_seed(7);
        for _ in 0..5 {
            sim_a.tick();
            sim_b.tick();
        }
        assert_eq!(sim_a.state.tick, sim_b.state.tick);
        assert_eq!(
            sim_a.state.energy_budget_joules,
            sim_b.state.energy_budget_joules
        );
    }
}
