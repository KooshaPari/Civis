//! Tests for FR-NET-003 — Energy Budget Conservation
//!
//! Epic: FR-NET
//! Energy budget SHALL decrease monotonically with consumption
//! and floor at zero (no negative energy).

#[cfg(test)]
mod fr_fr_net_003 {
    /// FR-NET-003: Energy decreases by consumption amount.
    #[test]
    fn energy_decreases_by_consumption() {
        let ws = civ_engine::WorldState {
            energy_budget_joules: civ_engine::Fixed::from_num(1000),
            ..civ_engine::WorldState::default()
        };
        let ws2 = civ_engine::step(ws, civ_engine::Fixed::from_num(100));
        assert_eq!(ws2.energy_budget_joules, civ_engine::Fixed::from_num(900));
    }

    /// FR-NET-003: Energy floors at zero when consumption exceeds budget.
    #[test]
    fn energy_floors_at_zero() {
        let ws = civ_engine::WorldState {
            energy_budget_joules: civ_engine::Fixed::from_num(50),
            ..civ_engine::WorldState::default()
        };
        let ws2 = civ_engine::step(ws, civ_engine::Fixed::from_num(100));
        assert_eq!(ws2.energy_budget_joules, civ_engine::Fixed::ZERO);
    }

    /// FR-NET-003: Zero consumption leaves energy unchanged.
    #[test]
    fn zero_consumption_no_change() {
        let ws = civ_engine::WorldState {
            energy_budget_joules: civ_engine::Fixed::from_num(500),
            ..civ_engine::WorldState::default()
        };
        let ws2 = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
        assert_eq!(ws2.energy_budget_joules, civ_engine::Fixed::from_num(500));
    }
}