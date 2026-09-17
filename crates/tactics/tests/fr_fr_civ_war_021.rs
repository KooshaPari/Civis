//! Tests for FR-CIV-WAR-021
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-021: Reads from psyche/agent state — morale modulates combat behavior.
//! Tests that morale state affects stance which modulates tactical outcomes.

use civ_tactics::{MoraleState, UnitStance};

#[cfg(test)]
mod fr_fr_civ_war_021 {
    use super::*;

    /// FR-CIV-WAR-021: Morale state modulates unit stance (standing vs routing).
    #[test]
    fn verify_fr_civ_war_021_basic() {
        let mut morale = MoraleState::new(100, 20);
        // Full strength = standing
        assert_eq!(morale.stance(), UnitStance::Standing);
        // After heavy casualties = routing (reads psyche-like state)
        morale.apply_casualties(95);
        assert_eq!(morale.stance(), UnitStance::Routing);
    }
}
