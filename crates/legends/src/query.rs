//! Read-only query API for the inspector + AI narrator (spec §6).
//!
//! All queries are synchronous, allocation-light, and O(neighborhood) — never
//! O(graph). They read the in-memory [`SagaGraph`]; the worker holds the write lock.

use std::collections::{BTreeSet, HashSet, VecDeque};
use std::ops::Range;

use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};

use crate::graph::SagaGraph;
use crate::ids::*;
use crate::model::*;

/// A compact reference to an entity (returned by `significant`, digests, sagas).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRef {
    pub id: LegendEntityId,
    pub kind: EntityKind,
    pub name: Option<NameRef>,
    pub significance: f32,
    pub promoted: bool,
}

impl EntityRef {
    fn of(e: &EntityNode) -> Self {
        EntityRef {
            id: e.id,
            kind: e.kind,
            name: e.name,
            significance: e.significance,
            promoted: e.promoted,
        }
    }
}

/// An entity's full sub-saga (spec §6 `saga_of`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Saga {
    pub entity: EntityRef,
    /// The entity's own events, chronological.
    pub events: Vec<EventNode>,
    /// Related entities (descendants, cluster, rivals) for clickable chips.
    pub related: Vec<EntityRef>,
}

/// A breadth-bounded causal DAG ("why did this happen" / "what did it lead to").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalDag {
    pub root: LegendEventId,
    pub events: Vec<EventNode>,
    /// (from, to, confidence) `CausedBy` edges within the walked neighborhood.
    pub edges: Vec<(LegendEventId, LegendEventId, f32)>,
}

/// One headline event in an epoch digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigestEvent {
    pub id: LegendEventId,
    pub kind: EventKind,
    pub magnitude: f32,
    pub region: Option<RegionId>,
    pub participants: Vec<LegendEntityId>,
}

/// The narrator contract — a compact, hashable bucket of an epoch (spec §6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpochDigest {
    pub epoch: Epoch,
    pub region: Option<RegionId>,
    pub headline_events: Vec<DigestEvent>,
    pub risen: Vec<EntityRef>,
    pub fallen: Vec<EntityRef>,
    pub causal_notes: Vec<(LegendEventId, LegendEventId, f32)>,
    /// blake3 over the (sorted, capped) body → SLM prose cache key (ai-rnd §4.2).
    pub digest_hash: [u8; 32],
}

const HEADLINE_CAP: usize = 20;

/// The query API surface from `docs/design/legends-engine.md` §6
/// (FR-CIV-LEGENDS-005). Bump this constant on any breaking signature
/// change so consumers can pin their compatibility.
pub const QUERY_API_VERSION: u32 = 1;

impl SagaGraph {
    /// The entity's full sub-saga: its events (chronological) + related entities (§6).
    ///
    /// `saga_of` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`** (saga-graph
    /// ingest stays compatible with `docs/design/legends-engine.md` query API).
    pub fn saga_of(&self, entity: LegendEntityId) -> Option<Saga> {
        let e = self.entity(entity)?;
        let idx = self.entity_idx(entity)?;
        let mut events: Vec<EventNode> = self
            .graph()
            .edges_directed(idx, Direction::Outgoing)
            .filter_map(|edge| self.graph()[edge.target()].as_event().cloned())
            .collect();
        events.sort_by_key(|ev| (ev.epoch.0, ev.id.0));

        let related: Vec<EntityRef> = self
            .graph()
            .neighbors_undirected(idx)
            .filter_map(|n| self.graph()[n].as_entity().map(EntityRef::of))
            .collect();

        Some(Saga {
            entity: EntityRef::of(e),
            events,
            related,
        })
    }

    /// Events touching `entity` within `epochs`, epoch-ordered (spec §6 `timeline`).
    ///
    /// `timeline` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`** (saga-graph
    /// ingest stays compatible with `docs/design/legends-engine.md` query API).
    pub fn timeline(&self, entity: LegendEntityId, epochs: Range<Epoch>) -> Vec<EventNode> {
        let Some(idx) = self.entity_idx(entity) else {
            return Vec::new();
        };
        let mut events: Vec<EventNode> = self
            .graph()
            .edges_directed(idx, Direction::Outgoing)
            .filter_map(|edge| self.graph()[edge.target()].as_event().cloned())
            .filter(|ev| ev.epoch >= epochs.start && ev.epoch < epochs.end)
            .collect();
        events.sort_by_key(|ev| (ev.epoch.0, ev.id.0));
        events
    }

