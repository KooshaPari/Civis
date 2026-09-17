//! Tests for FR-CIV-CORE-002
//!
//! Epic: FR-CIV-CORE
//! Status: CODE-ONLY-no-spec
//! Auto-generated test stub — 2026-09-16
//!
//! This test file verifies FR FR-CIV-CORE-002.
//! Fill in the test body with assertions that validate the requirement.

#[cfg(test)]
mod fr_fr_civ_core_002 {
    #[test]
    fn verify_fr_civ_core_002_basic() {
        // FR-CIV-CORE-002: WorldState has required fields and defaults
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
        assert_eq!(ws.faction_treasury.len(), ws.factions.len());
        assert_eq!(ws.faction_resources.len(), ws.factions.len());
    }

    #[test]
    fn verify_fr_civ_core_002_step_advances_tick() {
        let ws = civ_engine::WorldState::default();
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(100));
        assert_eq!(next.tick, 1);
    }

    #[test]
    fn verify_fr_civ_core_002_energy_floor_at_zero() {
        let ws = civ_engine::WorldState {
            energy_budget_joules: civ_engine::Fixed::from_num(50),
            ..civ_engine::WorldState::default()
        };
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(100));
        assert_eq!(next.energy_budget_joules, civ_engine::Fixed::ZERO);
    }
}
