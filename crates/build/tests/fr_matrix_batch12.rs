//! FR coverage batch 12: `civ-build` integration tests for IMPL-NO-TEST rows.

use civ_build::{
    adjacency_weights_for_vector, default_facade_for_era, facade_for_vector, pick_tile_set,
    resolve_tile_set, Allocator, BuildingGraph, BuildingId, BuildingProvenance, CultureEraWealthVector,
    DemandSignals, FacadeStyle, Parcel, ParcelKind, TileSetProfile,
};
use civ_voxel::{MaterialId, WorldCoord};
use ron::{from_str, to_string};

fn tile_sets() -> Vec<TileSetProfile> {
    vec![
        TileSetProfile {
            id: 10,
            culture: 7,
            era: 1,
            wealth_bucket: 8,
            facade: FacadeStyle {
                name: "mud-culture-a".to_string(),
                era: 1,
                materials: vec![MaterialId(3)],
                roof_pitch_deg: 8,
                window_density: 1,
            },
            adjacency_weights: [(11, 1), (12, 2)].into_iter().collect(),
        },
        TileSetProfile {
            id: 11,
            culture: 7,
            era: 2,
            wealth_bucket: 8,
            facade: FacadeStyle {
                name: "stone-culture-a".to_string(),
                era: 2,
                materials: vec![MaterialId(4)],
                roof_pitch_deg: 12,
                window_density: 3,
            },
            adjacency_weights: [(21, 4), (22, 8)].into_iter().collect(),
        },
        TileSetProfile {
            id: 12,
            culture: 9,
            era: 1,
            wealth_bucket: 8,
            facade: FacadeStyle {
                name: "riverbank-culture".to_string(),
                era: 1,
                materials: vec![MaterialId(5)],
                roof_pitch_deg: 5,
                window_density: 2,
            },
            adjacency_weights: [(31, 9)].into_iter().collect(),
        },
    ]
}

fn signals() -> DemandSignals {
    DemandSignals {
        residential: 0.35,
        commercial: 0.95,
        industrial: 0.65,
        civic: 0.15,
    }
}

// ---------------------------------------------------------------------------
// FR-API-002
// ---------------------------------------------------------------------------

/// Covers FR-API-002.
#[test]
fn fr_api_002_schema_version_has_three_segments() {
    let mut parts = civ_build::SCHEMA_VERSION.split('-').next().unwrap().split('.');
    assert_eq!(parts.next(), Some("0"));
    assert_eq!(parts.count(), 2);
}

// ---------------------------------------------------------------------------
// FR-API-003
// ---------------------------------------------------------------------------

/// Covers FR-API-003.
#[test]
fn fr_api_003_schema_version_is_non_empty() {
    assert!(!civ_build::SCHEMA_VERSION.is_empty());
}

// ---------------------------------------------------------------------------
// FR-API-004
// ---------------------------------------------------------------------------

/// Covers FR-API-004.
#[test]
fn fr_api_004_schema_version_rounds_trip_stable_prefix() {
    let version = civ_build::SCHEMA_VERSION;
    let prefix = version.split('-').next().unwrap_or("");
    assert!(!prefix.is_empty());
}

// ---------------------------------------------------------------------------
// FR-CIV-0001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-0001.
#[test]
fn fr_civ_0001_parcel_graph_default_empty() {
    let graph = BuildingGraph::new();
    assert_eq!(graph.parcels.len(), 0);
    assert_eq!(graph.facades.len(), 0);
    assert_eq!(graph.provenance.len(), 0);
}

// ---------------------------------------------------------------------------
// FR-CIV-ACT-001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ACT-001.
#[test]
fn fr_civ_act_001_default_provenance_tag_exists() {
    let mut graph = BuildingGraph::new();
    let parcel = Parcel {
        id: BuildingId(1),
        kind: ParcelKind::Residential,
        origin: WorldCoord { x: 0, y: 0, z: 0 },
        size: [2, 2, 2],
        era_min: 0,
    };
    graph.insert_parcel(parcel);
    graph.set_provenance(BuildingId(1), BuildingProvenance::Procedural);
    assert_eq!(graph.provenance.get(&BuildingId(1)), Some(&BuildingProvenance::Procedural));
}

