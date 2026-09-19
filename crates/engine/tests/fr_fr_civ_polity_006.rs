//! Tests for FR-CIV-POLITY-006
//!
//! Epic: FR-CIV-POLITY
//!
//! This test file verifies FR FR-CIV-POLITY-006: Faction relations mean score.

#[cfg(test)]
mod fr_fr_civ_polity_006 {
    /// Verify FR-CIV-POLITY-006: mean_score_involving returns average.
    #[test]
    fn verify_fr_civ_polity_006_basic() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        // Apply two signals involving faction 0
        ws.faction_relations.apply_signal(0u32, 1u32, DiplomacySignal { trade_volume: 0.4, combat_grievance: 0.0, resource_competition: 0.0, proximity: 0.5, need_complementarity: 0.0, scarcity_pressure: 0.0 });
        ws.faction_relations.apply_signal(0u32, 2u32, DiplomacySignal { trade_volume: 0.2, combat_grievance: 0.0, resource_competition: 0.0, proximity: 0.5, need_complementarity: 0.0, scarcity_pressure: 0.0 });
        let mean = ws.faction_relations.mean_score_involving(0u32);
        assert!(mean.is_some(), "should have a mean for faction 0");
        let mean_val = mean.unwrap();
        // Both scores are 0.4 and 0.2 respectively, mean = 0.3
        assert!(
            (mean_val - 0.3).abs() < 0.01,
            "mean should be ~0.3, got {mean_val}"
        );
    }

    /// Verify mean_score_involving returns None for unknown faction.
    #[test]
    fn mean_unknown_faction() {
        let ws = civ_engine::WorldState::default();
        assert!(ws.faction_relations.mean_score_involving(99).is_none());
    }
}
