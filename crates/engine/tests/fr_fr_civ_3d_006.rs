//! Tests for FR-CIV-3D-006
//!
//! Epic: FR-CIV-3D
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-3D-006: Terrain Determinism
//! Given identical scenario seeds, terrain generation produces bit-identical output.
//! Engine-side: verify seeded RNG produces deterministic sequences.

#[cfg(test)]
mod fr_fr_civ_3d_006 {
    use civ_engine::{create_rng, step, Fixed, WorldState};
    use rand::SeedableRng;
    use rand::Rng;

    /// Same seed produces identical RNG sequences.
    #[test]
    fn same_seed_same_rng_sequence() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let vals1: Vec<u64> = (0..100).map(|_| rng1.gen()).collect();
        let vals2: Vec<u64> = (0..100).map(|_| rng2.gen()).collect();
        assert_eq!(vals1, vals2, "Same seed must produce same RNG sequence");
    }

    /// Different seeds produce different RNG sequences.
    #[test]
    fn different_seeds_different_sequences() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(99);

        let vals1: Vec<u64> = (0..10).map(|_| rng1.gen()).collect();
        let vals2: Vec<u64> = (0..10).map(|_| rng2.gen()).collect();
        assert_ne!(vals1, vals2, "Different seeds must produce different sequences");
    }

    /// Determinism: same WorldState + same seed -> same step output.
    #[test]
    fn deterministic_step_with_same_seed() {
        let ws = WorldState {
            rng_seed: 12345,
            ..WorldState::default()
        };
        let ws2 = WorldState {
            rng_seed: 12345,
            ..WorldState::default()
        };
        let r1 = step(ws, Fixed::from_num(10));
        let r2 = step(ws2, Fixed::from_num(10));
        assert_eq!(r1.tick, r2.tick);
        assert_eq!(r1.energy_budget_joules, r2.energy_budget_joules);
    }
}
