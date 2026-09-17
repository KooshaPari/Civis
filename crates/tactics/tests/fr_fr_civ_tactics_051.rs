//! Tests for FR-CIV-TACTICS-051
//!
//! Epic: FR-CIV-TACTICS
//! Status: IMPL-NO-TEST
//!
//! FR-CIV-TACTICS-051: Formation offset generation for squad positioning.

use civ_tactics::{formation_offsets, FormationKind};

#[cfg(test)]
mod fr_fr_civ_tactics_051 {
    use super::*;

    /// FR-CIV-TACTICS-051: Line formation produces correct number of offsets.
    #[test]
    fn verify_fr_civ_tactics_051_basic() {
        let offsets = formation_offsets(FormationKind::Line, 5);
        assert_eq!(offsets.len(), 5);
    }

    /// FR-CIV-TACTICS-051: Column formation produces unique positions.
    #[test]
    fn formation_column_offsets_are_unique() {
        let offsets = formation_offsets(FormationKind::Column, 4);
        let mut positions: Vec<_> = offsets.iter().collect();
        positions.sort();
        positions.dedup();
        assert_eq!(positions.len(), 4, "all column positions should be unique");
    }
}
