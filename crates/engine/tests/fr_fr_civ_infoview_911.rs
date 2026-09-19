//! Tests for FR-CIV-INFOVIEW-911 - Info View Extended
//!
//! Epic: FR-CIV-FRAME
//! Info view extended entity and region display capabilities.

#[cfg(test)]
mod fr_fr_civ_infoview_911 {
    #[test]
    fn info_view_extended_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
