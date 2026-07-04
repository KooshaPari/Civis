//! Additional coverage tests for civ-legends (FR-CIV-TEST-014).
//! Targets uncovered pub fns not exercised in saga_graph.rs /
//! fr_legends_completion.rs / legends_coverage.rs.

use civ_legends::*;
use civ_legends::config::{kind_weight, kind_affinity};

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

// ---------------------------------------------------------------------------
// LegendsConfig::epoch_of
// ---------------------------------------------------------------------------

#[test]
fn legends_config_epoch_of_divides_tick_by_ticks_per_epoch() {
    let cfg = LegendsConfig {
        ticks_per_epoch: 10,
        ..LegendsConfig::default()
    };
    assert_eq!(cfg.epoch_of(0), Epoch(0));
    assert_eq!(cfg.epoch_of(9), Epoch(0));
    assert_eq!(cfg.epoch_of(10), Epoch(1));
    assert_eq!(cfg.epoch_of(99), Epoch(9));
    assert_eq!(cfg.epoch_of(100), Epoch(10));
}

#[test]
fn legends_config_epoch_of_handles_zero_ticks_per_epoch() {
    // ticks_per_epoch is guarded by .max(1) so it never panics with division by zero.
    let cfg = LegendsConfig {
        ticks_per_epoch: 0,
        ..LegendsConfig::default()
    };
    // 0 clamped to 1, so epoch = tick / 1 = tick
    assert_eq!(cfg.epoch_of(42), Epoch(42));
}

// ---------------------------------------------------------------------------
// kind_weight and kind_affinity (config.rs pub fns)
// ---------------------------------------------------------------------------

#[test]
fn kind_weight_returns_expected_tiers() {
    // Tier 1.0
    assert_eq!(kind_weight(&EventKind::Death), 1.0);
    assert_eq!(kind_weight(&EventKind::WarDeclared), 1.0);
    assert_eq!(kind_weight(&EventKind::Extinction), 1.0);
    // Tier 0.8
    assert_eq!(kind_weight(&EventKind::Battle), 0.8);
    assert_eq!(kind_weight(&EventKind::SettlementFounded), 0.8);
    // Tier 0.2
    assert_eq!(kind_weight(&EventKind::Sickness), 0.2);
    // Promotion = 0
    assert_eq!(kind_weight(&EventKind::Promotion), 0.0);
    // Other
    assert_eq!(kind_weight(&EventKind::Other("custom".to_owned())), 0.5);
}

#[test]
fn kind_affinity_returns_expected_pairs() {
    assert_eq!(kind_affinity(&EventKind::Famine, &EventKind::Migration), 1.0);
    assert_eq!(kind_affinity(&EventKind::WarDeclared, &EventKind::Battle), 0.9);
    assert_eq!(kind_affinity(&EventKind::Battle, &EventKind::Death), 0.8);
    // Unrelated pair -> 0.0
    assert_eq!(kind_affinity(&EventKind::Birth, &EventKind::Raid), 0.0);
}

// ---------------------------------------------------------------------------
// Role::weight
// ---------------------------------------------------------------------------

#[test]
fn role_weight_ordering_leader_gt_witness() {
    assert!(Role::Leader.weight() > Role::Witness.weight());
    assert!(Role::Founder.weight() > Role::Victim.weight());
    assert_eq!(Role::Leader.weight(), 1.0);
    assert_eq!(Role::Witness.weight(), 0.2);
    assert_eq!(Role::Aggressor.weight(), 0.8);
    assert_eq!(Role::Defender.weight(), 0.8);
    assert_eq!(Role::Builder.weight(), 0.7);
    assert_eq!(Role::Victim.weight(), 0.5);
    assert_eq!(Role::Effect.weight(), 0.5);
}

// ---------------------------------------------------------------------------
// EventKind::label
// ---------------------------------------------------------------------------