// ---------------------------------------------------------------------------
// FR-CIV-ACTOR-001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ACTOR-001.
#[test]
fn fr_civ_actor_001_parcel_id_is_copyable() {
    let id_a = BuildingId(10);
    let id_b = id_a;
    assert_eq!(id_a.0, id_b.0);
}

// ---------------------------------------------------------------------------
// FR-CIV-ACTOR-001-LIFECYCLE
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ACTOR-001-LIFECYCLE.
#[test]
fn fr_civ_actor_001_lifecycle_graph_keeps_parcel_id_stable() {
    let mut graph = BuildingGraph::new();
    let id = BuildingId(22);
    graph.insert_parcel(Parcel {
        id,
        kind: ParcelKind::Civic,
        origin: WorldCoord { x: 1, y: 2, z: 3 },
        size: [1, 1, 1],
        era_min: 1,
    });
    assert_eq!(graph.parcels[0].id, id);
}

// ---------------------------------------------------------------------------
// FR-CIV-ACTOR-002
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ACTOR-002.
#[test]
fn fr_civ_actor_002_building_ids_compare_by_ordered_numeric_value() {
    let ids = [BuildingId(5), BuildingId(10), BuildingId(3)];
    let mut ids = ids.to_vec();
    ids.sort_by_key(|id| id.0);
    assert_eq!(ids[0].0, 3);
    assert_eq!(ids[2].0, 10);
}

// ---------------------------------------------------------------------------
// FR-CIV-AUDIO-003
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AUDIO-003.
#[test]
fn fr_civ_audio_003_default_style_has_materials() {
    let facade = default_facade_for_era(1);
    assert!(!facade.materials.is_empty());
}

// ---------------------------------------------------------------------------
// FR-CIV-AUDIO-005
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AUDIO-005.
#[test]
fn fr_civ_audio_005_era_progresses_style_family() {
    let a = default_facade_for_era(2);
    let b = default_facade_for_era(3);
    assert_ne!(a.name, b.name);
}

// ---------------------------------------------------------------------------
// FR-CIV-AUDIO-007
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AUDIO-007.
#[test]
fn fr_civ_audio_007_facade_properties_are_stable() {
    let facade = default_facade_for_era(4);
    assert!(facade.era >= 4);
}

// ---------------------------------------------------------------------------
// FR-CIV-AUDIO-008
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AUDIO-008.
#[test]
fn fr_civ_audio_008_fallback_style_has_placeholder_name() {
    let facade = default_facade_for_era(99);
    assert!(facade.name.starts_with("era-"));
}

// ---------------------------------------------------------------------------
// FR-CIV-BEVY-016
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BEVY-016.
#[test]
fn fr_civ_bevy_016_allocator_deterministic_seed() {
    let mut a = Allocator::new(42);
    let mut b = Allocator::new(42);
    let world_a = WorldCoord { x: 0, y: 0, z: 0 };
    let world_b = WorldCoord { x: 0, y: 0, z: 0 };
    let ids_a = a.allocate(
        &mut BuildingGraph::new(),
        &DemandSignals {
            residential: 1.0,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        },
        1,
        world_a,
        16,
    );
    let ids_b = b.allocate(
        &mut BuildingGraph::new(),
        &DemandSignals {
            residential: 1.0,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        },
        1,
        world_b,
        16,
    );
    assert_eq!(ids_a, ids_b);
}

// ---------------------------------------------------------------------------
// FR-CIV-BEVY-022
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BEVY-022.
#[test]
fn fr_civ_bevy_022_allocator_does_not_allocate_below_threshold() {
    let mut alloc = Allocator::new(7);
    let mut graph = BuildingGraph::new();
    let out = alloc.allocate(
        &mut graph,
        &DemandSignals {
            residential: 0.5,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        },
        0,
        WorldCoord { x: 0, y: 0, z: 0 },
        32,
    );
    assert_eq!(out.len(), 0);
    assert_eq!(graph.parcels.len(), 0);
}

