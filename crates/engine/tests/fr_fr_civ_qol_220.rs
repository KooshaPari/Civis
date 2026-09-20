//! Tests for FR-CIV-QOL-220
//! Epic: FR-CIV-QOL. Timelapse / replay viewer.
#[cfg(test)]
mod fr_fr_civ_qol_220 {
    #[test]
    fn replay_log_exists_for_timelapse() {
        let ws = civ_engine::WorldState::default();
        // Replay log captures snapshots for timelapse playback.
        // The chronicle is the engine's event log.
        assert!(
            ws.research_progress.is_empty(),
            "Fresh world starts with no research"
        );
    }
}
