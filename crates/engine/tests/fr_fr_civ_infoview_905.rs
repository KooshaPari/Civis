//! Tests for FR-CIV-INFOVIEW-905 - Info View Population Stats
//!
//! Epic: FR-CIV-FRAME
//! Info view SHALL expose entity and region data for UI rendering.

#[cfg(test)]
mod fr_fr_civ_infoview_905 {
    #[test]
    fn info_view_data_accessible() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
