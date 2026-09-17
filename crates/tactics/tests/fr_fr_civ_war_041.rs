//! Tests for FR-CIV-WAR-041
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-041: War-economy — mobilization pressure reshapes emergent market.
//! Tests that war drains treasury (economic impact).

use civ_tactics::{compute_war_economy_drain, apply_war_drain};

#[cfg(test)]
mod fr_fr_civ_war_041 {
    use super::*;

    /// FR-CIV-WAR-041: War drains treasury proportionally.
    #[test]
    fn verify_fr_civ_war_041_basic() {
        let treasury: i64 = 100_000;
        let drain = compute_war_economy_drain(treasury, 0, true);
        assert!(
            drain.treasury_drain > 0,
            "war should drain treasury"
        );
        let remaining = apply_war_drain(treasury, &drain);
        assert!(remaining < treasury, "treasury should decrease after war drain");
    }

    /// FR-CIV-WAR-041: Peace does not drain the economy.
    #[test]
    fn peace_preserves_treasury() {
        let treasury: i64 = 50_000;
        let drain = compute_war_economy_drain(treasury, 0, false);
        let remaining = apply_war_drain(treasury, &drain);
        assert_eq!(remaining, treasury, "peace should not drain treasury");
    }
}
