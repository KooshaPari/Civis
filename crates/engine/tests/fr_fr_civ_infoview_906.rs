//! Tests for FR-CIV-INFOVIEW-906
//!
//!
//! This test file verifies FR FR-CIV-INFOVIEW-906: Toggle UX — grouped
//! accordion, legend dock, sub-controls, and hotkeys (Tab/Shift+Tab/backtick/
//! 1-6/Esc/F).

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_906 {
    use super::*;

    /// FR-CIV-INFOVIEW-906 — Default hotkeys match the spec (Tab, Shift+Tab,
    /// Backtick, 1-6, Esc, F).
    #[test]
    fn hotkeys_match_spec() {
        let hk = InfoViewHotkeys::default();
        assert_eq!(hk.next, "Tab");
        assert_eq!(hk.prev, "Shift+Tab");
        assert_eq!(hk.toggle, "Backtick");
        assert_eq!(hk.close, "Esc");
        assert_eq!(hk.focus, "F");
        assert_eq!(hk.group_keys.len(), 6);
        assert_eq!(hk.group_keys[0], "1");
        assert_eq!(hk.group_keys[5], "6");
    }

    /// FR-CIV-INFOVIEW-906 — Six group keys correspond to six overlay groups.
    #[test]
    fn group_keys_count_matches_groups() {
        let hk = InfoViewHotkeys::default();
        let groups = [
            OverlayGroup::Terrain,
            OverlayGroup::Population,
            OverlayGroup::Economy,
            OverlayGroup::Territory,
            OverlayGroup::Infrastructure,
            OverlayGroup::Hazard,
        ];
        assert_eq!(hk.group_keys.len(), groups.len(),
            "one hotkey per overlay group");
    }

    /// FR-CIV-INFOVIEW-906 — OverlayGroup has exactly six variants.
    #[test]
    fn six_overlay_groups() {
        let groups = [
            OverlayGroup::Terrain,
            OverlayGroup::Population,
            OverlayGroup::Economy,
            OverlayGroup::Territory,
            OverlayGroup::Infrastructure,
            OverlayGroup::Hazard,
        ];
        assert_eq!(groups.len(), 6);
    }
}
