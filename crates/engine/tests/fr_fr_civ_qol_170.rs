//! Tests for FR-CIV-QOL-170
//! Epic: FR-CIV-QOL. Photo mode.
#[cfg(test)]
mod fr_fr_civ_qol_170 {
    #[test]
    fn world_state_snapshot_for_photo_mode() {
        let ws = civ_engine::WorldState::default();
        let snapshot = ws.clone();
        assert_eq!(snapshot.tick, ws.tick);
        assert_eq!(snapshot.population, ws.population);
    }
}