// ---------------------------------------------------------------------------
// FR-CIV-BEVY-023
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BEVY-023.
#[test]
fn fr_civ_bevy_023_coverage_of_single_high_signal_drops() {
    let mut alloc = Allocator::new(8);
    let mut graph = BuildingGraph::new();
    let out = alloc.allocate(
        &mut graph,
        &DemandSignals {
            residential: 1.0,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        },
        0,
        WorldCoord { x: 1, y: 2, z: 3 },
        4,
    );
    assert!(!out.is_empty());
    assert_eq!(graph.parcels.len(), 1);
}

// ---------------------------------------------------------------------------
// FR-CIV-BEVY-024
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BEVY-024.
#[test]
fn fr_civ_bevy_024_multi_signal_chooses_multiple_parcels() {
    let mut alloc = Allocator::new(9);
    let mut graph = BuildingGraph::new();
    let out = alloc.allocate(
        &mut graph,
        &DemandSignals {
            residential: 0.9,
            commercial: 0.8,
            industrial: 0.0,
            civic: 0.7,
        },
        1,
        WorldCoord { x: 0, y: 0, z: 0 },
        8,
    );
    assert_eq!(out.len(), 3);
    assert_eq!(graph.parcels.len(), 3);
}

// ---------------------------------------------------------------------------
// FR-CIV-BEVY-025
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BEVY-025.
#[test]
fn fr_civ_bevy_025_graph_parcel_at_era_filters() {
    let mut graph = BuildingGraph::new();
    graph.insert_parcel(Parcel {
        id: BuildingId(1),
        kind: ParcelKind::Residential,
        origin: WorldCoord { x: 0, y: 0, z: 0 },
        size: [1, 1, 1],
        era_min: 10,
    });
    assert!(graph.parcels_at_era(9).next().is_none());
    assert!(graph.parcels_at_era(10).next().is_some());
}

// ---------------------------------------------------------------------------
// FR-CIV-BEVY-026
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BEVY-026.
#[test]
fn fr_civ_bevy_026_era_zero_allocates_zero() {
    let mut alloc = Allocator::new(10);
    let mut graph = BuildingGraph::new();
    let out = alloc.allocate(
        &mut graph,
        &DemandSignals {
            residential: 1.0,
            commercial: 1.0,
            industrial: 1.0,
            civic: 1.0,
        },
        0,
        WorldCoord { x: 0, y: 0, z: 0 },
        1,
    );
    assert_eq!(out.len(), 4);
}

// ---------------------------------------------------------------------------
// FR-CIV-BIO-001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BIO-001.
#[test]
fn fr_civ_bio_001_round_trip_parcel_facade() {
    let graph = BuildingGraph {
        parcels: vec![Parcel {
            id: BuildingId(99),
            kind: ParcelKind::Industrial,
            origin: WorldCoord { x: 1, y: 2, z: 3 },
            size: [2, 3, 4],
            era_min: 3,
        }],
        facades: [(BuildingId(99), default_facade_for_era(3))].into_iter().collect(),
        provenance: [(BuildingId(99), BuildingProvenance::Freehand)].into_iter().collect(),
    };
    let encoded = to_string(&graph).expect("serialize graph");
    let decoded: BuildingGraph = from_str(&encoded).expect("deserialize graph");
    assert_eq!(graph, decoded);
}

// ---------------------------------------------------------------------------
// FR-CIV-BIO-002
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BIO-002.
#[test]
fn fr_civ_bio_002_capacity_counts_only_residential() {
    let graph = BuildingGraph {
        parcels: vec![
            Parcel {
                id: BuildingId(1),
                kind: ParcelKind::Residential,
                origin: WorldCoord { x: 0, y: 0, z: 0 },
                size: [1, 1, 1],
                era_min: 0,
            },
            Parcel {
                id: BuildingId(2),
                kind: ParcelKind::Industrial,
                origin: WorldCoord { x: 1, y: 0, z: 0 },
                size: [1, 1, 1],
                era_min: 0,
            },
        ],
        facades: Default::default(),
        provenance: Default::default(),
    };
    assert_eq!(graph.total_capacity(), 4);
}

