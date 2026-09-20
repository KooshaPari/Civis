//! Tests for FR-CIV-MOD-000 — Core manifest types
//!
//! Epic: FR-CIV-MOD
//! Verifies that core manifest types are constructable.

#[cfg(test)]
mod fr_fr_civ_mod_000 {
    /// FR-CIV-MOD-000: ModType variants exist.
    #[test]
    fn mod_type_variants() {
        assert_eq!(civ_mod_host::ModType::Policy, civ_mod_host::ModType::Policy);
        assert_eq!(civ_mod_host::ModType::Economic, civ_mod_host::ModType::Economic);
        assert_eq!(civ_mod_host::ModType::Event, civ_mod_host::ModType::Event);
        assert_eq!(civ_mod_host::ModType::Scenario, civ_mod_host::ModType::Scenario);
    }

    /// FR-CIV-MOD-000: ModManifest has required fields.
    #[test]
    fn manifest_has_required_fields() {
        let meta = civ_mod_host::ModMeta {
            id: "test-mod".into(),
            name: "Test Mod".into(),
            version: "1.0.0".into(),
            api_version: "1".into(),
            mod_type: civ_mod_host::ModType::Policy,
            author: "Tester".into(),
            description: "A test".into(),
            homepage: None,
            license: None,
            author_pubkey_hex: None,
        };
        assert_eq!(meta.id, "test-mod");
        assert_eq!(meta.version, "1.0.0");
    }
}
