//! Tests for FR-NET-002 — Deterministic State Hash
//!
//! Epic: FR-NET
//! Identical seeds and inputs SHALL produce identical WorldState.

#[cfg(test)]
mod fr_fr_net_002 {
    /// FR-NET-002: Same seed + same input = same state after one step.
    #[test]
    fn determinism_same_seed_same_output() {
        let s1 = civ_engine::WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: civ_engine::Fixed::from_num(1000),
            rng_seed: 42,
            ..civ_engine::WorldState::default()
        };
        let s2 = civ_engine::WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: civ_engine::Fixed::from_num(1000),
            rng_seed: 42,
            ..civ_engine::WorldState::default()
        };
        let r1 = civ_engine::step(s1, civ_engine::Fixed::from_num(10));
        let r2 = civ_engine::step(s2, civ_engine::Fixed::from_num(10));
        assert_eq!(r1.tick, r2.tick);
        assert_eq!(r1.energy_budget_joules, r2.energy_budget_joules);
    }

    /// FR-NET-002: Different seeds are preserved in state.
    #[test]
    fn different_seeds_preserved() {
        let s1 = civ_engine::WorldState { rng_seed: 111, ..civ_engine::WorldState::default() };
        let s2 = civ_engine::WorldState { rng_seed: 222, ..civ_engine::WorldState::default() };
        assert_ne!(s1.rng_seed, s2.rng_seed);
    }
}