//! Extended coverage tests for crates/legends (FR-CIV-TEST-013).
//!
//! Targets: epoch_of, kind_weight, kind_affinity, node_count/edge_count,
//!          current_epoch, resolve_aggregate (all zero-test as of this PR).
use civ_legends::*;
use civ_legends::config::{kind_affinity, kind_weight};

fn cfg() -> LegendsConfig {
    LegendsConfig {
        promotion_threshold: 0.5,
        causal_window_epochs: 8,
        causal_min_confidence: 0.1,
        ticks_per_epoch: 10,
        gap_epochs: 2,
        ..LegendsConfig::default()
    }
}

fn ev(tick: u64, kind: EventKind, src: SourceCrate, mag: f32, sim: u64, role: Role) -> RawSimEvent {
    RawSimEvent::new(tick, kind, src, mag).with_participant(src, SimRuntimeId(sim), role)
}

#[test]
fn epoch_of_maps_ticks_to_buckets() {
    let c = cfg(); // ticks_per_epoch = 10
    assert_eq!(c.epoch_of(0), Epoch(0));
    assert_eq!(c.epoch_of(9), Epoch(0));
    assert_eq!(c.epoch_of(10), Epoch(1));
    assert_eq!(c.epoch_of(99), Epoch(9));
    assert_eq!(c.epoch_of(100), Epoch(10));
}

#[test]
fn kind_weight_death_and_war_are_high() {
    // Death/WarDeclared/WarEnded/SpeciationEvent/Extinction should have weight >= 0.9
    for kind in [
        EventKind::Death,
        EventKind::WarDeclared,
        EventKind::WarEnded,
        EventKind::SpeciationEvent,
        EventKind::Extinction,
    ] {
        let w = kind_weight(&kind);
        assert!(w >= 0.9, "{kind:?} weight={w} should be >= 0.9");
    }
    // Sickness should be lower weight
    let sick_w = kind_weight(&EventKind::Sickness);
    assert!(sick_w < 0.9, "Sickness weight={sick_w} should be < 0.9");
}

#[test]
fn kind_affinity_known_pairs_return_nonzero() {
    // Famine -> Migration and Disaster -> Bust are authored high-affinity pairs
    assert_eq!(kind_affinity(&EventKind::Famine, &EventKind::Migration), 1.0);
    assert_eq!(kind_affinity(&EventKind::Disaster, &EventKind::Bust), 1.0);
    // Unrelated pair should return 0.0
    assert_eq!(kind_affinity(&EventKind::Birth, &EventKind::Extinction), 0.0);
}

#[test]
fn node_count_edge_count_and_current_epoch_track_ingests() {
    let mut g = SagaGraph::new(cfg());
    assert_eq!(g.node_count(), 0);
    assert_eq!(g.edge_count(), 0);
    assert_eq!(g.current_epoch(), Epoch(0));

    // Ingest one event — adds 1 entity node + 1 event node + 1 participation edge
    g.ingest(ev(0, EventKind::Birth, SourceCrate::Agents, 0.9, 1, Role::Subject));
    assert_eq!(g.node_count(), 2, "entity + event nodes");
    assert_eq!(g.edge_count(), 1, "one participation edge");

    // Ingest at tick 20 (epoch 2 with ticks_per_epoch=10)
    g.ingest(ev(20, EventKind::Death, SourceCrate::Agents, 0.9, 1, Role::Subject));
    assert!(g.current_epoch() >= Epoch(2), "epoch must advance on high-tick ingest");
}

#[test]
fn resolve_aggregate_is_stable_and_idempotent() {
    let mut g = SagaGraph::new(cfg());
    let key = AggregateKey {
        kind: EntityKind::War,
        a: ClusterId(1),
        b: ClusterId(2),
        start_bucket: 0,
    };
    let id1 = g.resolve_aggregate(key.clone(), Epoch(0));
    let id2 = g.resolve_aggregate(key.clone(), Epoch(1));
    assert_eq!(id1, id2, "same key must resolve to same entity");

    // Different key must resolve to a different entity
    let key2 = AggregateKey {
        kind: EntityKind::War,
        a: ClusterId(3),
        b: ClusterId(4),
        start_bucket: 0,
    };
    let id3 = g.resolve_aggregate(key2, Epoch(0));
    assert_ne!(id1, id3, "different aggregate keys must resolve to different entities");
}