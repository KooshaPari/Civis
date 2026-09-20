//! Tests for FR-CIV-VERIFY-001
//! Epic: FR-CIV-VERIFY. agent-smoke.ps1 default exits 0.
#[cfg(test)]
mod fr_fr_civ_verify_001 {
    #[test]
    fn engine_compiles_cleanly() {
        // FR-CIV-VERIFY-001: agent-smoke.ps1 should pass on clean checkout.
        // Engine-side: the crate must compile without errors.
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }

    #[test]
    fn world_state_default_is_sane() {
        let ws = civ_engine::WorldState::default();
        assert!(ws.population > 0);
        assert!(!ws.factions.is_empty());
        assert!(ws.energy_budget_joules > civ_engine::Fixed::ZERO);
    }
}
