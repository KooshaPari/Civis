//! Tests for FR-CIV-VERIFY-002
//! Epic: FR-CIV-VERIFY. agent-smoke.ps1 -FullUnreal passes.
#[cfg(test)]
mod fr_fr_civ_verify_002 {
    #[test]
    fn engine_deterministic_under_replay() {
        // FR-CIV-VERIFY-002 requires determinism for UE integration.
        let ws1 = civ_engine::WorldState {
            rng_seed: 42,
            ..civ_engine::WorldState::default()
        };
        let ws2 = civ_engine::WorldState {
            rng_seed: 42,
            ..civ_engine::WorldState::default()
        };
        let r1 = civ_engine::step(ws1, civ_engine::Fixed::from_num(10));
        let r2 = civ_engine::step(ws2, civ_engine::Fixed::from_num(10));
        assert_eq!(r1.tick, r2.tick);
        assert_eq!(r1.energy_budget_joules, r2.energy_budget_joules);
    }
}
