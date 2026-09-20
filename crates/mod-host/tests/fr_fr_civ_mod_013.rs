//! Tests for FR-CIV-MOD-013 — Determinism scanning
//!
//! Epic: FR-CIV-MOD
//! Verifies scan_wasm_determinism on empty/invalid input.

#[cfg(test)]
mod fr_fr_civ_mod_013 {
    /// FR-CIV-MOD-013: scan_wasm_determinism_report on empty bytes returns parse error.
    #[test]
    fn scan_empty_wasm_returns_parse_error() {
        let result = civ_mod_host::scan_wasm_determinism_report(&[]);
        assert!(result.is_err());
    }

    /// FR-CIV-MOD-013: scan_wasm_determinism on empty bytes returns parse error.
    #[test]
    fn scan_empty_wasm_returns_error() {
        let result = civ_mod_host::scan_wasm_determinism(&[]);
        assert!(result.is_err());
    }

    /// FR-CIV-MOD-013: scan_wasm_determinism_report on minimal valid WASM succeeds.
    #[test]
    fn scan_minimal_valid_wasm_succeeds() {
        // Minimal valid WASM module: just a type section with no functions.
        // See: https://webassembly.github.io/spec/core/binary/index.html
        let minimal_wasm: &[u8] = &[
            0x00, 0x61, 0x73, 0x6D, // magic: \0asm
            0x01, 0x00, 0x00, 0x00, // version: 1
        ];
        let result = civ_mod_host::scan_wasm_determinism_report(minimal_wasm);
        assert!(result.is_ok(), "minimal valid WASM should parse: {result:?}");
    }
}
