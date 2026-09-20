//! Tests for FR-CIV-INFOVIEW-902
//!
//!
//! This test file verifies FR FR-CIV-INFOVIEW-902: Overlays are grouped
//! into the six CS2-class groups (Terrain/Population/Economy/Territory/
//! Infrastructure/Hazard); the panel is generated from the grouped registry.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_902 {
    use super::*;

    /// FR-CIV-INFOVIEW-902 — All six CS2-class groups exist.
    #[test]
    fn all_six_groups_exist() {
        let groups = [
            OverlayGroup::Terrain,
            OverlayGroup::Population,
            OverlayGroup::Economy,
            OverlayGroup::Territory,
            OverlayGroup::Infrastructure,
            OverlayGroup::Hazard,
        ];
        assert_eq!(groups.len(), 6);
        // Each group has a non-empty display name
        for g in &groups {
            assert!(!g.display_name().is_empty());
        }
    }

    /// FR-CIV-INFOVIEW-902 — Groups are distinct (no two share a display name).
    #[test]
    fn group_display_names_are_unique() {
        let groups = [
            OverlayGroup::Terrain,
            OverlayGroup::Population,
            OverlayGroup::Economy,
            OverlayGroup::Territory,
            OverlayGroup::Infrastructure,
            OverlayGroup::Hazard,
        ];
        let mut names: Vec<&str> = groups.iter().map(|g| g.display_name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 6, "all group names must be unique");
    }

    /// FR-CIV-INFOVIEW-902 — Registry can filter overlays by group.
    #[test]
    fn registry_filters_by_group() {
        let mut registry = OverlayRegistry::new();
        registry.register(InfoOverlay {
            id: "elev",
            name: "Elevation",
            group: OverlayGroup::Terrain,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Live,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![],
            description: "Height",
        });
        registry.register(InfoOverlay {
            id: "pop",
            name: "Population",
            group: OverlayGroup::Population,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Live,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![],
            description: "Pop density",
        });

        let terrain = registry.overlays_in_group(OverlayGroup::Terrain);
        assert_eq!(terrain.len(), 1);
        assert_eq!(terrain[0].id, "elev");

        let pop = registry.overlays_in_group(OverlayGroup::Population);
        assert_eq!(pop.len(), 1);
        assert_eq!(pop[0].id, "pop");
    }
}
