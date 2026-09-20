//! Tests for FR-CIV-POLITY-004
//!
//! Epic: FR-CIV-POLITY
//!
//! This test file verifies FR FR-CIV-POLITY-004: Deep diplomacy state.

#[cfg(test)]
mod fr_fr_civ_polity_004 {
    /// Verify FR-CIV-POLITY-004: DeepDiplomacyState default is empty.
    #[test]
    fn verify_fr_civ_polity_004_basic() {
        let ws = civ_engine::WorldState::default();
        assert!(
            ws.deep_diplomacy.active_wars.is_empty(),
            "default should have no active wars"
        );
        assert!(
            ws.deep_diplomacy.war_casualties.is_empty(),
            "default should have no war casualties"
        );
        assert!(
            ws.deep_diplomacy.faction_resources.is_empty(),
            "default should have no faction resources"
        );
    }

    /// Verify DeepDiplomacyState is Clone.
    #[test]
    fn deep_diplomacy_is_clone() {
        let ws = civ_engine::WorldState::default();
        let cloned = ws.deep_diplomacy.clone();
        assert_eq!(cloned.active_wars.len(), 0);
    }
}
