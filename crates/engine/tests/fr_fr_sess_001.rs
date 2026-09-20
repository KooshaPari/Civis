//! Tests for FR-SESS-001 - PvE Session Mode
//!
//! Epic: FR-SESS
//! The engine SHALL support PvE (human vs AI) sessions.

#[cfg(test)]
mod fr_fr_sess_001 {
    #[test]
    fn pve_session_state_default() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0, "PvE session starts at tick zero");
    }
}