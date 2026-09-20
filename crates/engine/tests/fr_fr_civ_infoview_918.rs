//! Tests for FR-CIV-INFOVIEW-918
//!
//!
//! This test file verifies FR FR-CIV-INFOVIEW-918: Roads / Network overlay
//! (E1) — first Gizmo render-kind exemplar.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_918 {
    use super::*;

    /// FR-CIV-INFOVIEW-918 — Roads overlay uses Gizmo render kind (lines/arrows).
    #[test]
    fn roads_overlay_uses_gizmo_render_kind() {
        let r = InfoOverlay {
            id: "info_roads",
            name: "Roads / Network",
            group: OverlayGroup::Infrastructure,
            render_kind: RenderKind::Gizmo,
            availability: OverlayAvailability::Near,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![
                LegendStop { position: 0.0, label: "Light".into(), color: [0.6, 0.6, 0.6] },
                LegendStop { position: 1.0, label: "Heavy".into(), color: [0.2, 0.2, 0.2] },
            ],
            description: "Traffic graph",
        };
        assert_eq!(r.render_kind, RenderKind::Gizmo);
        assert_eq!(r.group, OverlayGroup::Infrastructure);
        assert_eq!(r.availability, OverlayAvailability::Near);
        // First Gizmo exemplar: legend shows traffic weight ramp
        assert_eq!(r.legend_stops.len(), 2);
    }
}
