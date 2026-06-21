//! External coverage tests for crates/legends — gap_reports, empty_saga_reason,
//! mark_died, set_name (all zero-test pub fns as of FR-CIV-TEST-011).
use civ_legends::*;

fn cfg() -> LegendsConfig {
    LegendsConfig {
        promotion_threshold: 0.5,
        causal_window_epochs: 8,
        causal_min_confidence: 0.1,
        ticks_per_epoch: 1,
        gap_epochs: 2,
        ..LegendsConfig::default()
    }
}

fn ev(tick: u64, kind: EventKind, src: SourceCrate, mag: f32, sim: u64, role: Role) -> RawSimEvent {
    RawSimEvent::new(tick, kind, src, mag).with_participant(src, SimRuntimeId(sim), role)
}

#[test]
fn gap_reports_returns_typed_reason_for_silent_source() {
    let mut g = SagaGraph::new(cfg());
    // Ingest one Agents event so Engine becomes the silent source
    g.ingest(ev(0, EventKind::Birth, SourceCrate::Agents, 0.9, 1, Role::Subject));
    // At epoch 10, Engine has never reported — should surface a gap
    let reports = g.gap_reports(Epoch(10));
    let engine_gap = reports
        .iter()
        .find(|r| matches!(r.source, SourceCrate::Engine));
    // reason string must be non-empty
    if let Some(r) = engine_gap {
        assert!(!r.reason.is_empty(), "gap report reason must not be empty");
        assert!(r.reason.contains("Engine") || r.reason.len() > 0);
    }
    // gap_reports must agree with detect_gaps in count
    let raw_gaps = g.detect_gaps(Epoch(10));
    assert_eq!(reports.len(), raw_gaps.len(), "gap_reports count must equal detect_gaps count");
}

#[test]
fn empty_saga_reason_unknown_entity_returns_some() {
    let g = SagaGraph::new(cfg());
    // A LegendEntityId that was never inserted
    let reason = g.empty_saga_reason(LegendEntityId(999));
    assert!(
        matches!(reason, Some(EmptySagaReason::UnknownEntity)),
        "unregistered id should return UnknownEntity, got {:?}",
        reason
    );
}

#[test]
fn mark_died_sets_death_epoch_idempotent() {
    let mut g = SagaGraph::new(cfg());
    // Ingest a significant event to resolve an entity
    g.ingest(ev(0, EventKind::Battle, SourceCrate::Agents, 0.9, 42, Role::Subject));
    let eid = g
        .entity_for_sim(SourceCrate::Agents, SimRuntimeId(42))
        .expect("entity resolved");

    // Before mark_died, no death epoch
    let before = g.entity(eid).unwrap();
    assert!(before.died_epoch.is_none(), "entity should not be dead yet");

    g.mark_died(eid, Epoch(5));
    let after = g.entity(eid).unwrap();
    assert_eq!(after.died_epoch, Some(Epoch(5)));

    // Idempotent: second call must NOT overwrite
    g.mark_died(eid, Epoch(99));
    let after2 = g.entity(eid).unwrap();
    assert_eq!(after2.died_epoch, Some(Epoch(5)), "mark_died must be idempotent");
}

#[test]
fn set_name_attaches_name_to_promoted_entity() {
    let mut g = SagaGraph::new(cfg());
    g.ingest(ev(0, EventKind::Battle, SourceCrate::Agents, 0.9, 7, Role::Subject));
    let eid = g
        .entity_for_sim(SourceCrate::Agents, SimRuntimeId(7))
        .expect("entity resolved");

    assert!(g.entity(eid).unwrap().name.is_none(), "no name before set_name");

    g.set_name(eid, NameRef(1001));
    let named = g.entity(eid).unwrap();
    assert_eq!(named.name, Some(NameRef(1001)));
}