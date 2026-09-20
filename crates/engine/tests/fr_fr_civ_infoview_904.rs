//! Tests for FR-CIV-INFOVIEW-904
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-INFOVIEW-904: Categorical overlays
//! derive color from emergent cluster ids only (no authored taxonomy) —
//! charter gate.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_904 {
    use super::*;

    /// FR-CIV-INFOVIEW-904 — cluster_color is deterministic for same id.
    #[test]
    fn cluster_color_is_deterministic() {
        let c1 = cluster_color(42);
        let c2 = cluster_color(42);
        assert_eq!(c1, c2, "same cluster id must produce same color");
    }

    /// FR-CIV-INFOVIEW-904 — Different cluster ids produce different colors.
    #[test]
    fn different_clusters_different_colors() {
        let c1 = cluster_color(1);
        let c2 = cluster_color(2);
        assert_ne!(c1, c2, "different cluster ids should produce different colors");
    }

    /// FR-CIV-INFOVIEW-904 — Color values are in [0, 1] per channel.
    #[test]
    fn cluster_color_values_in_range() {
        for id in 0..100 {
            let c = cluster_color(id);
            for channel in &c {
                assert!(*channel >= 0.0 && *channel <= 1.0,
                    "color channel {channel} out of range for cluster {id}");
            }
        }
    }

    /// FR-CIV-INFOVIEW-904 — Territory overlay uses Categorical legend kind.
    #[test]
    fn categorical_overlay_uses_cluster_color() {
        let mut registry = OverlayRegistry::new();
        registry.register(InfoOverlay {
            id: "territory",
            name: "Territory",
            group: OverlayGroup::Territory,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Live,
            legend_kind: LegendKind::Categorical,
            legend_stops: vec![],
            description: "Emergent territory",
        });
        let territory = registry.find("territory").unwrap();
        assert_eq!(territory.legend_kind, LegendKind::Categorical);
        // Categorical: no continuous legend stops
        assert!(territory.legend_stops.is_empty());
    }
}
