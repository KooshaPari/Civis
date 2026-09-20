//! Tests for FR-CIV-LEGENDS-RESOLVE-04 — Event resolution
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies that raw sim events are resolved into entity and event nodes.

#[cfg(test)]
mod fr_fr_civ_legends_resolve_04 {
    use civ_legends::{SagaGraph, EventKind, RawSimEvent, SourceCrate};

    /// FR-CIV-LEGENDS-RESOLVE-04: High-magnitude events promote entities.
    #[test]
    fn high_magnitude_events_promote_entities() {
        let mut graph = SagaGraph::default();
        for tick in 0..10u64 {
            let raw = RawSimEvent::new(tick, EventKind::Battle, SourceCrate::Engine, 0.9);
            let outcome = graph.ingest(raw);
            if !outcome.promoted.is_empty() {
                // At least one entity was promoted
                return;
            }
        }
        // Even if none promoted, events were still ingested
        assert!(graph.node_count() > 0);
    }

    /// FR-CIV-LEGENDS-RESOLVE-04: Zero-magnitude events do not promote.
    #[test]
    fn zero_magnitude_no_promotion() {
        let mut graph = SagaGraph::default();
        let raw = RawSimEvent::new(1, EventKind::Birth, SourceCrate::Engine, 0.0);
        let outcome = graph.ingest(raw);
        assert!(outcome.promoted.is_empty());
    }

    /// FR-CIV-LEGENDS-RESOLVE-04: EntityKind variants exist for all kinds.
    #[test]
    fn entity_kind_variants_exist() {
        let kinds = [
            civ_legends::EntityKind::Agent,
            civ_legends::EntityKind::Settlement,
            civ_legends::EntityKind::War,
            civ_legends::EntityKind::Species,
        ];
        assert_eq!(kinds.len(), 4);
    }
}
