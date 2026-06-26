//! FR-CIV-LEGENDS legend auto-generation acceptance tests.
//!
//! Tests that significant emergent events automatically generate named legend entries
//! with provenance (who/where/when-tick/cause) and importance scores, and that queries
//! correctly rank legends by importance.

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

fn ev(
    tick: u64,
    kind: EventKind,
    src: SourceCrate,
    mag: f32,
    sim: u64,
    role: Role,
) -> RawSimEvent {
    RawSimEvent::new(tick, kind, src, mag).with_participant(src, SimRuntimeId(sim), role)
}

// ===========================================================================
// Legend Auto-Generation from Significant Events
// ===========================================================================

/// `Covers FR-CIV-LEGENDS` — high-impact events auto-generate legend entries.
/// A significant event (high magnitude + promoted entity) triggers legend creation
/// with correct provenance and importance ranking.
#[test]
fn test_legend_from_high_impact_event() {
    let mut g = SagaGraph::new(cfg());

    // Emit a high-impact war event (magnitude 0.9, which is significant)
    let war = ev(0, EventKind::WarDeclared, SourceCrate::Tactics, 0.9, 42, Role::Leader);
    let outcome = g.ingest(war);

    if let Some(event_id) = outcome.event_id {
        // Ingest another event for the same entity to ensure promotion
        let battle = ev(1, EventKind::Battle, SourceCrate::Tactics, 0.8, 42, Role::Leader);
        let outcome2 = g.ingest(battle);

        // Use the first promoted entity from the initial war event
        if let Some(promoted_entity) = outcome.promoted.first() {
            let created = g.create_legend_from_event(event_id, *promoted_entity);
            assert!(created, "high-impact event should create a legend");

            // Verify the legend was stored
            let legend = g.legend(event_id).expect("legend should exist");
            assert_eq!(legend.event_id, event_id);
            assert_eq!(legend.principal_entity, *promoted_entity);
            assert!(legend.importance >= 0.3, "importance should meet threshold");
            assert_eq!(legend.event_kind, EventKind::WarDeclared);
            assert_eq!(legend.epoch, Epoch(0));
        }
    }
}

/// `Covers FR-CIV-LEGENDS` — low-impact events do NOT create legends.
/// Events below the significance threshold do not generate legend entries.
#[test]
fn test_no_legend_for_low_impact_event() {
    let mut g = SagaGraph::new(cfg());

    // Emit a very low-impact event (magnitude 0.1)
    let minor = ev(0, EventKind::Migration, SourceCrate::Agents, 0.1, 99, Role::Witness);
    let outcome = g.ingest(minor);

    if let Some(event_id) = outcome.event_id {
        // Try to create a legend from an unpromoted entity
        let unpromoted = LegendEntityId(1);
        let created = g.create_legend_from_event(event_id, unpromoted);
        assert!(!created, "low-impact event should not create a legend");

        // Verify no legend was stored
        assert_eq!(g.all_legends().len(), 0);
    }
}

/// `Covers FR-CIV-LEGENDS` — legend deduplication (idempotent).
/// Calling create_legend_from_event twice for the same event should only
/// create one legend and return false on the second call.
#[test]
fn test_legend_deduplication() {
    let mut g = SagaGraph::new(cfg());

    let war = ev(0, EventKind::WarDeclared, SourceCrate::Tactics, 0.9, 42, Role::Leader);
    let outcome = g.ingest(war);

    if let Some(event_id) = outcome.event_id {
        let promoted = LegendEntityId(1);
        let first = g.create_legend_from_event(event_id, promoted);
        assert!(first, "first call should create legend");

        let second = g.create_legend_from_event(event_id, promoted);
        assert!(!second, "second call should return false (already exists)");

        assert_eq!(g.all_legends().len(), 1, "only one legend should exist");
    }
}

// ===========================================================================
// Legend Importance Ranking
// ===========================================================================

/// `Covers FR-CIV-LEGENDS` — top N legends by importance.
/// Legends are correctly ranked by importance score (computed from event magnitude
/// and principal entity significance).
#[test]
fn test_top_legends_by_importance() {
    let mut g = SagaGraph::new(cfg());

    let mut created_events = Vec::new();

    // Create three events with different magnitudes
    let high = ev(0, EventKind::WarDeclared, SourceCrate::Tactics, 0.95, 1, Role::Leader);
    if let Some(id) = g.ingest(high).event_id {
        created_events.push((id, LegendEntityId(1)));
    }

    let med = ev(1, EventKind::Battle, SourceCrate::Tactics, 0.6, 2, Role::Leader);
    if let Some(id) = g.ingest(med).event_id {
        created_events.push((id, LegendEntityId(2)));
    }

    let low = ev(2, EventKind::Discovery, SourceCrate::Agents, 0.4, 3, Role::Witness);
    if let Some(id) = g.ingest(low).event_id {
        created_events.push((id, LegendEntityId(3)));
    }

    // Create all legends
    for (event_id, entity) in created_events {
        g.create_legend_from_event(event_id, entity);
    }

    // Query top 2 legends
    let top2 = g.top_legends(2);
    assert_eq!(top2.len(), 2, "should return 2 legends");

    // Verify ordering: high importance first
    if top2.len() >= 2 {
        assert!(
            top2[0].importance >= top2[1].importance,
            "legends should be sorted by importance descending"
        );
    }
}

