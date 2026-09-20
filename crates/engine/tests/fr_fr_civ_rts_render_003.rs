//! Tests for FR-CIV-RTS-RENDER-003
//!
//!
//! This test file verifies FR FR-CIV-RTS-RENDER-003.
//! Maps to CIV-0600 FR-CIV-ASSET-004: Background Removal Quality Gate.

#[cfg(test)]
mod fr_fr_civ_rts_render_003 {
    use civ_engine::rts_types::NationColor;

    /// FR-CIV-RTS-RENDER-003 -- Invalid hex strings return None (quality gate rejects bad input).
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
