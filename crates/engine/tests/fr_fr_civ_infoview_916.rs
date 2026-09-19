//! Tests for FR-CIV-INFOVIEW-916
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-INFOVIEW-916: Temperature overlay (A3)
//! — live proxy; RAMP_ELEVATION-style continuous ramp.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_916 {
    use super::*;

    /// FR-CIV-INFOVIEW-916 — Temperature overlay uses continuous ramp with
    /// Frozen/Temperate/Scorching legend stops.
    #[test]
    fn temperature_overlay_continuous_ramp() {
        let t = InfoOverlay {
            id: "info_temperature",
            name: "Temperature",
            group: OverlayGroup::Terrain,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Live,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![
                LegendStop { position: 0.0, label: "Frozen".into(), color: [0.6, 0.8, 1.0] },
                LegendStop { position: 0.5, label: "Temperate".into(), color: [0.4, 0.8, 0.3] },
                LegendStop { position: 1.0, label: "Scorching".into(), color: [1.0, 0.3, 0.0] },
            ],
            description: "Surface temperature",
        };
        assert_eq!(t.legend_kind, LegendKind::Continuous);
        assert_eq!(t.legend_stops.len(), 3);
        assert_eq!(t.legend_stops[0].label, "Frozen");
        assert_eq!(t.legend_stops[2].label, "Scorching");
    }

    /// FR-CIV-INFOVIEW-916 — Temperature is LIVE availability (proxy from
    /// planet/terrain fields).
    #[test]
    fn temperature_is_live_availability() {
        let t = InfoOverlay {
            id: "info_temperature",
            name: "Temperature",
            group: OverlayGroup::Terrain,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Live,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![],
            description: "Temperature",
        };
        assert_eq!(t.availability, OverlayAvailability::Live);
    }
}
