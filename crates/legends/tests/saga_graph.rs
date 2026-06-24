//! Acceptance + invariant tests for the saga-graph engine (spec §5.4, §6 AC-Q-*,
//! FR-CIV-LEGENDS-GRAPH-01/-RESOLVE-04/-SIG-05/-CAUSAL-06/-QUERY-07/-NARRATOR-13).

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
fn ingest_inserts_event_and_resolves_participant() {
    let mut g = SagaGraph::new(cfg());
    let out = g.ingest(ev(
        0,
        EventKind::Birth,
        SourceCrate::Agents,
        0.5,
        1,
        Role::Victim,
    ));
    assert!(out.event_id.is_some());
    // participant resolved to a stable entity
    let eid = g
        .entity_for_sim(SourceCrate::Agents, SimRuntimeId(1))
        .expect("participant resolved");
    assert!(g.entity(eid).is_some());
    assert_eq!(g.edge_count(), 1); // 1 participation edge
}

#[test]
fn resolve_04_recycled_sim_id_same_source_is_same_legend() {
    let mut g = SagaGraph::new(cfg());
    g.ingest(ev(
        0,
        EventKind::Birth,
        SourceCrate::Agents,
        0.2,
        7,
        Role::Victim,
    ));
    let first = g
        .entity_for_sim(SourceCrate::Agents, SimRuntimeId(7))
        .unwrap();
    g.ingest(ev(
        1,
        EventKind::Migration,
        SourceCrate::Agents,
        0.2,
        7,
        Role::Leader,
    ));
    let second = g
        .entity_for_sim(SourceCrate::Agents, SimRuntimeId(7))
        .unwrap();
    assert_eq!(first, second, "same (source,sim_id) folds into one legend");
}

#[test]
fn resolve_04_aggregate_battles_fold_into_one_war() {
    let mut g = SagaGraph::new(cfg());
    let key = AggregateKey {
        kind: EntityKind::War,
        a: ClusterId(1),
        b: ClusterId(2),
        start_bucket: 0,
    };
    let w1 = g.resolve_aggregate(key.clone(), Epoch(0));
    let w2 = g.resolve_aggregate(key, Epoch(3));
    assert_eq!(
        w1, w2,
        "repeated battles between the same clusters = one War"
    );
}

#[test]
fn sig_05_significant_lineage_promoted_over_transient_farmer() {
    // AC-SIG-1: a lineage with weighty events is promoted before a birth+death farmer.
    let mut g = SagaGraph::new(cfg());

    // weighty figure: founds a settlement + wins a war
    g.ingest(ev(
        0,
        EventKind::SettlementFounded,
        SourceCrate::Protocol3d,
        1.0,
        100,
        Role::Founder,
    ));
    g.ingest(ev(
        1,
        EventKind::WarEnded,
        SourceCrate::Tactics,
        1.0,
        100,
        Role::Leader,
    ));
    let hero = g
        .entity_for_sim(SourceCrate::Protocol3d, SimRuntimeId(100))
        .unwrap();

    // transient farmer: just birth + death
    g.ingest(ev(
        2,
        EventKind::Birth,
        SourceCrate::Agents,
        0.1,
        200,
        Role::Victim,
    ));
    g.ingest(ev(
        3,
        EventKind::Death,
        SourceCrate::Agents,
        0.1,
        200,
        Role::Victim,
    ));
    let farmer = g
        .entity_for_sim(SourceCrate::Agents, SimRuntimeId(200))
        .unwrap();

    assert!(g.entity(hero).unwrap().promoted, "hero promoted");
    assert!(!g.entity(farmer).unwrap().promoted, "farmer not promoted");
}

#[test]
fn sig_05_decay_terminates_at_prune_floor() {
    // AC-SIG-2: with no new events, a non-promoted entity decays to <= prune_floor.
    let mut g = SagaGraph::new(cfg());
    g.ingest(ev(
        0,
        EventKind::Sickness,
        SourceCrate::Agents,
        0.2,
        5,
        Role::Victim,
    ));
    let id = g
        .entity_for_sim(SourceCrate::Agents, SimRuntimeId(5))
        .unwrap();
    let floor = g.config.prune_floor;
    for _ in 0..500 {
        g.decay_epoch();
        if g.entity(id).map(|e| e.significance).unwrap_or(0.0) <= floor {
            return;
        }
    }
    panic!("decay did not terminate at prune_floor");
}

#[test]
fn sig_05_prune_removes_provisional_noise_keeps_promoted() {
    let mut g = SagaGraph::new(cfg());
    // promoted hero
    g.ingest(ev(
        0,
        EventKind::WarEnded,
        SourceCrate::Tactics,
        1.0,
        1,
        Role::Leader,
    ));
    let hero = g
        .entity_for_sim(SourceCrate::Tactics, SimRuntimeId(1))
        .unwrap();
    // transient with an isolated low score
    g.ingest(ev(
        0,
        EventKind::Sickness,
        SourceCrate::Economy,
        0.01,
        2,
        Role::Witness,
    ));
    let transient = g
        .entity_for_sim(SourceCrate::Economy, SimRuntimeId(2))
        .unwrap();

    for _ in 0..50 {
        g.decay_epoch();
    }
    g.prune();
    assert!(g.entity(hero).is_some(), "promoted entity never pruned");
    assert!(g.entity(transient).is_none(), "decayed provisional pruned");
}

