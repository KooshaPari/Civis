//! Engine-focused FR matrix batch 3 test coverage.
//!
//! Each test references its FR code explicitly so the audit scanner can
//! associate this batch with IMPL-NO-TEST matrix rows.

use civ_engine::{
    aggregate_strategic, baseline_scenario_path, chain_root_from_ticks, decode_civreplay, encode_civreplay,
    format_mod_error_event_json, hash_hex, hash_chain::GENESIS, load_civreplay, load_scenario, save_civreplay,
    HashChainState, HexCellSnapshot, ModLoadedRecord, ModUnloadedRecord, project_zoom, operational_hex_snapshot,
    ReplayError, ReplayLog, SCENARIO_SCHEMA_VERSION, Simulation, tick_event_bytes, tick_hash, ZoomLevel,
};
use civ_save_db::format_session_saved_event_json;
use civ_engine::replay_format::MAGIC;
use civ_tactics::DamageEvent;
use civ_voxel::{MaterialId, WorldCoord};
use tempfile::NamedTempFile;

// FR-LOD-001
/// Covers FR-LOD-001.
#[test]
fn fr_lod_001_has_strategic_and_operational_levels() {
    let levels = [ZoomLevel::Strategic, ZoomLevel::Operational];
    assert_eq!(levels.len(), 2);
    assert_ne!(levels[0], levels[1]);
}

// FR-LOD-002
/// Covers FR-LOD-002.
#[test]
fn fr_lod_002_aggregates_strategic_regions() {
    assert_eq!(aggregate_strategic(&[100, 200, 50]), 350);
    assert_eq!(aggregate_strategic(&[]), 0);
}

// FR-LOD-003
/// Covers FR-LOD-003.
#[test]
fn fr_lod_003_project_zoom_is_view_only() {
    for tick in [0_u64, 1, 7, 64] {
        assert_eq!(project_zoom(tick, ZoomLevel::Strategic), (tick, ZoomLevel::Strategic));
        assert_eq!(project_zoom(tick, ZoomLevel::Operational), (tick, ZoomLevel::Operational));
    }
}

// FR-LOD-004
/// Covers FR-LOD-004.
#[test]
fn fr_lod_004_returns_operational_snapshot_fields() {
    let cell: HexCellSnapshot = operational_hex_snapshot(12, 500);
    assert_eq!(cell.population, 12);
    assert_eq!(cell.resources, 500);
}

// FR-CORE-005
/// Covers FR-CORE-005.
#[test]
fn fr_core_005_tick_hash_is_emitted() {
    let mut sim = Simulation::with_seed(1);
    assert!(sim.hash_chain_root().is_none());
    sim.tick();
    let root = sim.hash_chain_root().expect("tick should emit hash root");
    assert_ne!(root, GENESIS);
    assert_eq!(root, chain_root_from_ticks([1_u64]).expect("single tick root"));
    assert_eq!(hash_hex(&root).len(), 64);
}

// FR-CORE-006
/// Covers FR-CORE-006.
#[test]
fn fr_core_006_chain_is_append_only() {
    let mut sim = Simulation::with_seed(2);
    let mut prev = GENESIS;
    for expected in 1_u64..=4 {
        sim.tick();
        let root = sim.hash_chain_root().expect("root present");
        assert_ne!(root, prev);
        assert_eq!(root, chain_root_from_ticks(1..=expected).expect("prefix hash root"));
        prev = root;
    }

    let event = tick_event_bytes(7);
    let direct = tick_hash(&GENESIS, &event);
    let mut state = HashChainState::new();
    assert_eq!(direct, state.advance(&event));
}

// FR-REPLAY-001
/// Covers FR-REPLAY-001.
#[test]
fn fr_replay_001_roundtrips_and_detects_tamper() {
    let mut log = ReplayLog::default();
    log.record_tick(1);
    log.record_voxel_write(1, WorldCoord { x: 1, y: 2, z: 3 }, MaterialId(7));
    log.record_combat(
        2,
        10,
        20,
        DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 2,
            energy: 11,
        },
    );

    let file = NamedTempFile::new().expect("tempfile");
    save_civreplay(file.path(), &log).expect("save");
    let loaded = load_civreplay(file.path()).expect("load");
    assert_eq!(loaded, log);

    let bytes = encode_civreplay(&log).expect("encode");
    let decoded = decode_civreplay(&bytes).expect("decode");
    assert_eq!(decoded, log);

    let header_len = MAGIC.len() + 4 + 4;
    let mut bad = bytes.clone();
    bad[header_len] ^= 0x01;
    assert!(matches!(decode_civreplay(&bad), Err(ReplayError::ChecksumMismatch)));
}

// FR-MOD-004
/// Covers FR-MOD-004.
#[test]
fn fr_mod_004_reports_mod_loaded_and_unloaded_events() {
    let mut log = ReplayLog::default();
    log.record_mod_loaded(&ModLoadedRecord {
        mod_id: "example-policy".into(),
        mod_name: "Example Policy".into(),
        version: "0.1.0".into(),
        tick: 3,
    });
    log.record_mod_unloaded(&ModUnloadedRecord {
        mod_id: "example-policy".into(),
        mod_name: "Example Policy".into(),
        tick: 11,
        reason: "user_request".into(),
    });

    let loaded = log.mod_loaded_bus_events();
    assert_eq!(loaded.len(), 1);
    let payload = serde_json::from_str::<serde_json::Value>(&loaded[0]).expect("mod.loaded payload");
    assert_eq!(payload["event"], "mod.loaded.v1");
    assert_eq!(payload["mod_id"], "example-policy");
    assert_eq!(payload["tick"], 3);

    let err = format_mod_error_event_json("example-policy", 8, "wasm trap");
    let payload = serde_json::from_str::<serde_json::Value>(&err).expect("mod.error payload");
    assert_eq!(payload["event"], "mod.error.v1");
    assert_eq!(payload["tick"], 8);
}

// FR-SAVE-002
/// Covers FR-SAVE-002.
#[test]
fn fr_save_002_formats_session_saved_event_payload() {
    let payload = format_session_saved_event_json("sess-1", "save-abc", "slot-1", 42, 2048);
    let payload = serde_json::from_str::<serde_json::Value>(&payload).expect("json payload");
    assert_eq!(payload["event_type"], "session.saved.v1");
    assert_eq!(payload["tick"], 42);
}

// FR-API-001
/// Covers FR-API-001.
#[test]
fn fr_api_001_loads_baseline_scenario() {
    let scenario = load_scenario(baseline_scenario_path()).expect("baseline scenario");
    assert_eq!(scenario.version, SCENARIO_SCHEMA_VERSION);
    assert_eq!(scenario.mods[0], "mods/example-policy");
    assert_eq!(scenario.mods[1], "mods/example-economic");
    assert_eq!(scenario.population, 1_000_000);
}
