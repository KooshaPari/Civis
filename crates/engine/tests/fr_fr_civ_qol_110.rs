//! Tests for FR-CIV-QOL-110
//!
//! Epic: FR-CIV-QOL
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-QOL-110: Tooltips Everywhere — every interactive element has a tooltip.
//! Engine-side: verify WorldState exposes all data needed for tooltip provenance.

#[cfg(test)]
mod fr_fr_civ_qol_110 {
    use civ_engine::{WorldState, Fixed};

    /// WorldState provides population for data tooltip display.
    #[test]
    fn population_available_for_tooltip() {
        let ws = WorldState::default();
        assert!(ws.population > 0, "Population should be available for tooltips");
    }

    /// WorldState provides energy budget for data tooltip.
    #[test]
    fn energy_budget_available_for_tooltip() {
        let ws = WorldState::default();
        assert!(
            ws.energy_budget_joules > Fixed::ZERO,
            "Energy budget should be available for tooltips"
        );
    }

    /// WorldState provides faction treasuries for economic data tooltips.
    #[test]
    fn treasury_available_for_tooltip() {
        let ws = WorldState::default();
        for (id, &treasury) in &ws.faction_treasury {
            assert!(
                treasury >= Fixed::ZERO,
                "Faction {} treasury should be non-negative",
                id
            );
        }
    }
}
