//! Tests for FR-CIV-MOD-018 — Manifest parsing
//!
//! Epic: FR-CIV-MOD
//! Verifies that parse_manifest works with valid TOML.

#[cfg(test)]
mod fr_fr_civ_mod_018 {
    /// FR-CIV-MOD-018: parse_manifest succeeds with valid minimal manifest.
    #[test]
    fn parse_valid_manifest() {
        let toml = r#"
[mod]
id = "test-mod"
name = "Test Mod"
version = "1.0.0"
api_version = "1"
mod_type = "policy"
author = "Tester"
description = "A test mod"

[dependencies]
civlab-api = ">=1"
"#;
        let manifest = civ_mod_host::parse_manifest(toml, "test.toml".as_ref())
            .expect("valid manifest should parse");
        assert_eq!(manifest.meta.id, "test-mod");
        assert_eq!(manifest.meta.mod_type, civ_mod_host::ModType::Policy);
    }

    /// FR-CIV-MOD-018: parse_manifest fails with invalid TOML.
    #[test]
    fn parse_invalid_toml_fails() {
        let result = civ_mod_host::parse_manifest("not valid toml {{{", "bad.toml".as_ref());
        assert!(result.is_err());
    }
}
