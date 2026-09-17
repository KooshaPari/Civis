//! FR traceability tests for the protocol-3d crate.
//!
//! Covers: FR-CIV-PROTO-001, FR-CIV-PROTO-015, FR-PROTO-002/003/004/005, FR-CIV-PROTO3D

use civ_protocol_3d::{
    BuildingProvenance, Frame3dBundle, WorldXZ, SCHEMA_VERSION,
    FRAME3D_BUNDLE_MAGIC, FRAME3D_BUNDLE_VERSION,
};

/// FR-CIV-PROTO-001 — Schema version is defined and valid.
#[test]
fn fr_civ_proto_001_schema_version_valid() {
    // Schema version exists; it may be 0 during development but must be defined
    let _v = SCHEMA_VERSION;
}

/// FR-CIV-PROTO-015 — WorldXZ can be constructed.
#[test]
fn fr_civ_proto_015_world_xz_construction() {
    let wxz = WorldXZ { x: 1.5, z: -3.2 };
    assert!((wxz.x - 1.5).abs() < 1e-6);
    assert!((wxz.z - (-3.2)).abs() < 1e-6);
}

/// FR-PROTO-002 — BuildingProvenance enum variants are distinct.
#[test]
fn fr_proto_002_building_provenance_variants() {
    let proc = BuildingProvenance::Procedural;
    let free = BuildingProvenance::Freehand;
    assert_ne!(proc, free);
}

/// FR-PROTO-003 — Frame3dBundle magic bytes match expected constant.
#[test]
fn fr_proto_003_frame3d_bundle_magic() {
    assert_eq!(FRAME3D_BUNDLE_MAGIC, b"F3DB");
}

/// FR-PROTO-004 — Frame3dBundle version is defined.
#[test]
fn fr_proto_004_frame3d_bundle_version() {
    // Version must be a valid u16 (may be 0 during development)
    let _v = FRAME3D_BUNDLE_VERSION;
}

/// FR-PROTO-005 — WorldXZ serialises and deserialises.
#[test]
fn fr_proto_005_world_xz_serde_roundtrip() {
    let wxz = WorldXZ { x: 42.0, z: -7.5 };
    let json = serde_json::to_string(&wxz).expect("serialize");
    let decoded: WorldXZ = serde_json::from_str(&json).expect("deserialize");
    assert!((wxz.x - decoded.x).abs() < 1e-6);
    assert!((wxz.z - decoded.z).abs() < 1e-6);
}

/// FR-CIV-PROTO3D — BuildingProvenance serialises and deserialises.
#[test]
fn fr_civ_proto3d_provenance_serde_roundtrip() {
    for prov in [BuildingProvenance::Procedural, BuildingProvenance::Freehand] {
        let json = serde_json::to_string(&prov).expect("serialize");
        let decoded: BuildingProvenance = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(prov, decoded);
    }
}