#[test]
fn event_kind_label_other_variant_echoes_string() {
    assert_eq!(EventKind::Other("foobar".to_owned()).label(), "foobar");
    // Non-Other variants format as Debug name (just ensure non-empty)
    assert!(!EventKind::Battle.label().is_empty());
    assert!(!EventKind::Promotion.label().is_empty());
}

// ---------------------------------------------------------------------------
// SagaGraph::node_count, edge_count, current_epoch
// ---------------------------------------------------------------------------

#[test]
fn saga_graph_node_edge_current_epoch_after_ingest() {
    let mut g = SagaGraph::new(cfg());
    assert_eq!(g.node_count(), 0);
    assert_eq!(g.edge_count(), 0);
    assert_eq!(g.current_epoch(), Epoch(0));

    g.ingest(ev(5, EventKind::Battle, SourceCrate::Tactics, 0.8, 1, Role::Aggressor));
    // 1 entity node + 1 event node = 2; 1 participation edge
    assert_eq!(g.node_count(), 2);
    assert_eq!(g.edge_count(), 1);
    assert_eq!(g.current_epoch(), Epoch(5));
}

// ---------------------------------------------------------------------------
// RawSimEvent::with_region
// ---------------------------------------------------------------------------

#[test]
fn raw_sim_event_with_region_sets_region_field() {
    let raw = RawSimEvent::new(0, EventKind::Disaster, SourceCrate::Planet, 0.9)
        .with_region(RegionId(42));
    assert_eq!(raw.region, Some(RegionId(42)));
}

// ---------------------------------------------------------------------------
// EmptySagaReason::reason_text for all variants
// ---------------------------------------------------------------------------

#[test]
fn empty_saga_reason_text_all_variants_non_empty() {
    let unknown = EmptySagaReason::UnknownEntity;
    let text = unknown.reason_text();
    assert!(!text.is_empty());
    assert!(text.contains("not in saga graph"));

    let not_entity = EmptySagaReason::NotAnEntity;
    let text = not_entity.reason_text();
    assert!(!text.is_empty());
    assert!(text.contains("event"));

    let no_events = EmptySagaReason::NoEventsYet;
    let text = no_events.reason_text();
    assert!(!text.is_empty());
    assert!(text.contains("no witnessed events"));

    let gap = EmptySagaReason::ProducerGap {
        source: SourceCrate::Agents,
        reason: "silent for 5 epochs".to_owned(),
    };
    let text = gap.reason_text();
    assert_eq!(text, "silent for 5 epochs");
}

// ---------------------------------------------------------------------------
// HistoricalEvent::engine_promotion
// ---------------------------------------------------------------------------

#[test]
fn historical_event_engine_promotion_flags_are_correct() {
    let promo = HistoricalEvent::engine_promotion(10, Epoch(1), Some(RegionId(3)), LegendEntityId(7));
    assert!(promo.is_engine_authored());
    assert!(!promo.authored_outcome);
    assert_eq!(promo.kind, EventKind::Promotion);
    assert_eq!(promo.source, SourceCrate::Engine);
    assert_eq!(promo.raw_magnitude, 0.0);
    assert_eq!(promo.region, Some(RegionId(3)));
    assert!(promo.participants.iter().any(|(e, r)| *e == LegendEntityId(7) && *r == Role::Leader));
}

// ---------------------------------------------------------------------------
// Ocean::embellishment_gate / swap_gate
// ---------------------------------------------------------------------------

#[test]
fn ocean_embellishment_gate_clamps_to_0_1() {
    let min = Ocean { openness: 0.0, conscientiousness: 1.0, extraversion: 0.0, agreeableness: 1.0, neuroticism: 0.0 };
    let max = Ocean { openness: 1.0, conscientiousness: 0.0, extraversion: 1.0, agreeableness: 0.0, neuroticism: 1.0 };
    let g_min = min.embellishment_gate();
    let g_max = max.embellishment_gate();
    assert!((0.0..=1.0).contains(&g_min));
    assert!((0.0..=1.0).contains(&g_max));
    assert!(g_max > g_min, "open/unconsientious should gate higher than closed/conscientious");
}