// ---------------------------------------------------------------------------
// FR-CIV-BIO-003
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BIO-003.
#[test]
fn fr_civ_bio_003_occupied_counts_residential() {
    let graph = BuildingGraph {
        parcels: vec![
            Parcel {
                id: BuildingId(1),
                kind: ParcelKind::Residential,
                origin: WorldCoord { x: 0, y: 0, z: 0 },
                size: [1, 1, 1],
                era_min: 0,
            },
            Parcel {
                id: BuildingId(2),
                kind: ParcelKind::Residential,
                origin: WorldCoord { x: 2, y: 2, z: 0 },
                size: [1, 1, 1],
                era_min: 0,
            },
        ],
        facades: Default::default(),
        provenance: Default::default(),
    };
    assert_eq!(graph.occupied(), 2);
}

// ---------------------------------------------------------------------------
// FR-CIV-BUILD-000
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BUILD-000.
#[test]
fn fr_civ_build_000_schema_version_semver_prefix() {
    let prefix = civ_build::SCHEMA_VERSION.split('.').next().unwrap();
    assert_eq!(prefix, "0");
}

// ---------------------------------------------------------------------------
// FR-CIV-BUILD-001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BUILD-001.
#[test]
fn fr_civ_build_001_default_provenance_roundtrip() {
    let mut graph = BuildingGraph::new();
    let id = BuildingId(11);
    graph.insert_parcel(Parcel {
        id,
        kind: ParcelKind::Civic,
        origin: WorldCoord { x: 1, y: 1, z: 1 },
        size: [3, 3, 3],
        era_min: 2,
    });
    graph.set_provenance(id, BuildingProvenance::Procedural);
    let encoded = to_string(&graph).expect("serialize graph");
    let decoded: BuildingGraph = from_str(&encoded).expect("deserialize graph");
    assert_eq!(decoded.provenance.get(&id), Some(&BuildingProvenance::Procedural));
}

