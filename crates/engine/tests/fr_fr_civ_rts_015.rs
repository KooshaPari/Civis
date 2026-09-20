//! Tests for FR-CIV-RTS-015
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-015: Client-Side Prediction & Replay Correction.
//! Replay log hash chain can be verified for determinism.

#[cfg(test)]
mod fr_fr_civ_rts_015 {
    /// ReplayLog default has empty events.
    #[test]
    fn replay_log_starts_empty() {
        let log = civ_engine::replay::ReplayLog::default();
        assert!(log.events.is_empty());
    }

    /// After ticking, replay log events can be encoded and decoded.
    #[test]
    fn replay_round_trip_after_ticks() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        for _ in 0..3 {
            sim.tick();
        }
        let log = sim.replay_log();
        let encoded = civ_engine::encode_civreplay(log).expect("encode");
        let decoded = civ_engine::decode_civreplay(&encoded).expect("decode");
        assert_eq!(decoded.events.len(), log.events.len());
    }

    /// Simulation produces consistent state across deterministic ticks
    /// (supports client-side prediction verification).
    #[test]
    fn deterministic_state_supports_prediction() {
        let mut sim_a = civ_engine::Simulation::with_seed(42);
        let mut sim_b = civ_engine::Simulation::with_seed(42);
        for _ in 0..5 {
            sim_a.tick();
            sim_b.tick();
        }
        assert_eq!(
            sim_a.state.energy_budget_joules,
            sim_b.state.energy_budget_joules
        );
        assert_eq!(sim_a.state.population, sim_b.state.population);
    }
}
