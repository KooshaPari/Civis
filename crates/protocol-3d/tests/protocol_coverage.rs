//! External integration tests for protocol-3d wire types (FR-CIV-TEST-009).
//!
//! Covers gaps not exercised by inline unit tests:
//! - CivilianStateEntry serde round-trip + legacy health default
//! - BuildingDiffFrame serde with skip_serializing_if on empty buildings
//! - quantize_axis / dequantize_axis round-trip at non-zero origin
//! - is_frame3d_bundle vs is_frame3d_binary magic boundary
use civ_protocol_3d::{
    dequantize_axis, is_frame3d_binary, is_frame3d_bundle, quantize_axis, BuildingDiffFrame,
    BuildingProvenance, CivilianNeeds3d, CivilianStateEntry, GenomeSummary3d, SCHEMA_VERSION,
};
use serde_json::json;

// ── CivilianStateEntry serde ─────────────────────────────────────────────────

#[test]
fn civilian_state_entry_round_trips() {
    let entry = CivilianStateEntry {
        id: 7,
        faction_id: 2,
        needs: CivilianNeeds3d { food: 0.9, shelter: 0.8, safety: 0.7, social: 0.6, rest: 0.5 },
        profession: "Farmer".into(),
        genome_summary: GenomeSummary3d::default(),
        species: "human".into(),
        health: 0.75,
    };
    let json = serde_json::to_string(&entry).expect("serialize");
    let back: CivilianStateEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, entry);
}

#[test]
fn civilian_state_entry_legacy_defaults_health_to_one() {
    let legacy: CivilianStateEntry =
        serde_json::from_value(json!({ "id": 1 })).expect("legacy deserialize");
    assert_eq!(legacy.health, 1.0, "default health must be 1.0");
    assert_eq!(legacy.faction_id, 0);
    assert_eq!(legacy.needs, CivilianNeeds3d::default());
    assert_eq!(legacy.genome_summary, GenomeSummary3d::default());
}

// ── BuildingDiffFrame serde ───────────────────────────────────────────────────

#[test]
fn building_diff_frame_empty_buildings_omitted_in_json() {
    let frame = BuildingDiffFrame {
        tick: 42,
        provenance: BuildingProvenance::Procedural,
        buildings: vec![],
        graph: None,
    };
    let json = serde_json::to_string(&frame).expect("serialize");
    assert!(
        !json.contains("\"buildings\""),
        "empty vec must be omitted via skip_serializing_if: {json}"
    );
    let back: BuildingDiffFrame = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.tick, 42);
    assert!(back.buildings.is_empty());
}

// ── quantize / dequantize round-trip ────────────────────────────────────────

#[test]
fn quantize_dequantize_round_trip_non_zero_origin() {
    let origin_cells = 64_i32;
    let world_m = 10.0_f32;
    let cell = quantize_axis(world_m, origin_cells).expect("in range");
    let recovered = dequantize_axis(cell, origin_cells);
    assert!(
        (recovered - world_m).abs() < 0.5,
        "round-trip error too large: got {recovered} for {world_m}"
    );
    assert_eq!(quantize_axis(1_000_000.0, 0), None, "out-of-range must return None");
}

// ── magic boundary: bundle vs binary ────────────────────────────────────────

#[test]
fn frame3d_magic_bytes_are_disjoint() {
    let bundle_hdr = b"F3DB\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
    assert!(is_frame3d_bundle(bundle_hdr), "must detect F3DB magic");
    assert!(!is_frame3d_binary(bundle_hdr), "F3DB must not be mistaken for F3D0");

    assert!(!is_frame3d_bundle(b"nope"));
    assert!(!is_frame3d_binary(b"nope"));
    assert!(!is_frame3d_bundle(&[]));
    assert!(!is_frame3d_binary(&[]));
}

// ── SCHEMA_VERSION stability ─────────────────────────────────────────────────

#[test]
fn schema_version_is_zero() {
    assert_eq!(SCHEMA_VERSION, 0, "bump only on wire-incompatible change");
}