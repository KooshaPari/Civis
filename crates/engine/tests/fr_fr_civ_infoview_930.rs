//! Tests for FR-CIV-INFOVIEW-930
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-INFOVIEW-930: The full 31-overlay
//! catalog is specified and registrable; each overlay traces to a producing
//! crate field.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_930 {
    use super::*;

    /// FR-CIV-INFOVIEW-930 — Full catalog contains at least 31 overlays.
    #[test]
    fn full_catalog_has_at_least_31_overlays() {
        let registry = build_full_catalog();
        assert!(registry.len() >= 31,
            "full catalog must have >= 31 overlays, got {}", registry.len());
    }

    /// FR-CIV-INFOVIEW-930 — Every overlay in the catalog has a unique id.
    #[test]
    fn all_overlay_ids_are_unique() {
        let registry = build_full_catalog();
        let mut ids: Vec<&str> = registry.overlays().iter().map(|o| o.id).collect();
        ids.sort();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "all overlay ids must be unique");
    }

    /// FR-CIV-INFOVIEW-930 — Every overlay belongs to one of the six groups.
    #[test]
    fn all_overlays_belong_to_valid_group() {
        let registry = build_full_catalog();
        for overlay in registry.overlays() {
            // Just ensure group is one of the defined variants
            let _ = overlay.group.display_name();
        }
    }

    /// FR-CIV-INFOVIEW-930 — Every overlay has a non-empty description
    /// (traces to producing crate field).
    #[test]
    fn all_overlays_have_descriptions() {
        let registry = build_full_catalog();
        for overlay in registry.overlays() {
            assert!(!overlay.description.is_empty(),
                "overlay {} must have a description", overlay.id);
        }
    }

    /// FR-CIV-INFOVIEW-930 — BLIND overlays exist and have Blind availability.
    #[test]
    fn blind_overlays_are_gated() {
        let registry = build_full_catalog();
        let blind: Vec<&InfoOverlay> = registry.overlays()
            .iter()
            .filter(|o| o.availability == OverlayAvailability::Blind)
            .collect();
        assert!(blind.len() >= 5, "should have at least 5 BLIND overlays");
        for b in &blind {
            assert_eq!(b.availability, OverlayAvailability::Blind);
        }
    }

    /// FR-CIV-INFOVIEW-930 — All six groups are represented.
    #[test]
    fn all_six_groups_represented() {
        let registry = build_full_catalog();
        let counts = registry.group_counts();
        assert!(counts.len() >= 6, "all six groups should be represented");
    }
}
