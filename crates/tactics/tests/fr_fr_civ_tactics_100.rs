//! Tests for FR-CIV-TACTICS-100
//!
//! Epic: FR-CIV-TACTICS
//! Status: SPEC-ONLY
//!
//! FR-CIV-TACTICS-100: Tactical combat resolution across the full war bridge pipeline.

use civ_tactics::compute_war_economy_drain;

#[cfg(test)]
mod fr_fr_civ_tactics_100 {
    use super::*;

    /// FR-CIV-TACTICS-100: War economy drain is zero when not at war.
    #[test]
    fn verify_fr_civ_tactics_100_basic() {
        let drain = compute_war_economy_drain(10_000, 50, false);
        assert_eq!(drain.treasury_drain, 0);
        assert_eq!(drain.population_loss, 0);
    }
}
