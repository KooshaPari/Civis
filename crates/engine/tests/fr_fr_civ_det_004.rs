//! Tests for FR-CIV-DET-004 — Determinism (tick monotonicity)
//!
//! Epic: FR-CIV-DET
//! Verifies that tick counters only advance forward.

#[cfg(test)]
mod fr_fr_civ_det_004 {
    /// FR-CIV-DET-004: Simulation tick starts at zero.
    #[test]
    fn tick_starts_at_zero() {
        let sim = civ_engine::Simulation::with_seed(1);
        assert_eq!(sim.state.tick, 0);
    }

    /// FR-CIV-DET-004: Tick advances by exactly 1 each time.
    #[test]
    fn tick_advances_by_one() {
        let mut sim = civ_engine::Simulation::with_seed(1);
        sim.tick();
        assert_eq!(sim.state.tick, 1);
        sim.tick();
        assert_eq!(sim.state.tick, 2);
        sim.tick();
        assert_eq!(sim.state.tick, 3);
    }

    /// FR-CIV-DET-004: step() function also advances tick by 1.
    #[test]
    fn step_advances_tick_by_one() {
        let ws = civ_engine::WorldState {
            tick: 10,
            ..civ_engine::WorldState::default()
        };
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
        assert_eq!(next.tick, 11);
    }
}
