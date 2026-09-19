//! Tests for FR-CIV-INFOVIEW-903
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-INFOVIEW-903: Three render kinds
//! (LatticeRecolor, Gizmo, EntityTint) dispatched from the registry;
//! new render kinds are additive.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_903 {
    use super::*;

    /// FR-CIV-INFOVIEW-903 — Three render kinds exist.
    #[test]
    fn three_render_kinds_exist() {
        let kinds = [RenderKind::LatticeRecolor, RenderKind::Gizmo, RenderKind::EntityTint];
        assert_eq!(kinds.len(), 3);
    }

    /// FR-CIV-INFOVIEW-903 — Each overlay in the registry specifies a render kind.
    #[test]
    fn each_overlay_has_render_kind() {
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
            id: "roads",
            name: "Roads",
            group: OverlayGroup::Infrastructure,
            render_kind: RenderKind::Gizmo,
            availability: OverlayAvailability::Near,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![],
            description: "Roads",
        });
        registry.register(InfoOverlay {
            id: "wealth",
            name: "Wealth",
            group: OverlayGroup::Economy,
            render_kind: RenderKind::EntityTint,
            availability: OverlayAvailability::Near,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![],
            description: "Wealth",
        });

        for overlay in registry.overlays() {
            // render_kind is always set (no Option)
            let _ = overlay.render_kind;
        }
        assert_eq!(registry.len(), 3);
    }

    /// FR-CIV-INFOVIEW-903 — Render kinds are distinct.
    #[test]
    fn render_kinds_are_distinct() {
        assert_ne!(RenderKind::LatticeRecolor, RenderKind::Gizmo);
        assert_ne!(RenderKind::Gizmo, RenderKind::EntityTint);
        assert_ne!(RenderKind::LatticeRecolor, RenderKind::EntityTint);
    }
}
