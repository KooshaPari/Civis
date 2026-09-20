//! Tests for FR-CIV-MOD-017 — ManifestError variants
//!
//! Epic: FR-CIV-MOD
//! Verifies that ManifestError has the expected error variants.

#[cfg(test)]
mod fr_fr_civ_mod_017 {
    /// FR-CIV-MOD-017: ManifestError::Io variant works.
    #[test]
    fn io_error_variant() {
        use civ_mod_host::ManifestError;
        let err = ManifestError::Io {
            path: "test.toml".into(),
            message: "not found".into(),
        };
        let display = format!("{err}");
        assert!(display.contains("test.toml"));
    }

    /// FR-CIV-MOD-017: ManifestError::Parse variant works.
    #[test]
    fn parse_error_variant() {
        use civ_mod_host::ManifestError;
        let err = ManifestError::Parse {
            path: "bad.toml".into(),
            message: "invalid TOML".into(),
        };
        let display = format!("{err}");
        assert!(display.contains("bad.toml"));
    }

    /// FR-CIV-MOD-017: ManifestError::Validation variant works.
    #[test]
    fn validation_error_variant() {
        use civ_mod_host::ManifestError;
        let err = ManifestError::Validation {
            path: "bad.toml".into(),
            message: "missing [mod]".into(),
        };
        let display = format!("{err}");
        assert!(display.contains("missing"));
    }
}
