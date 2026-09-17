//! FR traceability tests for engine hash chain, tick profiling, and LOD.
//!
//! Covers: FR-CORE-005/006/007, FR-LOD-001/002/004,
//! NFR-CIV-DET-001/002, NFR-CIV-PERF-001/002

use civ_engine::hash_chain::{
    chain_advance, chain_root_from_ticks, tick_event_bytes, tick_hash, HashChainState, GENESIS,
    HASH_LEN,
};
use civ_engine::lod::{aggregate_strategic, should_tick_entity, HexCellSnapshot, LodPolicy};
use civ_engine::perf::{phases_over_budget, tick_over_budget, TickProfile};
use civ_agents::LodTier;

// ===========================================================================
// FR-CORE-005 / FR-CORE-006 — Hash chain determinism
// ===========================================================================

/// FR-CORE-005 — Genesis is all zeros.
#[test]
fn fr_core_005_genesis_is_zero() {
    assert_eq!(GENESIS, [0u8; HASH_LEN]);
}

/// FR-CORE-005 — tick_event_bytes produces little-endian encoding.
#[test]
fn fr_core_005_tick_event_bytes_le() {
    let bytes = tick_event_bytes(1);
    assert_eq!(bytes, [1, 0, 0, 0, 0, 0, 0, 0]);
}

/// FR-CORE-006 — Same input produces same hash (determinism).
#[test]
fn fr_core_006_tick_hash_deterministic() {
    let a = tick_hash(&GENESIS, &tick_event_bytes(0));
    let b = tick_hash(&GENESIS, &tick_event_bytes(0));
    assert_eq!(a, b);
}

/// FR-CORE-006 — Different inputs produce different hashes.
#[test]
fn fr_core_006_different_inputs_different_hash() {
    let a = tick_hash(&GENESIS, &tick_event_bytes(0));
    let b = tick_hash(&GENESIS, &tick_event_bytes(1));
    assert_ne!(a, b);
}

/// FR-CORE-006 — Hash chain advances correctly.
#[test]
fn fr_core_006_hash_chain_advance() {
    let mut state = HashChainState::new();
    let h1 = state.advance(&tick_event_bytes(0));
    let h2 = state.advance(&tick_event_bytes(1));
    assert_ne!(h1, h2);
    assert_ne!(h1, GENESIS);
}

/// FR-CORE-006 — chain_root_from_ticks returns None for empty input.
#[test]
fn fr_core_006_chain_root_empty() {
    let result = chain_root_from_ticks(std::iter::empty::<u64>());
    assert!(result.is_none());
}

/// FR-CORE-006 — chain_root_from_ticks matches manual chain advance.
#[test]
fn fr_core_006_chain_root_matches_manual() {
    let ticks = [0u64, 1, 2, 3];
    let root = chain_root_from_ticks(ticks.iter().copied()).unwrap();
    let manual = chain_advance(
        &chain_advance(
            &chain_advance(&chain_advance(&GENESIS, &tick_event_bytes(0)), &tick_event_bytes(1)),
            &tick_event_bytes(2),
        ),
        &tick_event_bytes(3),
    );
    assert_eq!(root, manual);
}

// ===========================================================================
// NFR-CIV-DET-001 — Determinism across hash chain
// ===========================================================================

/// NFR-CIV-DET-001 — Two identical chains produce the same root.
#[test]
fn fr_nfr_civ_det_001_identical_chains_same_root() {
    let a = chain_root_from_ticks(0..100).unwrap();
    let b = chain_root_from_ticks(0..100).unwrap();
    assert_eq!(a, b);
}

// ===========================================================================
// FR-CORE-007 / NFR-CIV-PERF-001 — Tick profiling
// ===========================================================================

/// FR-CORE-007 — TickProfile accumulates phases correctly.
#[test]
fn fr_core_007_tick_profile_accumulates() {
    let mut p = TickProfile::default();
    p.record("physics", 100);
    p.record("render", 500);
    p.record("audio", 50);
    assert_eq!(p.total_micros, 650);
    assert_eq!(p.phases.len(), 3);
}

/// FR-CORE-007 — slowest phase is identified correctly.
#[test]
fn fr_core_007_slowest_phase() {
    let mut p = TickProfile::default();
    p.record("physics", 100);
    p.record("render", 500);
    p.record("audio", 50);
    let slowest = p.slowest().unwrap();
    assert_eq!(slowest.0, "render");
    assert_eq!(slowest.1, 500);
}

/// NFR-CIV-PERF-001 — phases_over_budget filters correctly.
#[test]
fn fr_nfr_civ_perf_001_phases_over_budget() {
    let timings = [("a", 100), ("b", 500), ("c", 50)];
    let over = phases_over_budget(&timings, 200);
    assert_eq!(over.len(), 1);
    assert_eq!(over[0].0, "b");
}

/// NFR-CIV-PERF-002 — tick_over_budget returns true when total exceeds budget.
#[test]
fn fr_nfr_civ_perf_002_tick_over_budget() {
    let mut p = TickProfile::default();
    p.record("a", 100);
    p.record("b", 200);
    assert!(!tick_over_budget(&p, 500));
    assert!(tick_over_budget(&p, 250));
}

// ===========================================================================
// FR-LOD-001/002/004 — Level of Detail
// ===========================================================================

/// FR-LOD-001 — Hot entities tick every tick.
#[test]
fn fr_lod_001_hot_always_ticks() {
    for tick in 0..100 {
        assert!(should_tick_entity(tick, LodTier::Hot), "Hot should tick at tick {tick}");
    }
}

/// FR-LOD-002 — Warm entities tick on cadence boundary.
#[test]
fn fr_lod_002_warm_ticks_on_cadence() {
    let policy = LodPolicy::default();
    for tick in 0..64 {
        let should = tick % policy.warm_cadence == 0;
        assert_eq!(
            should_tick_entity(tick, LodTier::Warm),
            should,
            "Warm tick mismatch at tick {tick}"
        );
    }
}

/// FR-LOD-002 — aggregate_strategic sums district populations.
#[test]
fn fr_lod_002_aggregate_strategic() {
    let result = aggregate_strategic(&[10, 20, 30]);
    assert_eq!(result, 60);
}

/// FR-LOD-004 — HexCellSnapshot construction.
#[test]
fn fr_lod_004_hex_cell_snapshot() {
    let hex = HexCellSnapshot {
        population: 500,
        resources: 1000,
    };
    assert_eq!(hex.population, 500);
    assert_eq!(hex.resources, 1000);
}