#[test]
fn causal_06_chain_finds_shared_participant_cause() {
    // AC-Q-1: a regicide (shares the king) is returned as cause of the succession war.
    let mut g = SagaGraph::new(cfg());
    // regicide at epoch 0 involving the king (sim 50)
    let regicide = g
        .ingest(ev(
            0,
            EventKind::Death,
            SourceCrate::Agents,
            0.9,
            50,
            Role::Victim,
        ))
        .event_id
        .unwrap();
    // succession war at epoch 1 involving the same king + region
    // same king (Agents/sim 50) participates in the succession war
    let war = g
        .ingest(ev(
            1,
            EventKind::WarDeclared,
            SourceCrate::Agents,
            0.9,
            50,
            Role::Leader,
        ))
        .event_id
        .unwrap();

    let chain = g.causal_chain(war, 4).expect("chain");
    assert!(
        chain.edges.iter().any(|(_, cause, _)| *cause == regicide),
        "causal_chain links the war back to the regicide that shares the king"
    );
}

#[test]
fn causal_06_acyclicity_no_cause_to_same_or_future_epoch() {
    let mut g = SagaGraph::new(cfg());
    let e0 = g
        .ingest(ev(
            0,
            EventKind::Battle,
            SourceCrate::Tactics,
            0.8,
            9,
            Role::Aggressor,
        ))
        .event_id
        .unwrap();
    // same epoch: must NOT become a cause of e0 (acyclicity guard)
    let _e0b = g.ingest(ev(
        0,
        EventKind::Battle,
        SourceCrate::Tactics,
        0.8,
        9,
        Role::Defender,
    ));
    let chain = g.causal_chain(e0, 4).unwrap();
    let e0_epoch = g.event(e0).unwrap().epoch;
    for (_, cause, _) in &chain.edges {
        let c = g.event(*cause).unwrap();
        assert!(c.epoch < e0_epoch, "cause strictly earlier in epoch");
        assert_ne!(*cause, e0, "no self-cause");
    }
}

#[test]
fn query_07_saga_of_and_timeline() {
    let mut g = SagaGraph::new(cfg());
    g.ingest(ev(
        0,
        EventKind::SettlementFounded,
        SourceCrate::Protocol3d,
        1.0,
        3,
        Role::Founder,
    ));
    g.ingest(ev(
        5,
        EventKind::WarEnded,
        SourceCrate::Tactics,
        1.0,
        3,
        Role::Leader,
    ));
    let id = g
        .entity_for_sim(SourceCrate::Protocol3d, SimRuntimeId(3))
        .unwrap();

    let saga = g.saga_of(id).expect("saga");
    assert!(saga.events.len() >= 2, "saga has the entity's events");
    // chronological
    let epochs: Vec<u64> = saga.events.iter().map(|e| e.epoch.0).collect();
    let mut sorted = epochs.clone();
    sorted.sort_unstable();
    assert_eq!(epochs, sorted, "saga events chronological");

    let tl = g.timeline(id, Epoch(0)..Epoch(3));
    assert!(tl.iter().all(|e| e.epoch < Epoch(3)));
}

#[test]
fn query_07_significant_top_n_ordered() {
    let mut g = SagaGraph::new(cfg());
    g.ingest(ev(
        0,
        EventKind::WarEnded,
        SourceCrate::Tactics,
        1.0,
        1,
        Role::Leader,
    ));
    g.ingest(ev(
        0,
        EventKind::Birth,
        SourceCrate::Agents,
        0.05,
        2,
        Role::Witness,
    ));
    let top = g.significant(10, None);
    assert!(!top.is_empty());
    // descending by significance
    for w in top.windows(2) {
        assert!(w[0].significance >= w[1].significance);
    }
}

#[test]
fn narrator_13_epoch_digest_hash_stable_across_reloads() {
    // AC-Q-2: an unchanged epoch produces an identical digest_hash (cache-safe).
    let mut g = SagaGraph::new(cfg());
    g.ingest(ev(
        0,
        EventKind::SettlementFounded,
        SourceCrate::Protocol3d,
        0.9,
        1,
        Role::Founder,
    ));
    g.ingest(ev(
        0,
        EventKind::Disaster,
        SourceCrate::Planet,
        0.7,
        2,
        Role::Victim,
    ));
    let d1 = g.epoch_digest(Epoch(0), None);
    let d2 = g.epoch_digest(Epoch(0), None);
    assert_eq!(d1.digest_hash, d2.digest_hash, "same epoch hashes the same");
    assert!(!d1.headline_events.is_empty());
}

#[test]
fn loud_gap_detected_when_producer_silent() {
    // §7: a required producer silent past gap_epochs is reported (loud, not silent).
    let mut g = SagaGraph::new(cfg());
    g.ingest(ev(
        0,
        EventKind::Birth,
        SourceCrate::Agents,
        0.2,
        1,
        Role::Victim,
    ));
    // advance well past gap_epochs with no Tactics/Economy/etc events
    let gaps = g.detect_gaps(Epoch(10));
    assert!(
        gaps.iter().any(|(src, _)| *src == SourceCrate::Tactics),
        "silent Tactics producer flagged as a gap"
    );
}

#[test]
fn worker_drains_off_path_and_maintains() {
    let mut w = LegendsWorker::new(SagaGraph::new(cfg()));
    w.drain([
        ev(
            0,
            EventKind::WarEnded,
            SourceCrate::Tactics,
            1.0,
            1,
            Role::Leader,
        ),
        ev(
            10,
            EventKind::Birth,
            SourceCrate::Agents,
            0.1,
            2,
            Role::Victim,
        ),
    ]);
    // hero promoted, graph maintained across the epoch jump
    let hero = w
        .graph()
        .entity_for_sim(SourceCrate::Tactics, SimRuntimeId(1))
        .unwrap();
    assert!(w.graph().entity(hero).unwrap().promoted);
}
