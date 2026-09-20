//! Tests for FR-CIV-POLITY-001
//!
//! Epic: FR-CIV-POLITY
//!
//! This test file verifies FR FR-CIV-POLITY-001: Diplomacy relations matrix.

#[cfg(test)]
mod fr_fr_civ_polity_001 {
    /// Verify FR-CIV-POLITY-001: FactionRelations default is empty.
    #[test]
    fn verify_fr_civ_polity_001_basic() {
        let ws = civ_engine::WorldState::default();
        // Default world state has empty faction relations
        let rows: Vec<_> = ws.faction_relations.iter_rows().collect();
        assert!(rows.is_empty(), "default faction relations must be empty");
    }

    /// Verify FactionRelations records signals and updates scores.
    #[test]
    fn faction_relations_apply_signal() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        let signal = DiplomacySignal {
            trade_volume: 0.3,
            combat_grievance: 0.0,
            resource_competition: 0.0,
            proximity: 0.5,
            need_complementarity: 0.0,
            scarcity_pressure: 0.0,
        };
        let outcome = ws.faction_relations.apply_signal(0u32, 1u32, signal);
        assert_eq!(outcome.score, 0.3, "score should reflect trade volume");
        // Score goes from 0.0 (Neutral) to 0.3 (Trade) - this IS a threshold crossing
        assert_ne!(outcome.before, outcome.after, "score crosses Neutral->Trade threshold");
        assert_eq!(outcome.before, civ_agents::RelationKind::Neutral);
        assert_eq!(outcome.after, civ_agents::RelationKind::Trade);
        let record = ws.faction_relations.record(0u32, 1u32);
        assert!(record.is_some(), "record must exist after signal");
        assert_eq!(record.unwrap().samples, 1);
    }
}
