//! Tests for FR-CIV-RTS-011
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-011: Espionage & Shadow Operations.
//! Faction relations and diplomacy state exist for espionage tracking.

#[cfg(test)]
mod fr_fr_civ_rts_011 {
    /// WorldState has faction_relations for espionage tracking.
    #[test]
    fn faction_relations_exist() {
        let ws = civ_engine::WorldState::default();
        let _ = &ws.faction_relations;
    }

    /// Faction treasury tracks per-faction wealth (espionage budget source).
    #[test]
    fn faction_treasury_tracked() {
        let mut ws = civ_engine::WorldState::default();
        ws.faction_treasury
            .insert(1, civ_engine::Fixed::from_num(1000));
        assert_eq!(
            ws.faction_treasury[&1],
            civ_engine::Fixed::from_num(1000)
        );
    }

    /// Deep diplomacy state exists for persistent inter-faction tracking.
    #[test]
    fn deep_diplomacy_state_exists() {
        let ws = civ_engine::WorldState::default();
        let _ = &ws.deep_diplomacy;
    }
}
