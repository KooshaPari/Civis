//! Tests for FR-CIV-TACTICS-065
//!
//! Epic: FR-CIV-TACTICS
//! Status: IMPL-NO-TEST
//!
//! FR-CIV-TACTICS-065: Different formation kinds produce different spatial patterns.

use civ_tactics::{formation_offsets, FormationKind};

#[cfg(test)]
mod fr_fr_civ_tactics_065 {
    use super::*;

    /// FR-CIV-TACTICS-065: Square and wedge formations produce different offsets.
    #[test]
    fn verify_fr_civ_tactics_065_basic() {
        let square = formation_offsets(FormationKind::Square, 9);
        let wedge = formation_offsets(FormationKind::Wedge, 9);
        assert_ne!(square, wedge, "square and wedge should differ");
    }

    /// FR-CIV-TACTICS-065: Single unit always produces exactly one offset.
    #[test]
    fn single_unit_formation_is_always_one() {
        for kind in [FormationKind::Line, FormationKind::Column, FormationKind::Square, FormationKind::Wedge] {
            let offsets = formation_offsets(kind, 1);
            assert_eq!(offsets.len(), 1, "single unit should produce one offset");
        }
    }
}
