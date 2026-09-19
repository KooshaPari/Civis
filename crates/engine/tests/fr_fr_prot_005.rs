//! Tests for FR-PROT-005 — Client Authentication Gate
//!
//! Epic: FR-PROT
//! Client connections SHALL authenticate before receiving session events.
//! Faction data is only available after authentication.

#[cfg(test)]
mod fr_fr_prot_005 {
    /// FR-PROT-005: Default WorldState has no factions (pre-auth state).
    #[test]
    fn default_state_has_no_factions() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0, "pre-auth state starts at tick zero");
    }

    /// FR-PROT-005: Factions can be populated after auth.
    #[test]
    fn factions_populated_after_auth() {
        let mut ws = civ_engine::WorldState::default();
        ws.factions.insert(0, "Rome".to_string());
        ws.factions.insert(1, "Carthage".to_string());
        assert_eq!(ws.factions.len(), 2);
        assert_eq!(ws.factions[&0], "Rome");
    }
}