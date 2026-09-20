//! Tests for FR-CIV-MOD-019 — Mod lifecycle records
//!
//! Epic: FR-CIV-MOD
//! Verifies ModLoadedRecord and ModUnloadedRecord construction.

#[cfg(test)]
mod fr_fr_civ_mod_019 {
    /// FR-CIV-MOD-019: ModLoadedRecord has required fields.
    #[test]
    fn loaded_record_fields() {
        let record = civ_mod_host::ModLoadedRecord {
            mod_id: "test-mod".into(),
            mod_name: "Test".into(),
            version: "1.0.0".into(),
            tick: 42,
        };
        assert_eq!(record.mod_id, "test-mod");
        assert_eq!(record.tick, 42);
    }

    /// FR-CIV-MOD-019: format_mod_loaded_event produces non-empty string.
    #[test]
    fn format_loaded_event() {
        let record = civ_mod_host::ModLoadedRecord {
            mod_id: "my-mod".into(),
            mod_name: "My Mod".into(),
            version: "2.0.0".into(),
            tick: 100,
        };
        let formatted = civ_mod_host::format_mod_loaded_event(&record);
        assert!(!formatted.is_empty());
        assert!(formatted.contains("my-mod"));
    }

    /// FR-CIV-MOD-019: format_mod_loaded_event_json produces valid JSON.
    #[test]
    fn format_loaded_event_json() {
        let record = civ_mod_host::ModLoadedRecord {
            mod_id: "my-mod".into(),
            mod_name: "My Mod".into(),
            version: "2.0.0".into(),
            tick: 100,
        };
        let json = civ_mod_host::format_mod_loaded_event_json(&record);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["mod_id"], "my-mod");
    }
}
