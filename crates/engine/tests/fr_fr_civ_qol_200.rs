//! Tests for FR-CIV-QOL-200
//! Epic: FR-CIV-QOL. Notification system extensions.
#[cfg(test)]
mod fr_fr_civ_qol_200 {
    #[test]
    fn chronicle_for_notification_events() {
        let ws = civ_engine::WorldState::default();
        // Chronicle is the engine-side event feed for notifications.
        assert!(ws.chronicle.is_empty(), "Fresh world starts with empty chronicle");
    }

    #[test]
    fn chronicle_dedup_index_exists() {
        let ws = civ_engine::WorldState::default();
        assert!(
            ws.chronicle_age.is_empty(),
            "Fresh world starts with empty dedup index"
        );
    }
}
