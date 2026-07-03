//! Coverage for previously untested pub fns: mod system lifecycle, policy,
//! and perf-adjacent helpers accessible from the civ-engine crate root.
//! FR-CIV-TEST-005

use civ_engine::{
    phases_over_budget, tick_over_budget, TickProfile,
    effective_consumption, format_mod_error_event, format_mod_error_event_json,
    format_mod_loaded_event, format_mod_loaded_event_json, format_mod_unloaded_event_json,
    policy_from_kind, CapitalistPolicy, ControlSignals, ModHost, ModLoadedRecord,
    ModUnloadedRecord, NoopPolicy, Policy, PolicyInput, SubsistenceFirstPolicy,
};

// ── Policy factory ────────────────────────────────────────────────────────────

/// Covers FR-CIV-TEST-005: policy_from_kind("noop") returns NoopPolicy.
#[test]
fn test_policy_from_kind_noop() {
    let p = policy_from_kind("noop");
    assert_eq!(p.name(), "noop");
}

/// Covers FR-CIV-TEST-005: policy_from_kind("capitalist") returns CapitalistPolicy.
#[test]
fn test_policy_from_kind_capitalist() {
    let p = policy_from_kind("capitalist");
    assert_eq!(p.name(), "capitalist");
}

/// Covers FR-CIV-TEST-005: policy_from_kind("subsistence_first") returns SubsistenceFirstPolicy.
#[test]
fn test_policy_from_kind_subsistence_first() {
    let p = policy_from_kind("subsistence_first");
    assert_eq!(p.name(), "subsistence_first");
}

/// Covers FR-CIV-TEST-005: unknown kind falls back to noop.
#[test]
fn test_policy_from_kind_unknown_falls_back_to_noop() {
    let p = policy_from_kind("banana");
    assert_eq!(p.name(), "noop");
}

/// Covers FR-CIV-TEST-005: empty string kind falls back to noop.
#[test]
fn test_policy_from_kind_empty_string() {
    let p = policy_from_kind("");
    assert_eq!(p.name(), "noop");
}

// ── ControlSignals ────────────────────────────────────────────────────────────

/// Covers FR-CIV-TEST-005: default ControlSignals is all-empty.
#[test]
fn test_control_signals_default_is_empty() {
    let cs = ControlSignals::default();
    assert!(cs.production_multipliers.is_empty());
    assert!(cs.allocation_weights.is_empty());
    assert!(cs.tax_rates.is_empty());
}

/// Covers FR-CIV-TEST-005: ControlSignals round-trips through JSON.
#[test]
fn test_control_signals_json_round_trip() {
    let mut cs = ControlSignals::default();
    cs.production_multipliers.insert("iron".to_owned(), 1.5);
    cs.allocation_weights.insert("food".to_owned(), 2.0);
    cs.tax_rates.insert(1, 500);
    let json = serde_json::to_string(&cs).expect("serialize");
    let back: ControlSignals = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, cs);
}

// ── effective_consumption ─────────────────────────────────────────────────────

/// Covers FR-CIV-TEST-005: effective_consumption with positive multiplier.
#[test]
fn test_effective_consumption_positive() {
    let input = PolicyInput {
        base_consumption_joules: 1_000.0,
        scarcity_multiplier: 2.0,
    };
    assert!((effective_consumption(input) - 2_000.0).abs() < 1e-6);
}

/// Covers FR-CIV-TEST-005: effective_consumption clamps negative multiplier to zero.
#[test]
fn test_effective_consumption_negative_multiplier_clamps() {
    let input = PolicyInput {
        base_consumption_joules: 500.0,
        scarcity_multiplier: -3.0,
    };
    assert_eq!(effective_consumption(input), 0.0);
}

/// Covers FR-CIV-TEST-005: effective_consumption with zero base returns zero.
#[test]
fn test_effective_consumption_zero_base() {
    let input = PolicyInput {
        base_consumption_joules: 0.0,
        scarcity_multiplier: 5.0,
    };
    assert_eq!(effective_consumption(input), 0.0);
}

// ── Mod system: format helpers ────────────────────────────────────────────────

