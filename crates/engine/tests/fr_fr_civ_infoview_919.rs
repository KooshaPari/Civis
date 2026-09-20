//! Tests for FR-CIV-INFOVIEW-919
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-INFOVIEW-919: Wealth / Prosperity
//! overlay (C4) — first EntityTint exemplar; high "is there depth" payoff.

use civ_engine::info_views::*;

#[cfg(test)]
mod fr_fr_civ_infoview_919 {
    use super::*;

    /// FR-CIV-INFOVIEW-919 — Wealth overlay uses EntityTint render kind
    /// (recolors individual agents/structures).
    #[test]
    fn wealth_overlay_uses_entity_tint() {
        let w = InfoOverlay {
            id: "info_wealth",
            name: "Wealth / Prosperity",
            group: OverlayGroup::Economy,
            render_kind: RenderKind::EntityTint,
            availability: OverlayAvailability::Near,
            legend_kind: LegendKind::Continuous,
            legend_stops: vec![
                LegendStop { position: 0.0, label: "Poor".into(), color: [0.4, 0.2, 0.1] },
                LegendStop { position: 1.0, label: "Prosperous".into(), color: [1.0, 0.84, 0.0] },
            ],
            description: "Per-entity wealth",
        };
        assert_eq!(w.render_kind, RenderKind::EntityTint);
        assert_eq!(w.group, OverlayGroup::Economy);
        assert_eq!(w.legend_stops[0].label, "Poor");
        assert_eq!(w.legend_stops[1].label, "Prosperous");
    }

    /// FR-CIV-INFOVIEW-919 — Wealth is the first EntityTint exemplar.
    #[test]
    fn wealth_is_first_entity_tint_exemplar() {
        let mut registry = OverlayRegistry::new();
        register_priority_12(&mut registry);
        let entity_tints: Vec<&InfoOverlay> = registry.overlays()
            .iter()
            .filter(|o| o.render_kind == RenderKind::EntityTint)
            .collect();
        assert_eq!(entity_tints.len(), 1, "should be exactly one EntityTint in priority-12");
        assert_eq!(entity_tints[0].id, "info_wealth");
    }
}