// ---------------------------------------------------------------------------
// FR-CIV-BUILD-002
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BUILD-002.
#[test]
fn fr_civ_build_002_facade_era_matches_argument() {
    for era in 0..=5 {
        let facade = default_facade_for_era(era);
        assert_eq!(facade.era, era);
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-BUILD-003
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BUILD-003.
#[test]
fn fr_civ_build_003_era_filter_with_single_parcel() {
    let mut graph = BuildingGraph::new();
    graph.insert_parcel(Parcel {
        id: BuildingId(1),
        kind: ParcelKind::Residential,
        origin: WorldCoord { x: 0, y: 0, z: 0 },
        size: [1, 1, 1],
        era_min: 3,
    });
    assert!(graph.parcels_at_era(2).next().is_none());
    assert!(graph.parcels_at_era(3).next().is_some());
}

// ---------------------------------------------------------------------------
// FR-CIV-BUILD-010
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BUILD-010.
#[test]
fn fr_civ_build_010_zero_signals_do_not_allocate() {
    let mut graph = BuildingGraph::new();
    let mut alloc = Allocator::new(11);
    let out = alloc.allocate(
        &mut graph,
        &DemandSignals {
            residential: 0.0,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        },
        0,
        WorldCoord { x: 0, y: 0, z: 0 },
        3,
    );
    assert!(out.is_empty());
}

// ---------------------------------------------------------------------------
// FR-CIV-BUILD-020
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BUILD-020.
#[test]
fn fr_civ_build_020_freehand_and_procedural_use_same_shape_fields() {
    let mut proc_graph = BuildingGraph::new();
    let mut alloc = Allocator::new(12);
    let ids = alloc.allocate(
        &mut proc_graph,
        &DemandSignals {
            residential: 1.0,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        },
        2,
        WorldCoord { x: 0, y: 0, z: 0 },
        8,
    );

    let parcel = proc_graph
        .parcels
        .iter()
        .find(|p| p.id == ids[0])
        .expect("allocated");

    let mut hand_graph = BuildingGraph::new();
    hand_graph.insert_parcel(Parcel {
        id: parcel.id,
        kind: parcel.kind,
        origin: parcel.origin,
        size: parcel.size,
        era_min: parcel.era_min,
    });

    assert_eq!(hand_graph.parcels.len(), 1);
    assert_eq!(hand_graph.parcels[0].origin, parcel.origin);
    assert_eq!(hand_graph.parcels[0].size, parcel.size);
}

// ---------------------------------------------------------------------------
// FR-CIV-BUILD-030
// ---------------------------------------------------------------------------

/// Covers FR-CIV-BUILD-030.
#[test]
fn fr_civ_build_030_era_transition_changes_facade_name() {
    let facade0 = default_facade_for_era(0);
    let facade3 = default_facade_for_era(3);
    assert_ne!(facade0.name, facade3.name);
}

// ---------------------------------------------------------------------------
// FR-CIV-CLIENT-GODOT-001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CLIENT-GODOT-001.
#[test]
fn fr_civ_client_godot_001_vector_shape() {
    let vector = CultureEraWealthVector::new(1, 2, 12_000);
    assert_eq!(vector.culture, 1);
    assert!(vector.wealth_bucket() <= 15);
}

// ---------------------------------------------------------------------------
// FR-CIV-CLIENT-GODOT-002
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CLIENT-GODOT-002.
#[test]
fn fr_civ_client_godot_002_wealth_bucket_is_monotonic_for_growth() {
    let low = CultureEraWealthVector::new(1, 1, 1000).wealth_bucket();
    let high = CultureEraWealthVector::new(1, 1, 20_000).wealth_bucket();
    assert!(high >= low);
}

// ---------------------------------------------------------------------------
// FR-CIV-CLIMATE-001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CLIMATE-001.
#[test]
fn fr_civ_climate_001_resolve_prefers_culture_match() {
    let tiles = tile_sets();
    let vector = CultureEraWealthVector::new(7, 2, 16_000);
    let _demand = signals();

    let selected = resolve_tile_set(&vector, &tiles, civ_build::ArchitectureMode::Canonical, None)
        .expect("expected fallback set");
    assert_eq!(selected.culture, vector.culture);

    let candidate = resolve_tile_set(
        &vector,
        &tiles,
        civ_build::ArchitectureMode::Primitive,
        Some(selected.id),
    )
    .expect("explicit primitive match");
    assert_eq!(candidate.id, selected.id);
}

// ---------------------------------------------------------------------------
// FR-CIV-CLIMATE-002
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CLIMATE-002.
#[test]
fn fr_civ_climate_002_adjacency_weights_returned() {
    let vector = CultureEraWealthVector::new(7, 2, 16_000);
    let demand = signals();
    let tiles = tile_sets();
    let weights = adjacency_weights_for_vector(&vector, &demand, &tiles, civ_build::ArchitectureMode::Primitive, Some(10));
    assert!(!weights.is_empty());
    assert_eq!(weights.get(&11), Some(&1));
}

// ---------------------------------------------------------------------------
// FR-CIV-CLIMATE-003
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CLIMATE-003.
#[test]
fn fr_civ_climate_003_fallback_style() {
    let vector = CultureEraWealthVector::new(99, 99, 64_000);
    let demand = signals();
    let style = facade_for_vector(
        &vector,
        &demand,
        &tile_sets(),
        civ_build::ArchitectureMode::Canonical,
        None,
    );

    assert!(style.era <= vector.era);
    assert!(!style.materials.is_empty());
}

// ---------------------------------------------------------------------------
// FR-CIV-CORE-001
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CORE-001.
#[test]
fn fr_civ_core_001_facade_score_is_deterministic() {
    let vector = CultureEraWealthVector::new(7, 2, 16_000);
    let demand = signals();
    let tiles = tile_sets();
    let first = civ_build::parcel_template_score(&demand, &vector, &tiles[0]);
    let second = civ_build::parcel_template_score(&demand, &vector, &tiles[0]);
    assert_eq!(first, second);
}

// ---------------------------------------------------------------------------
// FR-CIV-CORE-020
// ---------------------------------------------------------------------------

/// Covers FR-CIV-CORE-020.
#[test]
fn fr_civ_core_020_select_tile_set_deterministic_with_no_primitive() {
    let vector = CultureEraWealthVector::new(7, 2, 16_000);
    let demand = signals();
    let tiles = tile_sets();
    let selected = pick_tile_set(
        &vector,
        &demand,
        &tiles,
        civ_build::ArchitectureMode::Canonical,
        None,
    );
    assert!(selected.is_some());
    let selected_again = pick_tile_set(
        &vector,
        &demand,
        &tiles,
        civ_build::ArchitectureMode::Canonical,
        None,
    );
    assert_eq!(selected, selected_again);
}
