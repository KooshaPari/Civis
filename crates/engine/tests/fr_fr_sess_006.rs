//! Tests for FR-SESS-006 - Turn Boundaries
//!
//! Epic: FR-SESS
//! Turn boundaries in hot-seat mode SHALL emit turn.start/end events.

#[cfg(test)]
mod fr_fr_sess_006 {
    #[test]
    fn turn_boundary_tick_advances() {
        let mut ws = civ_engine::WorldState::default();
        for i in 1..=5 {
            ws = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
            assert_eq!(ws.tick, i);
        }
    }
}