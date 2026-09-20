//! Tests for FR-CIV-POLITY-002
//!
//! Epic: FR-CIV-POLITY
//!
//! This test file verifies FR FR-CIV-POLITY-002: Diplomacy threshold crossings.

#[cfg(test)]
mod fr_fr_civ_polity_002 {
    /// Verify FR-CIV-POLITY-002: Large trade signal crosses to Alliance.
    #[test]
    fn verify_fr_civ_polity_002_basic() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        // Apply strong trade signal to push score above 0.5 (Alliance threshold)
        let signal = DiplomacySignal {
            trade_volume: 0.6,
            combat_grievance: 0.0,
            resource_competition: 0.0,
            proximity: 0.5,
            need_complementarity: 0.0,
            scarcity_pressure: 0.0,
        };
        let outcome = ws.faction_relations.apply_signal(0u32, 1u32, signal);
        assert_eq!(outcome.after, civ_agents::RelationKind::Alliance, "should cross to alliance");
    }

    /// Verify combat grievance pushes score negative.
    #[test]
    fn combat_grievance_negative() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        let signal = DiplomacySignal {
            trade_volume: 0.0,
            combat_grievance: 0.6,
            resource_competition: 0.0,
            proximity: 0.5,
            need_complementarity: 0.0,
            scarcity_pressure: 0.0,
        };
        let outcome = ws.faction_relations.apply_signal(0u32, 1u32, signal);
        assert!(outcome.score < 0.0, "score should be negative");
    }
}
