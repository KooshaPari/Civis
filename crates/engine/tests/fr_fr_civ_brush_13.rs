//! Tests for FR-CIV-BRUSH-13
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-13.

#[cfg(test)]
mod fr_fr_civ_brush_13 {
    use civ_engine::brush_types::{ActionKind, TOTAL_ACTION_KINDS, all_action_kinds};

    /// FR-CIV-BRUSH-13 -- Total action kinds count and mutating distinction.
    #[test]
    fn verify_fr_civ_brush_13_basic() {
        let kinds = all_action_kinds();
        assert_eq!(kinds.len(), TOTAL_ACTION_KINDS);
        // Select and Inspect are non-mutating
        assert!(!ActionKind::Select.is_mutating());
        assert!(!ActionKind::Inspect.is_mutating());
        // Everything else is mutating
        assert!(ActionKind::TerraformRaise.is_mutating());
        assert!(ActionKind::MaterialReplace.is_mutating());
    }
}
