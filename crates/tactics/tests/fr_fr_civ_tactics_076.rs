//! Tests for FR-CIV-TACTICS-076
//!
//! Epic: FR-CIV-TACTICS
//! Status: IMPL-NO-TEST
//!
//! FR-CIV-TACTICS-076: Morale state management — construction, stance, casualty impact.

use civ_tactics::{MoraleState, UnitStance};

#[cfg(test)]
mod fr_fr_civ_tactics_076 {
    use super::*;

    /// FR-CIV-TACTICS-076: Fresh unit starts at Standing stance.
    #[test]
    fn verify_fr_civ_tactics_076_basic() {
        let morale = MoraleState::new(100, 30);
        assert_eq!(morale.stance(), UnitStance::Standing);
    }

    /// FR-CIV-TACTICS-076: Heavy casualties cause routing.
    #[test]
    fn morale_drops_to_routing_after_casualties() {
        let mut morale = MoraleState::new(100, 25);
        morale.apply_casualties(90);
        assert_eq!(morale.stance(), UnitStance::Routing);
    }
}
