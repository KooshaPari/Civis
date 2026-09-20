//! Tests for FR-CIV-DET-007 — Determinism (RNG seed isolation)
//!
//! Epic: FR-CIV-DET
//! Verifies that seeded RNG produces different but reproducible sequences.

#[cfg(test)]
mod fr_fr_civ_det_007 {
    /// FR-CIV-DET-007: create_rng with same seed produces same first value.
    #[test]
    fn same_seed_same_rng_sequence() {
        use rand::Rng;
        let mut rng1 = civ_engine::create_rng(42);
        let mut rng2 = civ_engine::create_rng(42);
        let v1: u64 = rng1.gen();
        let v2: u64 = rng2.gen();
        assert_eq!(v1, v2);
    }

    /// FR-CIV-DET-007: Different seeds produce different first values.
    #[test]
    fn different_seeds_different_rng() {
        use rand::Rng;
        let mut rng1 = civ_engine::create_rng(1);
        let mut rng2 = civ_engine::create_rng(999);
        let v1: u64 = rng1.gen();
        let v2: u64 = rng2.gen();
        assert_ne!(v1, v2);
    }

    /// FR-CIV-DET-007: Simulation with seed 0 starts deterministically.
    #[test]
    fn seed_zero_deterministic() {
        let mut sim1 = civ_engine::Simulation::with_seed(0);
        let mut sim2 = civ_engine::Simulation::with_seed(0);
        sim1.tick();
        sim2.tick();
        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(
            sim1.state.energy_budget_joules,
            sim2.state.energy_budget_joules
        );
    }
}
