//! Behavior tests for `NationColor hex parsing`.
//!
//! These assertions are real and were previously filed under `FR-CIV-RTS-RENDER-003`.
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
mod rts_nation_hex_parsing {
    use civ_engine::rts_types::NationColor;
    #[test]
    fn verify_fr_civ_rts_render_003_basic() {
        // Valid 6-char hex
        assert!(NationColor::parse_hex("#c8303c").is_some());
        // Invalid: too short
        assert!(NationColor::parse_hex("#c83").is_none());
        // Invalid: non-hex chars
        assert!(NationColor::parse_hex("#zzzzzz").is_none());
        // Without leading # still works
        assert_eq!(NationColor::parse_hex("c8303c"), Some([0xc8, 0x30, 0x3c]));
    }
}