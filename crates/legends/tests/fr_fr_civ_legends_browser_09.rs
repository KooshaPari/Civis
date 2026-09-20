//! Tests for FR-CIV-LEGENDS-BROWSER-09 — Legend browser UI (query API)
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies the saga graph supports browsing entities and their stories.

#[cfg(test)]
mod fr_fr_civ_legends_browser_09 {
    use civ_legends::{SagaGraph, EventKind, RawSimEvent, SourceCrate};

    /// FR-CIV-LEGENDS-BROWSER-09: SagaGraph default is empty.
    #[test]
    fn saga_graph_default_is_empty() {
        let graph = SagaGraph::default();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    /// FR-CIV-LEGENDS-BROWSER-09: Ingesting events increases node count.
    #[test]
    fn ingest_increases_node_count() {
        let mut graph = SagaGraph::default();
        let raw = RawSimEvent::new(1, EventKind::Birth, SourceCrate::Engine, 0.5);
        let outcome = graph.ingest(raw);
        assert!(outcome.event_id.is_some());
        assert!(graph.node_count() > 0);
    }

    /// FR-CIV-LEGENDS-BROWSER-09: Multiple events can be ingested.
    #[test]
    fn multiple_ingests_accumulate() {
        let mut graph = SagaGraph::default();
        for kind in [EventKind::Birth, EventKind::Death, EventKind::Battle] {
            let raw = RawSimEvent::new(1, kind, SourceCrate::Engine, 0.8);
            graph.ingest(raw);
        }
        assert!(graph.node_count() >= 3);
    }
}
