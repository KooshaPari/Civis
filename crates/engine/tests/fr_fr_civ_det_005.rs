//! Tests for FR-CIV-DET-005 — Determinism (energy budget non-negativity)
//!
//! Epic: FR-CIV-DET
//! Covers: FR-CIV-DET-005
//! Verifies the energy budget cannot go below zero.

#[cfg(test)]
mod fr_fr_civ_det_005 {
    /// FR-CIV-DET-005: Energy budget floors at zero when consumption exceeds budget.
    #[test]
    fn energy_budget_floors_at_zero() {
        let ws = civ_engine::WorldState {
            energy_budget_joules: civ_engine::Fixed::from_num(50),
            ..civ_engine::WorldState::default()
        };
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(100));
        assert_eq!(next.energy_budget_joules, civ_engine::Fixed::ZERO);
    }

    /// FR-CIV-DET-005: Default energy budget is positive.
    #[test]
    fn default_energy_budget_is_positive() {
        let ws = civ_engine::WorldState::default();
        assert!(ws.energy_budget_joules > civ_engine::Fixed::ZERO);
    }

    /// FR-CIV-DET-005: Step correctly subtracts consumption.
    #[test]
    fn step_subtracts_consumption() {
        let ws = civ_engine::WorldState {
            energy_budget_joules: civ_engine::Fixed::from_num(1000),
            ..civ_engine::WorldState::default()
        };
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(300));
        assert_eq!(next.energy_budget_joules, civ_engine::Fixed::from_num(700));
    }
}
