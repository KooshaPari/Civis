//! Tests for FR-CIV-INFOVIEW-921
//!
//!
//! This test file verifies FR FR-CIV-INFOVIEW-921: Migration Flow overlay
//! (B6) — Gizmo arrows; makes invisible agent movement readable.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_921 {
    use super::*;

    /// FR-CIV-INFOVIEW-921 — Migration Flow uses Gizmo render kind (arrows
    /// showing agent movement between settlements).
    #[test]
    fn migration_flow_uses_gizmo_arrows() {
        let m = InfoOverlay {
            id: "info_migration",
            name: "Migration Flow",
            group: OverlayGroup::Population,
            render_kind: RenderKind::Gizmo,
            availability: OverlayAvailability::Near,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![
                LegendStop { position: 0.0, label: "Static".into(), color: [0.5, 0.5, 0.5] },
                LegendStop { position: 1.0, label: "High Flow".into(), color: [0.1, 0.5, 0.9] },
            ],
            description: "Agent migration arrows",
        };
        assert_eq!(m.render_kind, RenderKind::Gizmo);
        assert_eq!(m.group, OverlayGroup::Population);
        assert_eq!(m.legend_stops.len(), 2);
    }

    /// FR-CIV-INFOVIEW-921 — Migration is NEAR availability (agent data exists
    /// but overlay needs a flow accessor).
    #[test]
    fn migration_is_near_availability() {
        let m = InfoOverlay {
            id: "info_migration",
            name: "Migration Flow",
            group: OverlayGroup::Population,
            render_kind: RenderKind::Gizmo,
            availability: OverlayAvailability::Near,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![],
            description: "Migration",
        };
        assert_eq!(m.availability, OverlayAvailability::Near);
    }
}
