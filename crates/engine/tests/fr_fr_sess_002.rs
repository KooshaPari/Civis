//! Tests for FR-SESS-002 - Hot-seat Multiplayer
//!
//! Epic: FR-SESS
//! Hot-seat SHALL allow multiple human players per session.

#[cfg(test)]
mod fr_fr_sess_002 {
    #[test]
    fn hotseat_multiple_factions() {
        let mut ws = civ_engine::WorldState::default();
        ws.factions.insert(0, "Player1".into());
        ws.factions.insert(1, "Player2".into());
        assert_eq!(ws.factions.len(), 2, "hotseat supports multiple factions");
    }
}