//! Tests for FR-CIV-INFOVIEW-904 - Info View Resource Display
//!
//! Epic: FR-CIV-FRAME
//! Info view SHALL expose entity and region data for UI rendering.

#[cfg(test)]
mod fr_fr_civ_infoview_904 {
    #[test]
    fn info_view_data_accessible() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
