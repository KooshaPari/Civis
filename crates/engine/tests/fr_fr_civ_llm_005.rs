//! Tests for FR-CIV-LLM-005
//!
//! Epic: FR-CIV-LLM
//!
//! This test file verifies FR FR-CIV-LLM-005: Simulation state is cloneable.

#[cfg(test)]
mod fr_fr_civ_llm_005 {
    /// Verify FR-CIV-LLM-005: WorldState can be cloned and compared.
    #[test]
    fn verify_fr_civ_llm_005_basic() {
        let ws = civ_engine::WorldState::default();
        let ws2 = ws.clone();
        assert_eq!(ws.tick, ws2.tick);
        assert_eq!(ws.population, ws2.population);
        assert_eq!(ws.energy_budget_joules, ws2.energy_budget_joules);
    }

    /// Verify WorldState clone preserves faction data.
    #[test]
    fn world_state_clone_preserves_factions() {
        let mut ws = civ_engine::WorldState::default();
        let orig_len = ws.factions.len();
        ws.factions.insert(100, "TestFaction".to_string());
        let ws2 = ws.clone();
        assert_eq!(ws2.factions.len(), orig_len + 1);
        assert_eq!(ws2.factions.get(&100).unwrap(), "TestFaction");
    }
}
