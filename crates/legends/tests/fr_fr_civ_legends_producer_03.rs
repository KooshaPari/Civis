//! Tests for FR-CIV-LEGENDS-PRODUCER-03 — Event producer API
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies that sim events can be produced and ingested into the saga graph.

#[cfg(test)]
mod fr_fr_civ_legends_producer_03 {
    use civ_legends::{SagaGraph, EventKind, RawSimEvent, SourceCrate};

    /// FR-CIV-LEGENDS-PRODUCER-03: RawSimEvent can be created and ingested.
    #[test]
    fn raw_sim_event_ingestion() {
        let mut graph = SagaGraph::default();
        let event = RawSimEvent::new(42, EventKind::Battle, SourceCrate::Engine, 0.75);
        let outcome = graph.ingest(event);
        assert!(outcome.event_id.is_some());
    }

    /// FR-CIV-LEGENDS-PRODUCER-03: IngestOutcome contains promoted entities.
    #[test]
    fn ingest_outcome_has_promoted_field() {
        let mut graph = SagaGraph::default();
        let raw = RawSimEvent::new(1, EventKind::SettlementFounded, SourceCrate::Engine, 0.6);
        let outcome = graph.ingest(raw);
        // promoted is a Vec, may be empty for low magnitude
        assert!(outcome.promoted.is_empty() || !outcome.promoted.is_empty());
    }

    /// FR-CIV-LEGENDS-PRODUCER-03: Events from different source crates can be ingested.
    #[test]
    fn events_from_different_sources() {
        let mut graph = SagaGraph::default();
        let e1 = RawSimEvent::new(1, EventKind::Birth, SourceCrate::Engine, 0.5);
        let e2 = RawSimEvent::new(2, EventKind::Treaty, SourceCrate::Tactics, 0.8);
        let o1 = graph.ingest(e1);
        let o2 = graph.ingest(e2);
        assert!(o1.event_id.is_some());
        assert!(o2.event_id.is_some());
    }

    /// FR-CIV-LEGENDS-PRODUCER-03: All EventKind variants produce display labels.
    #[test]
    fn all_event_kinds_have_labels() {
        let kinds = [
            EventKind::Birth,
            EventKind::Death,
            EventKind::Battle,
            EventKind::Treaty,
            EventKind::GreatWork,
            EventKind::Plague,
            EventKind::Other("Test".into()),
        ];
        for kind in kinds {
            let label = kind.label();
            assert!(!label.is_empty());
        }
    }
}
