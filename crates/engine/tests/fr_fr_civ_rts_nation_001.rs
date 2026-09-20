//! Tests for FR-CIV-RTS-NATION-001
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-RTS-NATION-001.
//! Maps to CIV-0600 FR-CIV-ASSET-005: Nation Color Preservation in quantization.

#[cfg(test)]
mod fr_fr_civ_rts_nation_001 {
    use civ_engine::rts_types::{NationColor, BAKED_PRIMARY, BAKED_SECONDARY};

    /// FR-CIV-RTS-NATION-001 -- NationColor parses hex colors and extracts RGBA.
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
