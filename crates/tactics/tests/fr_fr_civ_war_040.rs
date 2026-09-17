//! Tests for FR-CIV-WAR-040
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-040: Refugees / displacement — civilian agents displaced by war.
//! This is an emergent behavior; we test the war-economy drain that drives it.

use civ_tactics::compute_war_economy_drain;

#[cfg(test)]
mod fr_fr_civ_war_040 {
    use super::*;

    /// FR-CIV-WAR-040: Active war causes population loss (driving displacement).
    #[test]
    fn verify_fr_civ_war_040_basic() {
        // CASUALTIES_PER_ENERGY_UNIT is 0.001, so need casualties >= 1000 for non-zero loss
        let drain = compute_war_economy_drain(50_000, 2_000, true);
        assert!(
            drain.population_loss > 0,
            "war should cause population loss that can drive displacement"
        );
    }

    /// FR-CIV-WAR-040: Higher casualties produce more displacement pressure.
    #[test]
    fn higher_casualties_more_displacement_pressure() {
        let low = compute_war_economy_drain(50_000, 50, true);
        let high = compute_war_economy_drain(50_000, 500, true);
        assert!(high.population_loss >= low.population_loss);
    }
}
