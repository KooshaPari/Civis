//! Tests for FR-CIV-WAR-012
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-012: Attrition & cohesion — drains strength/cohesion before engagements.

use civ_tactics::compute_war_economy_drain;

#[cfg(test)]
mod fr_fr_civ_war_012 {
    use super::*;

    /// FR-CIV-WAR-012: War drain is proportional to casualties.
    #[test]
    fn verify_fr_civ_war_012_basic() {
        let drain_low = compute_war_economy_drain(10_000, 10, true);
        let drain_high = compute_war_economy_drain(10_000, 100, true);
        assert!(
            drain_high.population_loss >= drain_low.population_loss,
            "higher casualties should cause more population loss"
        );
    }

    /// FR-CIV-WAR-012: Economy exhaustion flag set when treasury critically low.
    #[test]
    fn economy_exhaustion_detected() {
        let drain = compute_war_economy_drain(10, 0, true);
        assert!(
            drain.economically_exhausted,
            "very low treasury should trigger exhaustion"
        );
    }
}
