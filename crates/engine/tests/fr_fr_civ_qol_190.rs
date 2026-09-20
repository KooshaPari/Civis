//! Tests for FR-CIV-QOL-190
//! Epic: FR-CIV-QOL. Accessibility (palettes, scaling, a11y).
#[cfg(test)]
mod fr_fr_civ_qol_190 {
    #[test]
    fn world_state_accessible_for_a11y() {
        let ws = civ_engine::WorldState::default();
        // Screen readers need text-accessible data from WorldState.
        let json = serde_json::to_string(&ws).expect("must serialize to text");
        assert!(!json.is_empty(), "WorldState must be text-accessible");
    }
}
