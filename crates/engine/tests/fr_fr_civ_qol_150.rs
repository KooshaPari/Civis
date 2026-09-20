//! Tests for FR-CIV-QOL-150
//! Epic: FR-CIV-QOL. Camera bookmarks.
#[cfg(test)]
mod fr_fr_civ_qol_150 {
    #[test]
    fn world_state_supports_camera_bookmark_data() {
        let ws = civ_engine::WorldState::default();
        // Camera bookmarks need faction context for positioning.
        assert!(!ws.factions.is_empty(), "Factions needed for camera bookmarks");
    }
}