    /// "Why did this happen" — walk `CausedBy` predecessors, breadth-bounded (§6).
    ///
    /// `causal_chain` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`** (saga-graph
    /// ingest stays compatible with `docs/design/legends-engine.md` query API).
    pub fn causal_chain(&self, event: LegendEventId, max_depth: usize) -> Option<CausalDag> {
        self.walk_causal(event, max_depth, true)
    }

    /// "What did this lead to" — walk `CausedBy` successors, breadth-bounded (§6).
    ///
    /// `forward_chain` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`**
    /// (saga-graph ingest stays compatible with `docs/design/legends-engine.md`
    /// query API).
    pub fn forward_chain(&self, event: LegendEventId, max_depth: usize) -> Option<CausalDag> {
        self.walk_causal(event, max_depth, false)
    }

    fn walk_causal(
        &self,
        event: LegendEventId,
        max_depth: usize,
        backward: bool,
    ) -> Option<CausalDag> {
        let start = self.event_idx(event)?;
        let mut seen: HashSet<NodeIndex> = HashSet::new();
        let mut order: Vec<NodeIndex> = Vec::new();
        let mut edges: Vec<(LegendEventId, LegendEventId, f32)> = Vec::new();
        let mut q: VecDeque<(NodeIndex, usize)> = VecDeque::new();
        q.push_back((start, 0));
        seen.insert(start);

        while let Some((idx, depth)) = q.pop_front() {
            order.push(idx);
            if depth >= max_depth {
                continue;
            }
            // CausedBy edges point effect → cause. Backward walk = out-edges;
            // forward walk = in-edges.
            let dir = if backward {
                Direction::Outgoing
            } else {
                Direction::Incoming
            };
            for edge in self.graph().edges_directed(idx, dir) {
                if let LegendEdge::CausedBy { confidence } = edge.weight() {
                    let (effect_idx, cause_idx) = if backward {
                        (idx, edge.target())
                    } else {
                        (edge.source(), idx)
                    };
                    if let (Some(eff), Some(cau)) = (
                        self.graph()[effect_idx].as_event(),
                        self.graph()[cause_idx].as_event(),
                    ) {
                        edges.push((eff.id, cau.id, *confidence));
                    }
                    let next = if backward {
                        edge.target()
                    } else {
                        edge.source()
                    };
                    if seen.insert(next) {
                        q.push_back((next, depth + 1));
                    }
                }
            }
        }

        let events: Vec<EventNode> = order
            .iter()
            .filter_map(|i| self.graph()[*i].as_event().cloned())
            .collect();
        Some(CausalDag {
            root: event,
            events,
            edges,
        })
    }

    /// Current top-N entities by significance, optionally filtered by kind (§6).
    /// O(top_n) via the significance side-set.
    ///
    /// `significant` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`**
    /// (saga-graph ingest stays compatible with `docs/design/legends-engine.md`
    /// query API).
    pub fn significant(&self, top_n: usize, filter: Option<EntityKind>) -> Vec<EntityRef> {
        let mut out = Vec::with_capacity(top_n);
        for id in self.significant_desc() {
            if out.len() >= top_n {
                break;
            }
            if let Some(e) = self.entity(id) {
                if filter.map(|k| k == e.kind).unwrap_or(true) {
                    out.push(EntityRef::of(e));
                }
            }
        }
        out
    }

    /// Generic graph step for the browser (spec §6 `neighbors`).
    ///
    /// `neighbors` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`**
    /// (saga-graph ingest stays compatible with `docs/design/legends-engine.md`
    /// query API).
    pub fn neighbors(&self, entity: LegendEntityId) -> Vec<EntityRef> {
        let Some(idx) = self.entity_idx(entity) else {
            return Vec::new();
        };
        self.graph()
            .neighbors_undirected(idx)
            .filter_map(|n| self.graph()[n].as_entity().map(EntityRef::of))
            .collect()
    }

