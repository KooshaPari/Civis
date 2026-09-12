//! Focused persistence/replay regression tests for Civis engine.

use civ_engine::replay::{ReplayLog, ReplayEvent};
use civ_engine::replay_format::{decode_civreplay, encode_civreplay};
use civ_engine::hash_chain::{chain_advance, GENESIS};
use civ_engine::Simulation;
use civ_save_db::format_session_saved_event_json;
use civ_mod_host::format_mod_error_event_json;
use civ_agents::Civilian;

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

/// Durable archive roundtrip preserves the **combined actor + economy + hash-chain** surface.
///
/// This is the contract load-on-launch depends on. Previous coverage verified
/// these surfaces individually (see `civsave_archive_round_trip_persists_state_surface`
/// in save.rs and `actor_y_persists_across_replay` in save.rs); this test asserts
/// they all survive together in one archive roundtrip.
#[test]
fn durable_archive_preserves_actor_economy_hash_chain_combined() {
    let mut sim = Simulation::with_seed(101);
    for _ in 0..6 {
        sim.tick();
    }

    // Capture the combined surface before save.
    let tick_before = sim.state.tick;
    let pop_before = sim.state.population;
    let treasury_before = sim.state.faction_treasury.clone();
    let resources_before = sim.state.faction_resources.clone();
    let hash_before = sim.hash_chain_root().expect("hash chain must be set after ticks");
    let civilian_count_before: usize =
        sim.world.query::<&Civilian>().iter().count();

    assert!(civilian_count_before > 0, "seeded sim must have civilians");

    // Persist via the durable archive (`.civsave.zst`) and reload.
    let dir = tempfile::tempdir().expect("tempdir");
    let archive_path = dir.path().join("actor-economy-chain.civsave.zst");
    civ_engine::CivSaveBundle::save_archive(&archive_path, &sim).expect("save archive");
    let loaded = civ_engine::CivSaveBundle::load_archive(&archive_path)
        .expect("load archive");

    // Economy + tick surface.
    assert_eq!(loaded.state.tick, tick_before, "tick must survive");
    assert_eq!(loaded.state.population, pop_before, "population must survive");
    assert_eq!(
        loaded.state.faction_treasury, treasury_before,
        "faction treasury must survive"
    );
    assert_eq!(
        loaded.state.faction_resources, resources_before,
        "faction resources must survive"
    );

    // Actor surface.
    let civilian_count_after: usize =
        loaded.world.query::<&Civilian>().iter().count();
    assert_eq!(
        civilian_count_after, civilian_count_before,
        "civilian entity count must survive archive roundtrip"
    );

    // Hash chain root preserved (replay log persisted exactly).
    assert_eq!(
        loaded.hash_chain_root().expect("loaded must have hash chain"),
        hash_before,
        "hash-chain root must survive archive roundtrip"
    );
}

/// Full-scenario replay: record ticks, save via durable archive, reload, then
/// continue ticking from the loaded state. The continuation must be byte-identical
/// to a sim that ran straight through without an archive boundary in the middle.
#[test]
fn full_scenario_replay_across_archive_boundary_is_deterministic() {
    // Scenario A: continuous run (no archive boundary).
    let mut continuous = Simulation::with_seed(202);
    for _ in 0..4 {
        continuous.tick();
    }
    let archive_at_tick = continuous.state.tick;

    // Scenario B: run, archive, reload, run remaining ticks.
    let mut staged = Simulation::with_seed(202);
    for _ in 0..4 {
        staged.tick();
    }
    assert_eq!(staged.state.tick, archive_at_tick);

    let dir = tempfile::tempdir().expect("tempdir");
    let archive_path = dir.path().join("scenario.civsave.zst");
    civ_engine::CivSaveBundle::save_archive(&archive_path, &staged)
        .expect("save archive");
    let mut loaded = civ_engine::CivSaveBundle::load_archive(&archive_path)
        .expect("load archive");

    // Run the same number of additional ticks on both.
    for _ in 0..5 {
        continuous.tick();
        loaded.tick();
    }

    // Both paths must produce identical final state (determinism across the archive).
    assert_eq!(
        loaded.state.tick, continuous.state.tick,
        "tick after archive continuation must match continuous run"
    );
    assert_eq!(
        loaded.state.faction_treasury, continuous.state.faction_treasury,
        "faction treasury must match across archive boundary"
    );
    assert_eq!(
        loaded.state.faction_resources, continuous.state.faction_resources,
        "faction resources must match across archive boundary"
    );
    assert_eq!(
        loaded.hash_chain_root(), continuous.hash_chain_root(),
        "hash-chain root must match across archive boundary"
    );

    // Actor counts must also match (no actors lost or duplicated at the boundary).
    let continuous_civs: usize = continuous.world.query::<&Civilian>().iter().count();
    let loaded_civs: usize = loaded.world.query::<&Civilian>().iter().count();
    assert_eq!(
        loaded_civs, continuous_civs,
        "civilian count must match across archive boundary"
    );
}

/// Durable save: load-on-launch via `CivSaveBundle::load` (auto-detects folder vs archive)
/// must produce the same hash chain root as the original sim, proving load-on-launch
/// is bit-faithful for replay integrity checks.
#[test]
fn load_on_launch_via_civsave_bundle_preserves_hash_chain_root() {
    let mut sim = Simulation::with_seed(303);
    for _ in 0..3 {
        sim.tick();
    }
    let original_root = sim.hash_chain_root().expect("hash chain must exist");

    let dir = tempfile::tempdir().expect("tempdir");
    let archive_path = dir.path().join("launch.civsave.zst");
    civ_engine::CivSaveBundle::save_archive(&archive_path, &sim)
        .expect("save archive");

    // Simulate the load-on-launch path (single dispatch entry point).
    let loaded =
        civ_engine::CivSaveBundle::load(&archive_path).expect("load on launch");

    assert_eq!(
        loaded.hash_chain_root().expect("loaded has hash chain"),
        original_root,
        "load-on-launch must preserve hash chain root bit-exactly"
    );
    assert_eq!(
        loaded.state.tick, sim.state.tick,
        "load-on-launch must preserve tick"
    );
}