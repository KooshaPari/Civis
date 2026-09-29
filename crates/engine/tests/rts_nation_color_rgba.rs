//! Behavior tests for `NationColor parse to RGBA`.
//!
//! These assertions are real and were previously filed under `FR-CIV-RTS-NATION-001`.
//! That ID is not a requirement: the only place it appears in the repository
//! is the middle column of CIV-0600's §14 traceability table
//! (`docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3205-3224`), which uses it
//! as a "verification owner" label for a test file that does not exist.
//! CIV-0300 §12.1 owns `FR-CIV-RTS-001..015` and never mentions the
//! RENDER/ZOOM/NATION sub-namespaces.
//!
//! So the ID was dropped rather than satisfied: there is no requirement text to
//! implement, and inventing one would be the same defect in a new place. The
//! behavior is still worth a test, so the test stays under a name that says
//! what it checks.

#[cfg(test)]
mod rts_nation_color_rgba {
    use civ_engine::rts_types::{NationColor, BAKED_PRIMARY, BAKED_SECONDARY};
    #[test]
    fn verify_fr_civ_rts_nation_001_basic() {
        let nation = NationColor {
            primary_hex: "#c8303c".into(),
            secondary_hex: "#f0c040".into(),
            dark_hex: "#8a1020".into(),
        };
        let primary = nation.primary_rgba().expect("should parse primary hex");
        assert_eq!(primary, [0xc8, 0x30, 0x3c, 255]);
        let secondary = nation.secondary_rgba().expect("should parse secondary hex");
        assert_eq!(secondary, [0xf0, 0xc0, 0x40, 255]);
        // Baked colors match the template defaults
        assert_eq!(BAKED_PRIMARY, "#c8303c");
        assert_eq!(BAKED_SECONDARY, "#f0c040");
    }
}