//! Tests for FR-INT-001 — Integration (end-to-end simulation)
//!
//! Epic: FR-INT
//! Verifies that the full simulation pipeline (create, tick, integrity check) works end-to-end.

#[cfg(test)]
mod fr_fr_int_001 {
    /// FR-INT-001: Multi-tick simulation maintains integrity.
    #[test]
    fn multi_tick_simulation_maintains_integrity() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        for _ in 0..50 {
            sim.tick();
            civ_engine::check_integrity(&sim).expect("integrity should hold each tick");
        }
        assert_eq!(sim.state.tick, 50);
    }

    /// FR-INT-001: Hash chain root exists after ticking.
    #[test]
    fn hash_chain_root_exists_after_tick() {
        let mut sim = civ_engine::Simulation::with_seed(10);
        sim.tick();
        assert!(sim.hash_chain_root().is_some());
    }

    /// FR-INT-001: Tick invariants hold across 200 ticks.
    #[test]
    fn invariants_hold_across_200_ticks() {
        let mut sim = civ_engine::Simulation::with_seed(104);
        for _ in 0..200 {
            sim.tick();
            civ_engine::check_tick_invariants(&sim).expect("invariant");
        }
        assert_eq!(sim.state.tick, 200);
    }
}
