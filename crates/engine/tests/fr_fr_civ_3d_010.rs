//! Tests for FR-CIV-3D-010
//!
//! Epic: FR-CIV-3D
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-3D-010: VRAM Budget
//! Total GPU VRAM consumption does not exceed 512 MB.
//! Engine-side: verify WorldState serialization size is bounded.

#[cfg(test)]
mod fr_fr_civ_3d_010 {
    use civ_engine::WorldState;

    /// Default WorldState serialized size is well under VRAM-equivalent budget.
    #[test]
    fn worldstate_serialized_size_bounded() {
        let ws = WorldState::default();
        let json = serde_json::to_string(&ws).expect("serialize");
        // 512 MB in JSON terms would be absurdly large. A reasonable
        // WorldState should be under 100 KB serialized.
        assert!(
            json.len() < 100_000,
            "WorldState JSON size {} bytes exceeds 100KB sanity check",
            json.len()
        );
    }

    /// WorldState has expected default fields that contribute to size.
    #[test]
    fn worldstate_has_expected_fields() {
        let ws = WorldState::default();
        assert_eq!(ws.tick, 0);
        assert_eq!(ws.population, 1_000_000);
        assert_eq!(ws.factions.len(), 3);
        assert_eq!(ws.faction_treasury.len(), 3);
    }
}