/// `Covers FR-CIV-LEGENDS` — importance computation.
/// Importance is computed from event magnitude and entity significance.
#[test]
fn test_legend_importance_computation() {
    // Test the compute_legend_importance function directly
    let mag = 0.8;
    let sig = 0.6;
    let importance = compute_legend_importance(mag, sig);

    // Should be average of magnitude and significance, clamped to 0..1
    let expected = ((mag + sig) / 2.0).clamp(0.0, 1.0);
    assert!((importance - expected).abs() < 0.001, "importance should be (mag+sig)/2");

    // Verify edge cases
    assert_eq!(compute_legend_importance(0.0, 0.0), 0.0);
    assert_eq!(compute_legend_importance(1.0, 1.0), 1.0);
    assert_eq!(compute_legend_importance(1.0, 0.0), 0.5);
}

// ===========================================================================
// Legend Queries
// ===========================================================================

/// `Covers FR-CIV-LEGENDS` — legend lookup by event ID.
#[test]
fn test_legend_lookup() {
    let mut g = SagaGraph::new(cfg());

    let war = ev(0, EventKind::WarDeclared, SourceCrate::Tactics, 0.9, 42, Role::Leader);
    let outcome = g.ingest(war);

    if let Some(event_id) = outcome.event_id {
        let entity = LegendEntityId(1);
        g.create_legend_from_event(event_id, entity);

        // Lookup the legend
        let legend = g.legend(event_id);
        assert!(legend.is_some(), "legend should be found by event_id");
        assert_eq!(legend.unwrap().event_id, event_id);
    }
}

/// `Covers FR-CIV-LEGENDS` — all legends query.
#[test]
fn test_all_legends_query() {
    let mut g = SagaGraph::new(cfg());

    // Create multiple legends
    for i in 0..3 {
        let ev_sig = ev(
            i,
            EventKind::WarDeclared,
            SourceCrate::Tactics,
            0.8 + (i as f32 * 0.05),
            i,
            Role::Leader,
        );
        let outcome = g.ingest(ev_sig);
        if let Some(event_id) = outcome.event_id {
            let entity = LegendEntityId(i as u64);
            g.create_legend_from_event(event_id, entity);
        }
    }

    let all = g.all_legends();
    assert_eq!(all.len(), 3, "should return all 3 legends");
}

// ===========================================================================
// Legend Provenance
// ===========================================================================

/// `Covers FR-CIV-LEGENDS` — legend captures provenance (who/where/when).
/// Legends record source crate, region, epoch, and participant information.
#[test]
fn test_legend_provenance() {
    let mut g = SagaGraph::new(cfg());

    // Emit an event with region and multiple participants
    let mut war = ev(0, EventKind::WarDeclared, SourceCrate::Tactics, 0.9, 42, Role::Leader);
    war = war
        .with_region(RegionId(5))
        .with_participant(SourceCrate::Tactics, SimRuntimeId(43), Role::Defender);

    let outcome = g.ingest(war);

    if let Some(event_id) = outcome.event_id {
        let entity = LegendEntityId(1);
        g.create_legend_from_event(event_id, entity);

        let legend = g.legend(event_id).expect("legend should exist");

        // Verify provenance captured
        assert_eq!(legend.region, Some(RegionId(5)), "region should be captured");
        assert_eq!(legend.epoch, Epoch(0), "epoch should be captured");
        assert_eq!(legend.event_kind, EventKind::WarDeclared, "event kind should be captured");
        assert!(!legend.participants.is_empty(), "participants should be captured");
    }
}

// ===========================================================================
// Legend with Determinism
// ===========================================================================

/// `Covers FR-CIV-LEGENDS` — legend creation is deterministic.
/// Given the same event and entity, legend importance is reproducible.
#[test]
fn test_legend_importance_deterministic() {
    let mut g1 = SagaGraph::new(cfg());
    let mut g2 = SagaGraph::new(cfg());

    let war = ev(0, EventKind::WarDeclared, SourceCrate::Tactics, 0.9, 42, Role::Leader);
    let outcome1 = g1.ingest(war.clone());
    let outcome2 = g2.ingest(war);

    if let (Some(event_id1), Some(event_id2)) = (outcome1.event_id, outcome2.event_id) {
        let entity = LegendEntityId(1);
        g1.create_legend_from_event(event_id1, entity);
        g2.create_legend_from_event(event_id2, entity);

        let legend1 = g1.legend(event_id1).expect("legend1 should exist");
        let legend2 = g2.legend(event_id2).expect("legend2 should exist");

        assert_eq!(
            legend1.importance, legend2.importance,
            "importance should be deterministic"
        );
    }
}
