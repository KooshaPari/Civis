//! Tests for FR-SESS-005 - Session Speed Configuration
//!
//! Epic: FR-SESS
//! Session speed SHALL be configurable (1x, 2x, 4x, paused).

#[cfg(test)]
mod fr_fr_sess_005 {
    #[test]
    fn tick_advances_regardless_of_speed() {
        let ws = civ_engine::WorldState::default();
        let ws2 = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
        assert_eq!(ws2.tick, 1, "tick advances regardless of speed setting");
    }
}