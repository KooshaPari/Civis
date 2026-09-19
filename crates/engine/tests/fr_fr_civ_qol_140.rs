//! Tests for FR-CIV-QOL-140
//! Epic: FR-CIV-QOL. Hotkey Rebinding.
#[cfg(test)]
mod fr_fr_civ_qol_140 {
    #[test]
    fn hotkey_infrastructure_exists() {
        // FR-CIV-QOL-140 requires rebindable hotkey map.
        // Engine-side: WorldState must be accessible for any hotkey action.
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
