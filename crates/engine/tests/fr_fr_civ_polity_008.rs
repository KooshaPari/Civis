//! Tests for FR-CIV-POLITY-008
//!
//! Epic: FR-CIV-POLITY
//!
//! This test file verifies FR FR-CIV-POLITY-008: Faction relations apply_signal clamping.

#[cfg(test)]
mod fr_fr_civ_polity_008 {
    /// Verify FR-CIV-POLITY-008: Score is clamped to [-1.0, 1.0].
    #[test]
    fn verify_fr_civ_polity_008_basic() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        // Apply very large trade to test upper clamp
        let signal = DiplomacySignal {
            trade_volume: 10.0,
            combat_grievance: 0.0,
            resource_competition: 0.0,
            proximity: 0.5,
            need_complementarity: 0.0,
            scarcity_pressure: 0.0,
        };
        let outcome = ws.faction_relations.apply_signal(0u32, 1u32, signal);
        assert!(outcome.score <= 1.0, "score must be clamped to <= 1.0");
    }

    /// Verify negative clamping.
    #[test]
    fn negative_clamping() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        let signal = DiplomacySignal {
            trade_volume: 0.0,
            combat_grievance: 10.0,
            resource_competition: 0.0,
            proximity: 0.5,
            need_complementarity: 0.0,
            scarcity_pressure: 0.0,
        };
        let outcome = ws.faction_relations.apply_signal(0u32, 1u32, signal);
        assert!(outcome.score >= -1.0, "score must be clamped to >= -1.0");
    }

    /// Verify multiple signals accumulate.
    #[test]
    fn multiple_signals_accumulate() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        ws.faction_relations.apply_signal(0u32, 1u32, DiplomacySignal { trade_volume: 0.1, combat_grievance: 0.0, resource_competition: 0.0, proximity: 0.5, need_complementarity: 0.0, scarcity_pressure: 0.0 });
        ws.faction_relations.apply_signal(0u32, 1u32, DiplomacySignal { trade_volume: 0.1, combat_grievance: 0.0, resource_competition: 0.0, proximity: 0.5, need_complementarity: 0.0, scarcity_pressure: 0.0 });
        let record = ws.faction_relations.record(0u32, 1u32).unwrap();
        assert_eq!(record.samples, 2, "samples should be 2");
        assert!(record.score > 0.0, "score should be positive");
    }
}