/// Covers FR-CIV-TEST-005: format_mod_loaded_event produces expected text.
#[test]
fn test_format_mod_loaded_event() {
    let record = ModLoadedRecord {
        mod_id: "test-mod".to_owned(),
        mod_name: "Test Mod".to_owned(),
        version: "1.0.0".to_owned(),
        tick: 5,
    };
    let line = format_mod_loaded_event(&record);
    assert!(line.contains("mod.loaded.v1"), "got: {line}");
    assert!(line.contains("test-mod"), "got: {line}");
    assert!(line.contains("tick=5"), "got: {line}");
}

/// Covers FR-CIV-TEST-005: format_mod_loaded_event_json is valid JSON with required keys.
#[test]
fn test_format_mod_loaded_event_json() {
    let record = ModLoadedRecord {
        mod_id: "json-mod".to_owned(),
        mod_name: "JSON Mod".to_owned(),
        version: "2.0.0".to_owned(),
        tick: 99,
    };
    let json = format_mod_loaded_event_json(&record);
    let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
    assert_eq!(v["event"], "mod.loaded.v1");
    assert_eq!(v["mod_id"], "json-mod");
    assert_eq!(v["tick"], 99);
}

/// Covers FR-CIV-TEST-005: format_mod_unloaded_event_json has required keys.
#[test]
fn test_format_mod_unloaded_event_json() {
    let record = ModUnloadedRecord {
        mod_id: "old-mod".to_owned(),
        mod_name: "Old Mod".to_owned(),
        tick: 42,
        reason: "user_request".to_owned(),
    };
    let json = format_mod_unloaded_event_json(&record);
    let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
    assert_eq!(v["event"], "mod.unloaded.v1");
    assert_eq!(v["reason"], "user_request");
    assert_eq!(v["tick"], 42);
}

/// Covers FR-CIV-TEST-005: format_mod_error_event produces text form.
#[test]
fn test_format_mod_error_event_text() {
    let line = format_mod_error_event("broken-mod", 7, "trap at 0x0");
    assert!(line.contains("mod.error.v1"), "got: {line}");
    assert!(line.contains("broken-mod"), "got: {line}");
    assert!(line.contains("tick=7"), "got: {line}");
}

/// Covers FR-CIV-TEST-005: format_mod_error_event_json is valid JSON.
#[test]
fn test_format_mod_error_event_json() {
    let json = format_mod_error_event_json("broken-mod", 7, "trap at 0x0");
    let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
    assert_eq!(v["event"], "mod.error.v1");
    assert_eq!(v["mod_id"], "broken-mod");
    assert_eq!(v["tick"], 7);
    assert_eq!(v["message"], "trap at 0x0");
}

// ── ModHost: error paths ──────────────────────────────────────────────────────

/// Covers FR-CIV-TEST-005: load_mod_path with nonexistent path returns Err.
#[test]
fn test_mod_host_load_nonexistent_path_returns_err() {
    let mut host = ModHost::new();
    let result = host.load_mod_path("/nonexistent/path/to/nowhere");
    assert!(result.is_err(), "expected Err for nonexistent path");
}

/// Covers FR-CIV-TEST-005: unload_mod with unknown id returns Err.
#[test]
fn test_mod_host_unload_unknown_id_returns_err() {
    let mut host = ModHost::new();
    let result = host.unload_mod("does-not-exist", "test", 0);
    assert!(result.is_err(), "expected Err for unknown mod id");
}

/// Covers FR-CIV-TEST-005: new ModHost has no mods.
#[test]
fn test_mod_host_new_is_empty() {
    let host = ModHost::new();
    assert!(host.mods().is_empty());
    assert!(host.loaded_records().is_empty());
    assert!(host.browser_entries().is_empty());
}

/// Covers FR-CIV-TEST-005: guest_memory_snapshot returns empty for unknown mod.
#[test]
fn test_mod_host_guest_memory_snapshot_unknown_returns_empty() {
    let host = ModHost::new();
    assert!(host.guest_memory_snapshot("unknown").is_empty());
}

// ── Perf budget helpers ───────────────────────────────────────────────────────

#[test]
fn test_phases_over_budget_empty() {
    let result = phases_over_budget(&[], 16_000u64);
    assert!(result.is_empty());
}

#[test]
fn test_tick_over_budget_within() {
    let mut p = TickProfile::default();
    p.record("phase_a", 8_000u64);
    assert!(!tick_over_budget(&p, 16_000u64));
}

#[test]
fn test_tick_over_budget_exceeded() {
    let mut p = TickProfile::default();
    p.record("phase_a", 20_000u64);
    assert!(tick_over_budget(&p, 16_000u64));
}