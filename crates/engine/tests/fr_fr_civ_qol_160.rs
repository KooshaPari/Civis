//! Tests for FR-CIV-QOL-160
//! Epic: FR-CIV-QOL. Game speed controls.
#[cfg(test)]
mod fr_fr_civ_qol_160 {
    use civ_engine::{Fixed, step, WorldState};

    #[test]
    fn step_advances_simulation_clock() {
        let ws = WorldState::default();
        let next = step(ws, Fixed::from_num(100));
        assert_eq!(next.tick, 1, "Step must advance tick by 1");
    }

    #[test]
    fn multiple_steps_advance_clock_proportionally() {
        let mut ws = WorldState::default();
        for i in 1..=10 {
            ws = step(ws, Fixed::from_num(10));
            assert_eq!(ws.tick, i);
        }
    }
}
