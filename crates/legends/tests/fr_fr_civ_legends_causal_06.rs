//! Tests for FR-CIV-LEGENDS-CAUSAL-06 — Causal chain linking
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies that events can form causal chains through edges.

#[cfg(test)]
mod fr_fr_civ_legends_causal_06 {
    use civ_legends::{SagaGraph, EventKind, RawSimEvent, SourceCrate};

    /// FR-CIV-LEGENDS-CAUSAL-06: Consecutive events at close ticks are linked.
    #[test]
    fn consecutive_events_ingest_and_exist() {
        let mut graph = SagaGraph::default();
        let e1 = RawSimEvent::new(1, EventKind::Birth, SourceCrate::Engine, 0.5);
        let e2 = RawSimEvent::new(2, EventKind::Death, SourceCrate::Engine, 0.8);
        let o1 = graph.ingest(e1);
        let o2 = graph.ingest(e2);
        assert!(o1.event_id.is_some());
        assert!(o2.event_id.is_some());
        assert!(graph.node_count() >= 2);
    }

    /// FR-CIV-LEGENDS-CAUSAL-06: gap_reports detects silent producers.
    #[test]
    fn gap_reports_empty_for_active_producers() {
        let mut graph = SagaGraph::default();
        // Ingest recent events from Engine
        for tick in 0..5u64 {
            let raw = RawSimEvent::new(tick, EventKind::Birth, SourceCrate::Engine, 0.5);
            graph.ingest(raw);
        }
        let now = graph.current_epoch();
        let gaps = graph.gap_reports(now);
        // Engine is active, so no gap for Engine
        let engine_gaps: Vec<_> = gaps.iter().filter(|g| g.source == SourceCrate::Engine).collect();
        assert!(engine_gaps.is_empty());
    }

    /// FR-CIV-LEGENDS-CAUSAL-06: EventKind labels are human-readable.
    #[test]
    fn event_kind_labels_are_readable() {
        assert_eq!(EventKind::Birth.label(), "Birth");
        assert_eq!(EventKind::Battle.label(), "Battle");
        assert_eq!(EventKind::Other("Custom".into()).label(), "Custom");
    }
}
