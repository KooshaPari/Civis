//! Tests for FR-CIV-RTS-003
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-003: Unit Group & Formation Control.
//! Formation types (Wedge, Line, Column, Square) must exist and be selectable.

#[cfg(test)]
mod fr_fr_civ_rts_003 {
    /// FormationKind enum exists with expected variants.
    #[test]
    fn formation_kind_variants_exist() {
        use civ_tactics::FormationKind;
        let _wedge = FormationKind::Wedge;
        let _line = FormationKind::Line;
        let _column = FormationKind::Column;
        let _square = FormationKind::Square;
    }

    /// formation_offsets returns positions for each formation type.
    #[test]
    fn formation_offsets_produce_positions() {
        use civ_tactics::{formation_offsets, FormationKind};
        let offsets_wedge = formation_offsets(FormationKind::Wedge, 4);
        assert_eq!(offsets_wedge.len(), 4, "Wedge should have 4 unit slots");
        let offsets_line = formation_offsets(FormationKind::Line, 3);
        assert_eq!(offsets_line.len(), 3, "Line should have 3 unit slots");
    }
}
