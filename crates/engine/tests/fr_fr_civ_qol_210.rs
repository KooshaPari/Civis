//! Tests for FR-CIV-QOL-210
//! Epic: FR-CIV-QOL. Settings persistence.
#[cfg(test)]
mod fr_fr_civ_qol_210 {
    #[test]
    fn world_state_serializable_for_settings() {
        let ws = civ_engine::WorldState::default();
        // Settings persistence requires JSON round-trip capability.
        let json = serde_json::to_string(&ws).expect("must serialize");
        let ws2: civ_engine::WorldState = serde_json::from_str(&json).expect("must deserialize");
        assert_eq!(ws.tick, ws2.tick);
    }
}
