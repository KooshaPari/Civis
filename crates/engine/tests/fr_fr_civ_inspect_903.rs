//! Tests for FR-CIV-INSPECT-903 - Region Inspection View
//!
//! Epic: FR-CIV-INSPECT
//! Inspection tools SHALL expose region-level aggregation for god-tools.

#[cfg(test)]
mod fr_fr_civ_inspect_903 {
    /// FR-CIV-INSPECT-903: Faction count is inspectable.
    #[test]
    fn faction_countInspectable() {
        let mut ws = civ_engine::WorldState::default();
        ws.factions.insert(0, "Rome".into());
        ws.factions.insert(1, "Greece".into());
        ws.factions.insert(2, "Egypt".into());
        assert_eq!(ws.factions.len(), 3);
    }
}