//! Tests for FR-CIV-INFOVIEW-917
//!
//!
//! This test file verifies FR FR-CIV-INFOVIEW-917: Resource Deposits overlay
//! (A8) — NEAR availability; turns static world into a strategic map.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_917 {
    use super::*;

    /// FR-CIV-INFOVIEW-917 — Resource Deposits is registered with NEAR
    /// availability and belongs to Terrain group.
    #[test]
    fn resource_deposits_overlay_registration() {
        let mut registry = OverlayRegistry::new();
        registry.register(InfoOverlay {
            id: "info_resources",
            name: "Resource Deposits",
            group: OverlayGroup::Terrain,
            render_kind: RenderKind::LatticeRecolor,
            availability: OverlayAvailability::Near,
            legend_kind: LegendKind::Categorical,
            legend_stops: vec![],
            description: "Strategic resource locations",
        });
        let r = registry.find("info_resources").unwrap();
        assert_eq!(r.availability, OverlayAvailability::Near);
        assert_eq!(r.group, OverlayGroup::Terrain);
        assert_eq!(r.name, "Resource Deposits");
    }
}
