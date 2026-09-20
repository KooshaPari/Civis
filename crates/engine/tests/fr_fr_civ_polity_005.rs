//! Tests for FR-CIV-POLITY-005
//!
//! Epic: FR-CIV-POLITY
//!
//! This test file verifies FR FR-CIV-POLITY-005: Faction relations data integrity.

#[cfg(test)]
mod fr_fr_civ_polity_005 {
    /// Verify FR-CIV-POLITY-005: FactionRelations data survives field mutation.
    #[test]
    fn verify_fr_civ_polity_005_basic() {
        use civ_agents::DiplomacySignal;
        let mut ws = civ_engine::WorldState::default();
        let signal = DiplomacySignal {
            trade_volume: 0.3,
            combat_grievance: 0.1,
            resource_competition: 0.0,
            proximity: 0.5,
            need_complementarity: 0.0,
            scarcity_pressure: 0.0,
        };
        ws.faction_relations.apply_signal(0u32, 1u32, signal);
        // Verify data persists in the mutable reference
        let record = ws.faction_relations.record(0u32, 1u32);
        assert!(record.is_some(), "relation must exist after signal");
        assert_eq!(record.unwrap().samples, 1);
        assert!(record.unwrap().score > 0.0, "score should be positive");

        // Apply another signal and verify accumulation
        ws.faction_relations.apply_signal(0u32, 1u32, DiplomacySignal {
            trade_volume: 0.2,
            combat_grievance: 0.0,
            resource_competition: 0.0,
            proximity: 0.5,
            need_complementarity: 0.0,
            scarcity_pressure: 0.0,
        });
        let record = ws.faction_relations.record(0u32, 1u32).unwrap();
        assert_eq!(record.samples, 2, "samples should accumulate");
    }
}
