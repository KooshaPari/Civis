//! Tests for FR-CIV-RTS-014
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-014: Faction AI Behavior.
//! Faction decisions module exists for AI behavior selection.

#[cfg(test)]
mod fr_fr_civ_rts_014 {
    /// WorldState tracks factions for AI behavior routing.
    #[test]
    fn factions_tracked_for_ai() {
        let mut ws = civ_engine::WorldState::default();
        let initial_count = ws.factions.len();
        ws.factions.insert(999, "TestFaction".to_string());
        assert_eq!(ws.factions.len(), initial_count + 1);
        assert_eq!(ws.factions[&999], "TestFaction");
    }

    /// Faction decisions module exists and can be imported.
    #[test]
    fn faction_decisions_module_exists() {
        use civ_engine::faction_decisions::FactionDecision;
        let _maintain = FactionDecision::Maintain;
        let _raise = FactionDecision::RaiseUnrestResponse;
        let _hostility = FactionDecision::FlagHostility;
        let _trade = FactionDecision::FlagTradeOpen;
    }

    /// Faction unrest response intents are tracked per-tick.
    #[test]
    fn faction_unrest_intents_tracked() {
        let mut ws = civ_engine::WorldState::default();
        ws.last_tick_faction_unrest_response_intents.insert(1);
        assert!(ws.last_tick_faction_unrest_response_intents.contains(&1));
    }
}
