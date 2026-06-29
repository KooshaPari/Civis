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
pub mod decay;
pub mod graph;
pub mod ids;
pub mod model;
pub mod query;
pub mod rumor;
pub mod worker;

mod chronicle;
pub use chronicle::{Chronicle, ChronicleEntry, ChronicleEventKind};

mod myth_formation;
pub use myth_formation::{Myth, MythFormationConfig, MythId, MythIndex};

pub use config::LegendsConfig;
pub use decay::{DecayConfig, Prominence, ProminenceTracker};
pub use graph::{AggregateKey, EmptySagaReason, GapReport, IngestOutcome, SagaGraph};
pub use ids::{
    ClusterId, Epoch, LegendEntityId, LegendEventId, NameRef, Provenance, RawEventRef, RegionId,
    SimRef, SimRuntimeId, SourceCrate,
};
pub use model::{
    compute_legend_importance, summary_key, EntityKind, EntityNode, EventKind, EventNode,
    HistoricalEvent, LegendEdge, LegendEntry, LegendNode, RawSimEvent, Role, Tag,
};
pub use query::{CausalDag, DigestEvent, EntityRef, EpochDigest, Saga, QUERY_API_VERSION};
pub use rumor::{
    register_render, render, retell, witness, Chronicle, ChronicleEntry, DefaultNameResolver,
    HistorianMind, NameResolver, Ocean, Register, Rumor, RumorMill,
};
pub use worker::LegendsWorker;

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
