//! Tests for FR-CIV-WAR-042
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-042: Reconstruction — post-war rebuilding via existing emergence.
//! Tests that economy exhaustion is detectable (a prerequisite for reconstruction timing).

use civ_tactics::compute_war_economy_drain;

#[cfg(test)]
mod fr_fr_civ_war_042 {
    use super::*;

    /// FR-CIV-WAR-042: Sustained war can exhaust the economy (reconstruction trigger).
    #[test]
    fn verify_fr_civ_war_042_basic() {
        let drain = compute_war_economy_drain(10, 100, true);
        assert!(
            drain.economically_exhausted,
            "very low treasury should be marked economically exhausted"
        );
    }

    /// FR-CIV-WAR-042: Healthy treasury is not exhausted.
    #[test]
    fn healthy_treasury_not_exhausted() {
        let drain = compute_war_economy_drain(1_000_000, 10, true);
        assert!(
            !drain.economically_exhausted,
            "large treasury should not be exhausted"
        );
    }
}
