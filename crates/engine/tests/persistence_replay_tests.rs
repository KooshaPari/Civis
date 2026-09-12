//! Focused persistence/replay regression tests for Civis engine.

use civ_engine::replay::{ReplayLog, ReplayEvent};
use civ_engine::replay_format::{decode_civreplay, encode_civreplay};
use civ_engine::hash_chain::{chain_advance, GENESIS};
use civ_save_db::format_session_saved_event_json;
use civ_mod_host::format_mod_error_event_json;

#[test]
fn research_event_extends_hash_chain() {
    let mut log = ReplayLog::default();
    // Before recording research, hash chain is None (GENESIS state)
    assert_eq!(log.running_hash, None);

    let snapshot_hash = vec![0x01, 0x02, 0x03];
    log.record_research(5, snapshot_hash.clone(), true);

    // Hash chain must be set after recording research
    assert!(log.running_hash.is_some(), "Hash chain should be extended after record_research");
    assert_eq!(log.events.len(), 1);
    assert!(matches!(&log.events[0], ReplayEvent::ResearchOutcome { tick, .. } if *tick == 5));
}

#[test]
fn research_hash_chain_matches_manual_advance() {
    let mut log = ReplayLog::default();
    let snapshot_hash = vec![0xAA, 0xBB];

    log.record_research(3, snapshot_hash, true);
    let chain_hash = log.running_hash.unwrap();

    // Manually compute expected hash
    let prev = GENESIS;
    let payload = civ_engine::hash_chain::research_event_bytes(3, &[0xAA, 0xBB], true);
    let expected = chain_advance(&prev, &payload);

    assert_eq!(chain_hash, expected, "Research event hash must match manual computation");
}

#[test]
fn record_mod_permission_violation_valid_json() {
    let mut log = ReplayLog::default();

    // Valid mod permission violation should record
    log.record_mod_permission_violation("test-mod", 42, "some-call", None);
    assert_eq!(log.events.len(), 1);
    assert!(matches!(&log.events[0], ReplayEvent::ModPermissionViolation { tick, .. } if *tick == 42));
}

#[test]
fn mod_permission_violation_rejects_malformed_bus_json() {
    let mut log = ReplayLog::default();

    // Malformed JSON should be rejected, not silently swallowed
    let malformed = r#"{"event":"mod.permission_violation.v1","tick":42,"mod_id":"test"#;
    log.record_mod_permission_violation_bus(42, malformed);

    // Should not record an event with empty/invalid bus_json
    let recorded = log.mod_permission_violation_bus_at_tick(42);
    assert!(recorded.is_empty(), "Malformed JSON should not produce recorded events");
}

#[test]
fn rng_draw_consistency_in_replay() {
    let mut log = ReplayLog::default();

    log.record_rng_draw(10, 0.5, true);
    log.record_rng_draw(10, 0.5, false); // Same tick, different result

    assert_eq!(log.rng_draw_event_count(), 2);
}

#[test]
fn save_load_roundtrip_through_replay() {
    let mut log = ReplayLog::default();

    log.record_tick(1);
    log.record_rng_draw(2, 0.7, true);

    // Encode/decode roundtrip
    let encoded = encode_civreplay(&log).unwrap();
    let decoded = decode_civreplay(&encoded).unwrap();

    assert_eq!(decoded.events.len(), 2);
    assert_eq!(decoded.rng_draw_event_count(), 1);
}

#[test]
fn session_saved_event_json_format() {
    let json = format_session_saved_event_json("sess-1", "save-abc", "slot-1", 42, 2048);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["event_type"], "session.saved.v1");
    assert_eq!(parsed["tick"], 42);
    assert_eq!(parsed["slot"], "slot-1");
    assert_eq!(parsed["byte_size"], 2048);
}

#[test]
fn mod_error_event_json_format() {
    let json = format_mod_error_event_json("example-mod", 8, "wasm trap");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["event"], "mod.error.v1");
    assert_eq!(parsed["tick"], 8);
}