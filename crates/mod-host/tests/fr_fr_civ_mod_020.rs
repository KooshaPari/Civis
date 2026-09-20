//! Tests for FR-CIV-MOD-020 — Browser entries and archive support
//!
//! Epic: FR-CIV-MOD
//! Verifies browser entries and CIVMOD_MANIFEST_NAME constant.

#[cfg(test)]
mod fr_fr_civ_mod_020 {
    /// FR-CIV-MOD-020: CIVMOD_MANIFEST_NAME is manifest.toml.
    #[test]
    fn civmod_manifest_name() {
        assert_eq!(civ_mod_host::CIVMOD_MANIFEST_NAME, "manifest.toml");
    }

    /// FR-CIV-MOD-020: Empty host has no browser entries.
    #[test]
    fn empty_host_no_browser_entries() {
        let host = civ_mod_host::ModHost::new();
        let entries = host.browser_entries();
        assert!(entries.is_empty());
    }

    /// FR-CIV-MOD-020: Empty host has no loaded records.
    #[test]
    fn empty_host_no_loaded_records() {
        let host = civ_mod_host::ModHost::new();
        assert!(host.loaded_records().is_empty());
    }

    /// FR-CIV-MOD-020: Empty host has no enforcement violations.
    #[test]
    fn empty_host_no_violations() {
        let host = civ_mod_host::ModHost::new();
        assert_eq!(host.enforcement_violations("any-mod"), 0);
    }
}