    /// Compact, hashable digest of an epoch — the SLM narrator's input (spec §6).
    /// Deterministic given graph state (sorted + capped) so the same epoch hashes
    /// the same across reloads (AC-Q-2), without requiring sim determinism.
    ///
    /// `epoch_digest` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`**
    /// (saga-graph ingest stays compatible with `docs/design/legends-engine.md`
    /// query API).
    pub fn epoch_digest(&self, epoch: Epoch, region: Option<RegionId>) -> EpochDigest {
        // gather this epoch's events (optionally region-scoped)
        let region_filter: Option<BTreeSet<LegendEventId>> =
            region.and_then(|r| self.region_events(r).map(|v| v.iter().copied().collect()));
        let mut events: Vec<&EventNode> = self
            .epoch_buckets
            .get(&epoch)
            .map(|ids| {
                ids.iter()
                    .filter(|id| {
                        region_filter
                            .as_ref()
                            .map(|set| set.contains(id))
                            .unwrap_or(true)
                    })
                    .filter_map(|id| self.event(*id))
                    .collect()
            })
            .unwrap_or_default();

        // headline = top events by magnitude, deterministically tie-broken by id, capped
        events.sort_by(|a, b| {
            b.magnitude
                .total_cmp(&a.magnitude)
                .then(a.id.0.cmp(&b.id.0))
        });
        let headline_events: Vec<DigestEvent> = events
            .iter()
            .take(HEADLINE_CAP)
            .map(|e| DigestEvent {
                id: e.id,
                kind: e.kind.clone(),
                magnitude: e.magnitude,
                region: e.region,
                participants: e.participants.to_vec(),
            })
            .collect();

        // risen = promoted this epoch (their Promotion event), fallen = died this epoch
        let mut risen: Vec<EntityRef> = Vec::new();
        for e in &events {
            if e.kind == EventKind::Promotion {
                if let Some(&pid) = e.participants.first() {
                    if let Some(ent) = self.entity(pid) {
                        risen.push(EntityRef::of(ent));
                    }
                }
            }
        }
        risen.sort_by_key(|r| r.id.0);
        risen.dedup_by_key(|r| r.id.0);

        let mut fallen: Vec<EntityRef> = self
            .graph()
            .node_indices()
            .filter_map(|i| self.graph()[i].as_entity())
            .filter(|e| e.died_epoch == Some(epoch))
            .map(EntityRef::of)
            .collect();
        fallen.sort_by_key(|r| r.id.0);

        // causal notes = high-confidence cause→effect pairs among this epoch's events
        let mut causal_notes: Vec<(LegendEventId, LegendEventId, f32)> = Vec::new();
        for e in &events {
            if let Some(idx) = self.event_idx(e.id) {
                for edge in self.graph().edges_directed(idx, Direction::Outgoing) {
                    if let LegendEdge::CausedBy { confidence } = edge.weight() {
                        if let Some(cause) = self.graph()[edge.target()].as_event() {
                            causal_notes.push((e.id, cause.id, *confidence));
                        }
                    }
                }
            }
        }
        causal_notes.sort_by(|a, b| (a.0 .0, a.1 .0).cmp(&(b.0 .0, b.1 .0)));

        let digest_hash = hash_digest(
            epoch,
            region,
            &headline_events,
            &risen,
            &fallen,
            &causal_notes,
        );
        EpochDigest {
            epoch,
            region,
            headline_events,
            risen,
            fallen,
            causal_notes,
            digest_hash,
        }
    }
}

fn hash_digest(
    epoch: Epoch,
    region: Option<RegionId>,
    headlines: &[DigestEvent],
    risen: &[EntityRef],
    fallen: &[EntityRef],
    notes: &[(LegendEventId, LegendEventId, f32)],
) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(&epoch.0.to_le_bytes());
    h.update(&region.map(|r| r.0).unwrap_or(u64::MAX).to_le_bytes());
    for e in headlines {
        h.update(&e.id.0.to_le_bytes());
        h.update(e.kind.label().as_bytes());
        h.update(&((e.magnitude * 1000.0) as u64).to_le_bytes());
    }
    for r in risen {
        h.update(b"R");
        h.update(&r.id.0.to_le_bytes());
    }
    for f in fallen {
        h.update(b"F");
        h.update(&f.id.0.to_le_bytes());
    }
    for (a, b, c) in notes {
        h.update(&a.0.to_le_bytes());
        h.update(&b.0.to_le_bytes());
        h.update(&((c * 1000.0) as u64).to_le_bytes());
    }
    *h.finalize().as_bytes()
}
