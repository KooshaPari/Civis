//! Tests for FR-CIV-LEGENDS-NARRATOR-13 — AI narrator integration
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies that the narrator can read saga data for prose generation.

#[cfg(test)]
mod fr_fr_civ_legends_narrator_13 {
    use civ_legends::{SagaGraph, EventKind, RawSimEvent, SourceCrate};

    /// FR-CIV-LEGENDS-NARRATOR-13: Graph accumulates events for narrator reading.
    #[test]
    fn graph_accumulates_for_narrator() {
        let mut graph = SagaGraph::default();
        for kind in [EventKind::WarDeclared, EventKind::GreatWork, EventKind::Plague] {
            let raw = RawSimEvent::new(10, kind, SourceCrate::Engine, 0.9);
            graph.ingest(raw);
        }
        assert!(graph.node_count() >= 3);
    }

    /// FR-CIV-LEGENDS-NARRATOR-13: Role weights prioritize leaders over witnesses.
    #[test]
    fn role_weight_leader_greater_than_witness() {
        assert!(civ_legends::Role::Leader.weight() > civ_legends::Role::Witness.weight());
    }

    /// FR-CIV-LEGENDS-NARRATOR-13: empty_saga_reason for entity with no events.
    #[test]
    fn narrator_handles_empty_saga() {
        let graph = SagaGraph::default();
        let id = civ_legends::LegendEntityId(0);
        let reason = graph.empty_saga_reason(id);
        assert!(reason.is_some());
        // Should be UnknownEntity since nothing was ingested
        assert!(matches!(
            reason.unwrap(),
            civ_legends::EmptySagaReason::UnknownEntity
        ));
    }
}
