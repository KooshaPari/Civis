//! Tests for FR-GUARD-001 — Guardrails (integrity checks)
//!
//! Epic: FR-GUARD
//! Verifies the integrity monitor catches hash chain tampering and invariant violations.

#[cfg(test)]
mod fr_fr_guard_001 {
    /// FR-GUARD-001: check_integrity passes on a fresh simulation.
    #[test]
    fn integrity_passes_on_fresh_sim() {
        let sim = civ_engine::Simulation::with_seed(1);
        civ_engine::check_integrity(&sim).expect("fresh sim should be valid");
    }

    /// FR-GUARD-001: check_integrity passes after a tick.
    #[test]
    fn integrity_passes_after_tick() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        sim.tick();
        civ_engine::check_integrity(&sim).expect("after tick should be valid");
    }

    /// FR-GUARD-001: check_integrity rejects tampered hash chain.
    #[test]
    fn integrity_rejects_tampered_hash() {
        let mut sim = civ_engine::Simulation::with_seed(7);
        sim.tick();
        sim.replay_log_mut().running_hash = Some([0xAA; 32]);
        let err = civ_engine::check_integrity(&sim).unwrap_err();
        assert_eq!(err, civ_engine::IntegrityError::HashChainMismatch);
    }

    /// FR-GUARD-001: check_tick_invariants passes after tick.
    #[test]
    fn tick_invariants_pass_after_tick() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        sim.tick();
        civ_engine::check_tick_invariants(&sim).expect("tick-level check");
    }
}
