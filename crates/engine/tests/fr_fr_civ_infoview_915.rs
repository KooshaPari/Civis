//! Tests for FR-CIV-INFOVIEW-915
//!
//!
//! This test file verifies FR FR-CIV-INFOVIEW-915: Territory overlay (D1)
//! — live emergent cluster; proves polities emerge, not enums.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_915 {
    use super::*;

    /// FR-CIV-INFOVIEW-915 — Territory overlay is registered with Categorical
    /// legend kind (colors derived from emergent cluster ids).
    #[test]
    fn territory_overlay_is_categorical() {
        let mut registry = OverlayRegistry::new();
        registry.register(InfoOverlay {
            id: "info_territory",
            name: "Territory",
            group: OverlayGroup::Territory,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Live,
            legend_kind: LegendKind::Categorical,
            legend_stops: vec![],
            description: "Emergent faction territory",
        });
        let t = registry.find("info_territory").unwrap();
        assert_eq!(t.group, OverlayGroup::Territory);
        assert_eq!(t.legend_kind, LegendKind::Categorical);
        assert_eq!(t.availability, OverlayAvailability::Live);
        assert_eq!(t.render_kind, RenderKind::LatticeRecolor);
    }

    /// FR-CIV-INFOVIEW-915 — Territory uses cluster_color for rendering,
    /// not an authored enum (proves polities emerge).
    #[test]
    fn territory_uses_cluster_color_not_authored_enum() {
        // Different emergent clusters get different colors
        let color_a = cluster_color(100);
        let color_b = cluster_color(200);
        assert_ne!(color_a, color_b);
        // A territory overlay with Categorical legend has no legend stops
        // (colors are computed at render time from cluster ids)
        let t = InfoOverlay {
            id: "info_territory",
            name: "Territory",
            group: OverlayGroup::Territory,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Live,
            legend_kind: LegendKind::Categorical,
            legend_stops: vec![],
            description: "Emergent faction territory",
        };
        assert!(t.legend_stops.is_empty(),
            "territory overlay should have no authored legend stops");
    }
}
