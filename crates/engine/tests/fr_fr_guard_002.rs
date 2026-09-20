//! Tests for FR-GUARD-002 — Guardrails (tick invariants)
//!
//! Epic: FR-GUARD
//! Verifies that tick-level invariants (population, energy, ledger) are enforced.

#[cfg(test)]
mod fr_fr_guard_002 {
    /// FR-GUARD-002: check_tick_invariants passes after normal tick.
    #[test]
    fn invariants_pass_after_tick() {
        let mut sim = civ_engine::Simulation::with_seed(104);
        sim.tick();
        civ_engine::check_tick_invariants(&sim).expect("invariants should hold");
    }

    /// FR-GUARD-002: check_tick_invariants rejects negative energy budget.
    #[test]
    fn invariants_reject_negative_energy() {
        let mut sim = civ_engine::Simulation::with_seed(3);
        sim.tick();
        sim.state.energy_budget_joules = civ_engine::Fixed::from_num(-1);
        let err = civ_engine::check_tick_invariants(&sim).unwrap_err();
        assert!(matches!(
            err,
            civ_engine::InvariantError::NegativeEnergyBudget { .. }
        ));
    }

    /// FR-GUARD-002: Empty ledger skips growth check.
    #[test]
    fn empty_ledger_skips_growth_check() {
        let sim = civ_engine::Simulation::with_seed(2);
        assert!(sim.economy_state.ledger.is_empty());
        civ_engine::check_tick_invariants(&sim).expect("no growth check when empty");
    }
}
