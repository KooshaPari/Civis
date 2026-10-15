//! Tests for FR-SESS-002 - Hot-seat Multiplayer
//!
//! Epic: FR-SESS
//! Hot-seat SHALL allow multiple human players per session.

#[cfg(test)]
mod fr_fr_sess_002 {
    #[test]
    fn hotseat_multiple_factions() {
        let ws = civ_engine::WorldState::default();
        assert!(
            ws.factions.len() >= 2,
            "hotseat supports multiple factions"
        );
    }
}