#[test]
fn ocean_swap_gate_clamps_to_0_1() {
    let low = Ocean { openness: 0.0, conscientiousness: 1.0, extraversion: 0.0, agreeableness: 1.0, neuroticism: 0.0 };
    let high = Ocean { openness: 1.0, conscientiousness: 0.0, extraversion: 1.0, agreeableness: 0.0, neuroticism: 1.0 };
    let g_low = low.swap_gate();
    let g_high = high.swap_gate();
    assert!((0.0..=1.0).contains(&g_low));
    assert!((0.0..=1.0).contains(&g_high));
    assert!(g_high > g_low);
}

// ---------------------------------------------------------------------------
// DefaultNameResolver::resolve
// ---------------------------------------------------------------------------

#[test]
fn default_name_resolver_produces_entity_prefix() {
    let r = DefaultNameResolver;
    let name = r.resolve(NameRef(42));
    assert_eq!(name, "entity:42");
    // Must not use '#' (tracery reserved)
    assert!(!name.contains('#'));
}

// ---------------------------------------------------------------------------
// render() (the simple non-tracery render surface)
// ---------------------------------------------------------------------------

#[test]
fn render_produces_non_empty_string_with_epoch_and_kind() {
    let rumor = Rumor {
        event_id: LegendEventId(1),
        origin_epoch: Epoch(7),
        hop: 0,
        subject: LegendEntityId(99),
        claimed_kind: EventKind::Battle,
        claimed_magnitude: 0.75,
        tags: smallvec::SmallVec::new(),
        text: "origin".to_owned(),
        chain: smallvec::SmallVec::new(),
        salience: 0.0,
    };
    let text = render(&rumor);
    assert!(text.contains('7'), "should contain epoch 7: got '{}'", text);
    assert!(!text.is_empty());
}

// ---------------------------------------------------------------------------
// forward_chain (inverse causal walk)
// ---------------------------------------------------------------------------

#[test]
fn forward_chain_returns_some_on_known_event() {
    let mut g = SagaGraph::new(cfg());
    let eid = g.ingest(ev(0, EventKind::Death, SourceCrate::Agents, 0.9, 1, Role::Victim))
        .event_id.unwrap();
    g.ingest(ev(1, EventKind::Migration, SourceCrate::Agents, 0.7, 1, Role::Leader));

    let chain = g.forward_chain(eid, 3);
    assert!(chain.is_some(), "forward_chain must return Some for a known event");
    let chain = chain.unwrap();
    assert_eq!(chain.root, eid);
    assert!(!chain.events.is_empty());
}

#[test]
fn forward_chain_returns_none_for_unknown_event() {
    let g = SagaGraph::new(cfg());
    assert!(g.forward_chain(LegendEventId(9999), 4).is_none());
}

// ---------------------------------------------------------------------------
// SagaGraph::neighbors returns empty for unknown entity
// ---------------------------------------------------------------------------

#[test]
fn neighbors_returns_empty_for_unknown_entity() {
    let g = SagaGraph::new(cfg());
    let result = g.neighbors(LegendEntityId(9999));
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// summary_key produces stable non-zero hash
// ---------------------------------------------------------------------------

#[test]
fn summary_key_is_stable_and_deterministic() {
    let participants = [LegendEntityId(1), LegendEntityId(2)];
    let k1 = summary_key(&EventKind::Battle, &participants, 0.5, Epoch(3));
    let k2 = summary_key(&EventKind::Battle, &participants, 0.5, Epoch(3));
    assert_eq!(k1, k2, "summary_key must be deterministic");
    // Different magnitude -> different hash
    let k3 = summary_key(&EventKind::Battle, &participants, 0.9, Epoch(3));
    assert_ne!(k1, k3);
}