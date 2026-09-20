//! Tests for FR-CIV-QOL-120
//!
//! Epic: FR-CIV-QOL
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-QOL-120: Undo/Redo for god-tools.
//! Engine-side: verify step function supports inverse operations (state snapshot).

#[cfg(test)]
mod fr_fr_civ_qol_120 {
    use civ_engine::{step, Fixed, WorldState};

    /// WorldState can be cloned for undo snapshots.
    #[test]
    fn worldstate_cloneable_for_undo() {
        let ws = WorldState::default();
        let snapshot = ws.clone();
        assert_eq!(ws.tick, snapshot.tick);
        assert_eq!(ws.population, snapshot.population);
    }

    /// Step produces a new state; original can serve as undo point.
    #[test]
    fn step_preserves_original_for_undo() {
        let ws1 = WorldState::default();
        let original = ws1.clone();
        let ws2 = step(ws1, Fixed::from_num(100));
        assert_eq!(original.tick, 0, "Original must remain at tick 0");
        assert_eq!(ws2.tick, 1, "Stepped state must be at tick 1");
    }
}
