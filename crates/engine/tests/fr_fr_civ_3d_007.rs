//! Tests for FR-CIV-3D-007
//!
//! Epic: FR-CIV-3D
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-3D-007: Agentic Generation Reproducibility
//! Given identical seed, the generation pipeline produces the same output.
//! Engine-side: verify seed-based reproducibility.

#[cfg(test)]
mod fr_fr_civ_3d_007 {
    use civ_engine::{create_rng, Fixed, WorldState, step};
    use rand::Rng;
    use rand::SeedableRng;

    /// Same seed + same steps = identical world state trajectory.
    #[test]
    fn identical_trajectory_from_same_seed() {
        let make_state = || WorldState {
            tick: 0,
            population: 1000,
            energy_budget_joules: Fixed::from_num(1_000_000),
            rng_seed: 7777,
            ..WorldState::default()
        };

        let mut ws1 = make_state();
        let mut ws2 = make_state();

        for _ in 0..50 {
            ws1 = step(ws1, Fixed::from_num(100));
            ws2 = step(ws2, Fixed::from_num(100));
        }

        assert_eq!(ws1.tick, ws2.tick);
        assert_eq!(ws1.energy_budget_joules, ws2.energy_budget_joules);
        assert_eq!(ws1.population, ws2.population);
    }

    /// RNG from same seed produces reproducible random values.
    #[test]
    fn rng_reproducibility() {
        let seed = 9999u64;
        let mut rng_a = create_rng(seed);
        let mut rng_b = create_rng(seed);

        for _ in 0..500 {
            let a: u32 = rng_a.gen_range(0..1000);
            let b: u32 = rng_b.gen_range(0..1000);
            assert_eq!(a, b, "RNG values must match for same seed");
        }
    }
}
