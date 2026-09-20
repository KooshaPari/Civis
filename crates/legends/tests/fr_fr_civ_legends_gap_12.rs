//! Tests for FR-CIV-LEGENDS-GAP-12 — Producer gap detection
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies that the legends system detects silent producers.

#[cfg(test)]
mod fr_fr_civ_legends_gap_12 {
    use civ_legends::{SagaGraph, EventKind, RawSimEvent, SourceCrate};

    /// FR-CIV-LEGENDS-GAP-12: Empty graph has no gap reports.
    #[test]
    fn empty_graph_no_gaps() {
        let graph = SagaGraph::default();
        let now = graph.current_epoch();
        let gaps = graph.gap_reports(now);
        assert!(gaps.is_empty());
    }

    /// FR-CIV-LEGENDS-GAP-12: Recent events means no gap for that producer.
    #[test]
    fn recent_events_no_gaps() {
        let mut graph = SagaGraph::default();
        for tick in 0..5u64 {
            let raw = RawSimEvent::new(tick, EventKind::Birth, SourceCrate::Engine, 0.5);
            graph.ingest(raw);
        }
        let now = graph.current_epoch();
        let gaps = graph.gap_reports(now);
        assert!(gaps.is_empty());
    }

    /// FR-CIV-LEGENDS-GAP-12: LegendsConfig has gap_epochs field.
    #[test]
    fn config_has_gap_epochs() {
        let config = civ_legends::LegendsConfig::default();
        assert!(config.gap_epochs > 0);
    }
}
