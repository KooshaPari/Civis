//! Tests for FR-CIV-LEGENDS-INSPECT-08 — Inspector query API
//!
//! Epic: FR-CIV-LEGENDS
//! Verifies the read-only query API for the inspector.

#[cfg(test)]
mod fr_fr_civ_legends_inspect_08 {
    use civ_legends::SagaGraph;

    /// FR-CIV-LEGENDS-INSPECT-08: Query API version is defined.
    #[test]
    fn query_api_version_is_defined() {
        assert!(civ_legends::query::QUERY_API_VERSION > 0);
    }

    /// FR-CIV-LEGENDS-INSPECT-08: Entity lookup returns None for unknown id.
    #[test]
    fn entity_lookup_unknown_returns_none() {
        let graph = SagaGraph::default();
        let id = civ_legends::LegendEntityId(999);
        assert!(graph.entity(id).is_none());
    }

    /// FR-CIV-LEGENDS-INSPECT-08: Event lookup returns None for unknown id.
    #[test]
    fn event_lookup_unknown_returns_none() {
        let graph = SagaGraph::default();
        let id = civ_legends::LegendEventId(999);
        assert!(graph.event(id).is_none());
    }

    /// FR-CIV-LEGENDS-INSPECT-08: empty_saga_reason returns UnknownEntity for nonexistent.
    #[test]
    fn empty_saga_unknown_entity() {
        let graph = SagaGraph::default();
        let id = civ_legends::LegendEntityId(42);
        let reason = graph.empty_saga_reason(id);
        assert!(reason.is_some());
    }
}
