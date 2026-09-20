//! Tests for FR-CIV-BRUSH-03
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-03.

#[cfg(test)]
mod fr_fr_civ_brush_03 {
    use civ_engine::brush_types::{ActionMode, ModeGroup};

    /// FR-CIV-BRUSH-03 — ActionMode carries kind, label, icon, and group.
    #[test]
    fn verify_fr_civ_brush_03_basic() {
        let mode = ActionMode {
            kind: civ_engine::brush_types::ActionKind::TerraformRaise,
            label: "Raise",
            icon: "raise",
            group: ModeGroup::Precise,
        };
        assert_eq!(mode.kind, civ_engine::brush_types::ActionKind::TerraformRaise);
        assert_eq!(mode.label, "Raise");
        assert_eq!(mode.group, ModeGroup::Precise);
    }
}
