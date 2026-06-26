//! # civ-legends — emergent-history saga-graph engine
//!
//! A `petgraph::StableDiGraph<LegendNode, LegendEdge>` over the sim event stream:
//! **Entity** nodes (agents/settlements/clusters/species/wars/disasters) + **Event**
//! nodes joined by **causal edges** (`CausedBy`/`ParticipatedIn`/`Succeeded`/…), with
//! a measured-significance score that promotes "historically significant" entities, and
//! a read-only query API ([`SagaGraph::saga_of`], [`SagaGraph::causal_chain`],
//! [`SagaGraph::epoch_digest`]) the inspector + AI narrator read.
//!
//! Design spec: `docs/design/legends-engine.md`. The engine is a **measured record of
//! what the sim already produced**, never a generator of outcomes (emergence charter).
//! It runs **off the sim hot path**: producers emit a cheap [`RawSimEvent`] onto the
//! existing `crates/watch` bus; a [`worker::LegendsWorker`] drains it and does all
//! resolution/scoring/linking — the tick never blocks on the engine.
//!
//! Requirements: FR-CIV-LEGENDS-GRAPH-01, -INGEST-02, -RESOLVE-04, -SIG-05,
//! -CAUSAL-06, -QUERY-07, -NARRATOR-13; NFR-CIV-LEGENDS-SCALE-02, -CONFIG-04, -LOUD-03.

pub mod config;
pub mod graph;
pub mod ids;
pub mod model;
pub mod query;
pub mod rumor;
pub mod worker;

pub use config::LegendsConfig;
pub use graph::{AggregateKey, EmptySagaReason, GapReport, IngestOutcome, SagaGraph};
pub use ids::{
    ClusterId, Epoch, LegendEntityId, LegendEventId, NameRef, Provenance, RawEventRef, RegionId,
    SimRef, SimRuntimeId, SourceCrate,
};
pub use model::{
    summary_key, EntityKind, EntityNode, EventKind, EventNode, HistoricalEvent, LegendEdge,
    LegendNode, PromotionCriteria, RawSimEvent, Role, Tag,
};
pub use query::{CausalDag, DigestEvent, EntityRef, EpochDigest, NamedEntitySummary, NamedLegendsResult, Saga, QUERY_API_VERSION};
pub use rumor::{
    register_render, render, retell, witness, Chronicle, ChronicleEntry, DefaultNameResolver,
    HistorianMind, NameResolver, Ocean, Register, Rumor, RumorMill,
};
pub use worker::LegendsWorker;

#[cfg(test)]
mod tests {
    use crate::ids::{Epoch, SimRuntimeId, SourceCrate};
    use crate::model::{EventKind, RawSimEvent, Role};
    use crate::graph::SagaGraph;

    /// FR-CIV-LEGENDS deepening — new event kinds ingest and appear in the graph.
    #[test]
    fn new_event_kinds_ingest_correctly() {
        let mut graph = SagaGraph::default();
        for kind in [
            EventKind::Treaty,
            EventKind::Betrayal,
            EventKind::GreatWork,
            EventKind::Plague,
        ] {
            let raw = RawSimEvent::new(1, kind, SourceCrate::Engine, 0.8);
            let outcome = graph.ingest(raw);
            assert!(outcome.event_id.is_some(), "expected event inserted into graph");
        }
        // 4 event nodes created.
        let event_count = graph
            .graph()
            .node_indices()
            .filter(|i| graph.graph()[*i].as_event().is_some())
            .count();
        assert_eq!(event_count, 4, "expected 4 event nodes for 4 new kinds");
    }

    /// FR-CIV-LEGENDS deepening — promote_to_legend sets title and marks entity.
    #[test]
    fn promotion_triggers_on_threshold() {
        let mut graph = SagaGraph::default();
        // Ingest enough high-magnitude events to promote.
        for tick in 0..10u64 {
            let raw = RawSimEvent::new(tick, EventKind::Battle, SourceCrate::Tactics, 1.0)
                .with_participant(SourceCrate::Tactics, SimRuntimeId(1), Role::Aggressor);
            graph.ingest(raw);
        }
        let eid = graph
            .entity_for_sim(SourceCrate::Tactics, SimRuntimeId(1))
            .expect("entity should exist");

        let result = graph.promote_to_legend(eid, "The Veteran".to_string(), Role::Leader);
        assert!(result.is_ok(), "promote_to_legend should succeed: {result:?}");
        let entity = graph.entity(eid).expect("entity should be in graph");
        assert_eq!(entity.title.as_deref(), Some("The Veteran"));
        assert!(entity.promoted, "entity should be marked promoted");
    }

    /// FR-CIV-LEGENDS deepening — query_named_legends returns only titled entities.
    #[test]
    fn query_legends_returns_named_only() {
        let mut graph = SagaGraph::default();
        // Create two entities via events.
        for sim_id in [1u64, 2u64] {
            for tick in 0..5u64 {
                let raw = RawSimEvent::new(tick, EventKind::Battle, SourceCrate::Tactics, 1.0)
                    .with_participant(SourceCrate::Tactics, SimRuntimeId(sim_id), Role::Aggressor);
                graph.ingest(raw);
            }
        }
        let eid1 = graph
            .entity_for_sim(SourceCrate::Tactics, SimRuntimeId(1))
            .expect("entity 1 should exist");
        let eid2 = graph
            .entity_for_sim(SourceCrate::Tactics, SimRuntimeId(2))
            .expect("entity 2 should exist");

        // Promote only entity 1.
        graph
            .promote_to_legend(eid1, "Founder of the First Age".to_string(), Role::Founder)
            .expect("promote should succeed");

        let result = graph.query_named_legends();
        assert_eq!(result.named_entities.len(), 1, "only one named legend");
        assert_eq!(result.named_entities[0].entity_id, eid1);
        // Entity 2 must not appear.
        assert!(
            result.named_entities.iter().all(|s| s.entity_id != eid2),
            "unnamed entity must not appear in query_named_legends"
        );
    }
}

impl SagaGraph {
    /// Mark a promoted/extant entity as deceased at `epoch` (feeds `fallen` in digests
    /// and the inspector's death epoch). Idempotent; no-op for unknown ids.
    pub fn mark_died(&mut self, entity: LegendEntityId, epoch: Epoch) {
        if let Some(idx) = self.entity_idx(entity) {
            if let crate::model::LegendNode::Entity(e) = &mut self.g[idx] {
                if e.died_epoch.is_none() {
                    e.died_epoch = Some(epoch);
                }
            }
        }
    }

    /// Attach a name (from the ai-rnd namer) to a promoted entity (§4.3, §7).
    pub fn set_name(&mut self, entity: LegendEntityId, name: NameRef) {
        if let Some(idx) = self.entity_idx(entity) {
            if let crate::model::LegendNode::Entity(e) = &mut self.g[idx] {
                e.name = Some(name);
            }
        }
    }
}
