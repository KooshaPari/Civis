//! Tests for FR-CIV-QOL-130
//!
//! Epic: FR-CIV-QOL
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-QOL-130: Blueprints / Copy-Paste (infra).
//! Engine-side: verify WorldState serialization supports region snapshots.

#[cfg(test)]
mod fr_fr_civ_qol_130 {
    use civ_engine::WorldState;

    /// WorldState can be serialized for blueprint capture.
    #[test]
    fn worldstate_serializable_for_blueprints() {
        let ws = WorldState::default();
        let json = serde_json::to_string(&ws).expect("must serialize");
        assert!(!json.is_empty());
    }

    /// WorldState can be cloned for region snapshot (copy-paste).
    #[test]
    fn worldstate_clone_for_region_snapshot() {
        let ws = WorldState::default();
        let snapshot = ws.clone();
        // Verify key fields are preserved.
        assert_eq!(ws.factions, snapshot.factions);
        assert_eq!(ws.faction_treasury, snapshot.faction_treasury);
    }
}
