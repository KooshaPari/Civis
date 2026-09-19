//! Tests for FR-CIV-QOL-230
//! Epic: FR-CIV-QOL. Legends data browser.
#[cfg(test)]
mod fr_fr_civ_qol_230 {
    #[test]
    fn significance_accumulator_exists_for_legends() {
        // FR-CIV-QOL-230 requires Legends event data.
        // The significance accumulator tracks entity importance.
        let ws = civ_engine::WorldState::default();
        // Significance is default-constructed.
        let _sig = ws.significance;
    }

    #[test]
    fn world_state_has_legends_relevant_fields() {
        let ws = civ_engine::WorldState::default();
        // Chronology of significant events.
        assert!(ws.chronicle.is_empty());
        // Research progress for tech legends.
        assert!(ws.research_progress.is_empty());
    }
}
