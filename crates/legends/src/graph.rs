//! The saga graph: a `petgraph::StableDiGraph` + side indices, with ingest,
//! entity resolution, significance scoring, promotion, causal linking, and pruning
//! (spec §3.5, §4, §5).
//!
//! `StableDiGraph` is chosen because node indices must survive removal
//! (decayed/merged entities), which a plain `Graph` does not guarantee.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use smallvec::SmallVec;

use crate::config::{kind_affinity, kind_weight, LegendsConfig};
use crate::ids::*;
use crate::model::*;

/// Total-orderable f32 wrapper for the significance side-set.
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedF32(f32);
impl Eq for OrderedF32 {}
impl PartialOrd for OrderedF32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OrderedF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// Outcome of ingesting one [`RawSimEvent`].
#[derive(Debug, Clone, Default)]
pub struct IngestOutcome {
    pub event_id: Option<LegendEventId>,
    /// Entities promoted to historically-significant by this event.
    pub promoted: Vec<LegendEntityId>,
    /// `CausedBy` edges attached (cause, confidence).
    pub causes_linked: usize,
}

/// The saga graph + all side indices (spec §3.5), kept consistent on every mutation.
pub struct SagaGraph {
    pub(crate) g: StableDiGraph<LegendNode, LegendEdge>,
    pub config: LegendsConfig,

    // --- side indices (§3.5) ---
    entity_index: HashMap<LegendEntityId, NodeIndex>,
    event_index: HashMap<LegendEventId, NodeIndex>,
    sim_resolution: HashMap<(SourceCrate, SimRuntimeId), LegendEntityId>,
    /// Aggregate-entity resolution (War/Disaster/PolityCluster) by stable key (§4.2.4).
    aggregate_resolution: HashMap<AggregateKey, LegendEntityId>,
    pub(crate) epoch_buckets: BTreeMap<Epoch, Vec<LegendEventId>>,
    region_buckets: HashMap<RegionId, Vec<LegendEventId>>,
    significant_set: BTreeSet<(OrderedF32, LegendEntityId)>,

    // --- loud-gap tracking (§7) ---
    last_seen_epoch: HashMap<SourceCrate, Epoch>,

    // --- legend tracking (FR-CIV-LEGENDS) ---
    /// Named legend entries auto-generated from significant events.
    /// Keyed by event_id for fast lookup and deduplication.
    pub(crate) legends: HashMap<LegendEventId, LegendEntry>,

    // --- id allocators ---
    next_entity: u64,
    next_event: u64,
    cur_epoch: Epoch,
}

/// Stable aggregate-resolution key so repeated battles fold into one War node, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AggregateKey {
    pub kind: EntityKind,
    pub a: ClusterId,
    pub b: ClusterId,
    pub start_bucket: u64,
}

/// One loud gap in a producer's event stream (FR-CIV-LEGENDS-006).
#[derive(Debug, Clone, PartialEq)]
pub struct GapReport {
    pub source: SourceCrate,
    pub last_seen_epoch: Epoch,
    pub now_epoch: Epoch,
    /// Human-readable reason surfaced in the UI / logs, e.g.
    /// `"no Agents events for epoch 0..12"`. Format-stable so the inspector
    /// can grep for it.
    pub reason: String,
}

/// Why a saga is empty (FR-CIV-LEGENDS-006). Distinct from "no data" — the
/// UI renders the variant's text, never a silent omission.
#[derive(Debug, Clone, PartialEq)]
pub enum EmptySagaReason {
    /// The `LegendEntityId` was never inserted into the graph.
    UnknownEntity,
    /// The id resolved to an event node, not an entity (caller bug).
    NotAnEntity,
    /// Entity is in the graph but has no events yet.
    NoEventsYet,
    /// The producer that registered this entity is currently silent.
    ProducerGap { source: SourceCrate, reason: String },
}

impl EmptySagaReason {
    pub fn reason_text(&self) -> String {
        match self {
            EmptySagaReason::UnknownEntity => "entity not in saga graph".to_string(),
            EmptySagaReason::NotAnEntity => "id resolves to an event, not an entity".to_string(),
            EmptySagaReason::NoEventsYet => "entity has no witnessed events yet".to_string(),
            EmptySagaReason::ProducerGap { reason, .. } => reason.clone(),
        }
    }
}

impl Default for SagaGraph {
    fn default() -> Self {
        Self::new(LegendsConfig::default())
    }
}

impl SagaGraph {
    pub fn new(config: LegendsConfig) -> Self {
        SagaGraph {
            g: StableDiGraph::new(),
            config,
            entity_index: HashMap::new(),
            event_index: HashMap::new(),
            sim_resolution: HashMap::new(),
            aggregate_resolution: HashMap::new(),
            epoch_buckets: BTreeMap::new(),
            region_buckets: HashMap::new(),
            significant_set: BTreeSet::new(),
            last_seen_epoch: HashMap::new(),
            legends: HashMap::new(),
            next_entity: 1,
            next_event: 1,
            cur_epoch: Epoch(0),
        }
    }

    pub fn node_count(&self) -> usize {
        self.g.node_count()
    }
    pub fn edge_count(&self) -> usize {
        self.g.edge_count()
    }

    // ---- lookups (used by the query API) ----

    pub(crate) fn entity_idx(&self, id: LegendEntityId) -> Option<NodeIndex> {
        self.entity_index.get(&id).copied()
    }
    pub(crate) fn event_idx(&self, id: LegendEventId) -> Option<NodeIndex> {
        self.event_index.get(&id).copied()
    }
    pub fn entity(&self, id: LegendEntityId) -> Option<&EntityNode> {
        self.entity_idx(id).and_then(|i| self.g[i].as_entity())
    }
    pub fn event(&self, id: LegendEventId) -> Option<&EventNode> {
        self.event_idx(id).and_then(|i| self.g[i].as_event())
    }

    /// Resolution bridge the inspector uses on a click (spec §6 `entity_for_sim`).
    ///
    /// `entity_for_sim` — spec §6 query API. **`Covers FR-CIV-LEGENDS-005`**
    /// (saga-graph ingest stays compatible with `docs/design/legends-engine.md`
    /// query API).
    pub fn entity_for_sim(
        &self,
        source: SourceCrate,
        sim_id: SimRuntimeId,
    ) -> Option<LegendEntityId> {
        self.sim_resolution.get(&(source, sim_id)).copied()
    }

    // ---- entity resolution (§4.2) ----

    fn mint_entity_id(&mut self) -> LegendEntityId {
        let id = LegendEntityId(self.next_entity);
        self.next_entity += 1;
        id
    }

    /// Resolve `(source, sim_id)` to a stable entity id, minting a provisional
    /// node on a miss (§4.2). `kind`/`region` describe a freshly-minted entity.
    fn resolve_entity(
        &mut self,
        source: SourceCrate,
        sim_id: SimRuntimeId,
        kind: EntityKind,
        region: Option<RegionId>,
        epoch: Epoch,
    ) -> LegendEntityId {
        if let Some(id) = self.sim_resolution.get(&(source, sim_id)) {
            return *id;
        }
        let id = self.mint_entity_id();
        self.sim_resolution.insert((source, sim_id), id);
        self.insert_entity(EntityNode {
            id,
            kind,
            name: None,
            born_epoch: epoch,
            died_epoch: None,
            significance: 0.0,
            promoted: false,
            home_region: region,
            cluster: None,
            sim_ref: Some(SimRef { source, sim_id }),
            tags: SmallVec::new(),
        });
        id
    }

    /// Resolve (or fold into) an aggregate entity (War/Disaster/PolityCluster) by
    /// stable aggregate key (§4.2.4).
    pub fn resolve_aggregate(&mut self, key: AggregateKey, epoch: Epoch) -> LegendEntityId {
        if let Some(id) = self.aggregate_resolution.get(&key) {
            return *id;
        }
        let id = self.mint_entity_id();
        let kind = key.kind;
        self.aggregate_resolution.insert(key, id);
        self.insert_entity(EntityNode {
            id,
            kind,
            name: None,
            born_epoch: epoch,
            died_epoch: None,
            significance: 0.0,
            promoted: false,
            home_region: None,
            cluster: None,
            sim_ref: None,
            tags: SmallVec::new(),
        });
        id
    }

    fn insert_entity(&mut self, node: EntityNode) -> NodeIndex {
        let id = node.id;
        let score = node.significance;
        let idx = self.g.add_node(LegendNode::Entity(node));
        self.entity_index.insert(id, idx);
        self.significant_set.insert((OrderedF32(score), id));
        idx
    }

    // ---- significance + promotion (§5, §4.3) ----

    fn entity_mut(&mut self, id: LegendEntityId) -> Option<&mut EntityNode> {
        let idx = *self.entity_index.get(&id)?;
        match &mut self.g[idx] {
            LegendNode::Entity(e) => Some(e),
            _ => None,
        }
    }

    /// Apply one event's significance contribution to a participant and re-promote
    /// if it crosses the threshold (spec §5.1, §4.3). Returns `true` on a *new* promotion.
    fn bump_significance(&mut self, id: LegendEntityId, delta: f32) -> bool {
        let threshold = self.config.promotion_threshold;
        let (old_score, new_score, newly_promoted) = {
            let Some(e) = self.entity_mut(id) else {
                return false;
            };
            let old = e.significance;
            e.significance += delta;
            let newly = !e.promoted && e.significance >= threshold;
            if newly {
                e.promoted = true;
            }
            (old, e.significance, newly)
        };
        // keep the ordered side-set consistent
        self.significant_set.remove(&(OrderedF32(old_score), id));
        self.significant_set.insert((OrderedF32(new_score), id));
        newly_promoted
    }

    /// Per-epoch exponential decay over all entities (spec §5.2). Keeps the
    /// "significant now" ranking fresh; `promoted` stays monotonic.
    pub fn decay_epoch(&mut self) {
        let decay = self.config.decay;
        let ids: Vec<LegendEntityId> = self.entity_index.keys().copied().collect();
        for id in ids {
            let (old, new) = {
                let Some(e) = self.entity_mut(id) else {
                    continue;
                };
                let old = e.significance;
                e.significance *= decay;
                (old, e.significance)
            };
            self.significant_set.remove(&(OrderedF32(old), id));
            self.significant_set.insert((OrderedF32(new), id));
        }
    }

    /// Garbage-collect provisional (`!promoted`) entities below `prune_floor` that
    /// have no edge to a promoted entity (spec §5.3). Promoted entities and any
    /// entity reaching a promoted entity are never pruned. Returns count pruned.
    pub fn prune(&mut self) -> usize {
        let floor = self.config.prune_floor;
        let candidates: Vec<(LegendEntityId, NodeIndex)> = self
            .entity_index
            .iter()
            .filter_map(|(id, idx)| match &self.g[*idx] {
                LegendNode::Entity(e) if !e.promoted && e.significance <= floor => {
                    Some((*id, *idx))
                }
                _ => None,
            })
            .collect();

        let mut pruned = 0;
        for (id, idx) in candidates {
            if self.touches_promoted(idx) {
                continue;
            }
            let score = self
                .g
                .node_weight(idx)
                .and_then(|n| n.as_entity())
                .map(|e| e.significance)
                .unwrap_or(0.0);
            self.g.remove_node(idx);
            self.entity_index.remove(&id);
            self.significant_set.remove(&(OrderedF32(score), id));
            self.sim_resolution.retain(|_, v| *v != id);
            pruned += 1;
        }
        if pruned > 0 {
            tracing::debug!("legends: pruned {pruned} provisional entities this epoch");
        }
        pruned
    }

    /// True if any neighbor (1 hop) of `idx` is a promoted entity.
    fn touches_promoted(&self, idx: NodeIndex) -> bool {
        self.g
            .neighbors_undirected(idx)
            .any(|n| matches!(&self.g[n], LegendNode::Entity(e) if e.promoted))
    }

    // ---- ingest (the core pipeline, §4) ----

    /// Ingest one raw event off the bus: normalize → resolve participants → insert
    /// event node → score significance + promote → causal link (spec §4). This is the
    /// single mutation entry point; it keeps every side index consistent.
    pub fn ingest(&mut self, raw: RawSimEvent) -> IngestOutcome {
        let epoch = self.config.epoch_of(raw.tick);
        self.cur_epoch = epoch.max(self.cur_epoch);
        self.last_seen_epoch.insert(raw.source, epoch);

        // 1. resolve participants → stable entity ids
        let mut participants: SmallVec<[LegendEntityId; 4]> = SmallVec::new();
        let mut roles: SmallVec<[(LegendEntityId, Role); 4]> = SmallVec::new();
        for (src, sim_id, role) in raw.participants.iter().copied() {
            let kind = entity_kind_for(role);
            let eid = self.resolve_entity(src, sim_id, kind, raw.region, epoch);
            participants.push(eid);
            roles.push((eid, role));
        }

        // 2. insert the event node
        let magnitude = raw.raw_magnitude.clamp(0.0, 1.0);
        let event_id = LegendEventId(self.next_event);
        self.next_event += 1;
        let summary = summary_key(&raw.kind, &participants, magnitude, epoch);
        let ev = EventNode {
            id: event_id,
            epoch,
            region: raw.region,
            kind: raw.kind.clone(),
            magnitude,
            participants: participants.clone(),
            summary_key: summary,
            source_crate: raw.source,
            provenance: raw.provenance,
            raw_ref: raw.raw_ref,
        };
        let ev_idx = self.g.add_node(LegendNode::Event(ev));
        self.event_index.insert(event_id, ev_idx);
        self.epoch_buckets.entry(epoch).or_default().push(event_id);
        if let Some(r) = raw.region {
            self.region_buckets.entry(r).or_default().push(event_id);
        }

        // 3. participation edges + significance bumps
        let mut promoted = Vec::new();
        for (eid, role) in roles.iter().copied() {
            if let Some(pidx) = self.entity_index.get(&eid).copied() {
                self.g
                    .add_edge(pidx, ev_idx, LegendEdge::ParticipatedIn { role });
            }
            let reach = self.reach(eid);
            let delta = magnitude * role.weight() * kind_weight(&raw.kind) * reach;
            if self.bump_significance(eid, delta) {
                promoted.push(eid);
            }
        }

        // 4. causal linking (§4.4)
        let causes_linked = self.link_causes(ev_idx, event_id, epoch, &participants, &raw.kind);

        // 5. emit a Promotion event for each newly-promoted entity (§4.3)
        for &eid in &promoted {
            self.emit_promotion(eid, epoch, raw.tick);
        }

        IngestOutcome {
            event_id: Some(event_id),
            promoted,
            causes_linked,
        }
    }

    /// reach(e) = log-scaled neighborhood influence (kinship + cluster + territory).
    fn reach(&self, id: LegendEntityId) -> f32 {
        let Some(idx) = self.entity_index.get(&id) else {
            return 1.0;
        };
        let degree = self.g.neighbors_undirected(*idx).count() as f32;
        1.0 + (1.0 + degree).ln()
    }

    /// Emit a `Promotion` bookkeeping event so "X rose to prominence" is itself part
    /// of the saga (§4.3). Does not re-feed significance (`kind_weight(Promotion)=0`).
    fn emit_promotion(&mut self, eid: LegendEntityId, epoch: Epoch, _tick: u64) {
        let event_id = LegendEventId(self.next_event);
        self.next_event += 1;
        let parts: SmallVec<[LegendEntityId; 4]> = std::iter::once(eid).collect();
        let summary = summary_key(&EventKind::Promotion, &parts, 0.0, epoch);
        let ev = EventNode {
            id: event_id,
            epoch,
            region: self.entity(eid).and_then(|e| e.home_region),
            kind: EventKind::Promotion,
            magnitude: 0.0,
            participants: parts,
            summary_key: summary,
            source_crate: SourceCrate::Engine,
            provenance: Provenance::Lived,
            raw_ref: None,
        };
        let ev_idx = self.g.add_node(LegendNode::Event(ev));
        self.event_index.insert(event_id, ev_idx);
        self.epoch_buckets.entry(epoch).or_default().push(event_id);
        if let Some(pidx) = self.entity_index.get(&eid).copied() {
            self.g.add_edge(
                pidx,
                ev_idx,
                LegendEdge::ParticipatedIn { role: Role::Leader },
            );
        }
    }

    /// Heuristic causal linking (spec §4.4). Proposes `CausedBy` edges to prior
    /// events using shared-participant + spatio-temporal proximity + kind-affinity,
    /// confidence-scored, capped, and acyclicity-guarded. Returns edges attached.
    fn link_causes(
        &mut self,
        ev_idx: NodeIndex,
        event_id: LegendEventId,
        epoch: Epoch,
        participants: &[LegendEntityId],
        kind: &EventKind,
    ) -> usize {
        let (w_shared, w_prox, w_aff) = self.config.causal_weights;
        let window = self.config.causal_window_epochs;
        let pset: HashSet<LegendEntityId> = participants.iter().copied().collect();
        let region = self.event(event_id).and_then(|e| e.region);

        // gather candidate prior events within the recency window
        let lo = Epoch(epoch.0.saturating_sub(window));
        let mut candidates: Vec<(LegendEventId, f32)> = Vec::new();
        for (&e_epoch, ids) in self.epoch_buckets.range(lo..=epoch) {
            for &cand_id in ids {
                if cand_id == event_id {
                    continue;
                }
                let Some(cand) = self.event(cand_id) else {
                    continue;
                };
                // acyclicity guard: cause must be strictly earlier in epoch (§4.4.5)
                if cand.epoch.0 >= epoch.0 {
                    continue;
                }
                let shared = cand
                    .participants
                    .iter()
                    .filter(|p| pset.contains(p))
                    .count() as f32;
                let shared_s = if shared > 0.0 { 1.0 } else { 0.0 };
                let prox_s = match (cand.region, region) {
                    (Some(a), Some(b)) if a == b => 1.0,
                    (Some(_), Some(_)) => 0.3,
                    _ => 0.0,
                } * recency(e_epoch, epoch, window);
                let aff_s = kind_affinity(&cand.kind, kind);
                let confidence = w_shared * shared_s + w_prox * prox_s + w_aff * aff_s;
                if confidence >= self.config.causal_min_confidence {
                    candidates.push((cand_id, confidence));
                }
            }
        }

        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        candidates.truncate(self.config.max_causes_per_event);
        let mut linked = 0;
        for (cause_id, confidence) in &candidates {
            if let Some(cidx) = self.event_index.get(cause_id).copied() {
                // CausedBy points effect → cause (the "why?" walk follows out-edges).
                self.g.add_edge(
                    ev_idx,
                    cidx,
                    LegendEdge::CausedBy {
                        confidence: *confidence,
                    },
                );
                linked += 1;
            }
        }

        // Succeeded spine: link to the immediately-prior event sharing the dominant
        // participant, regardless of causal confidence (§4.4).
        if let Some(&dominant) = participants.first() {
            if let Some(prev) = self.prev_event_with_participant(dominant, event_id) {
                if let (Some(a), Some(b)) = (self.event_index.get(&prev).copied(), Some(ev_idx)) {
                    self.g.add_edge(b, a, LegendEdge::Succeeded);
                }
            }
        }

        linked
    }

    /// Most recent prior event (other than `exclude`) touching `participant`.
    fn prev_event_with_participant(
        &self,
        participant: LegendEntityId,
        exclude: LegendEventId,
    ) -> Option<LegendEventId> {
        let pidx = self.entity_index.get(&participant)?;
        self.g
            .edges_directed(*pidx, Direction::Outgoing)
            .filter_map(|e| {
                let target = e.target();
                self.g[target].as_event().map(|ev| ev.id)
            })
            .filter(|id| *id != exclude)
            .max_by_key(|id| id.0)
    }

    // ---- loud-gap detection (§7) ----

    /// Required producers that must keep emitting; silence is a loud gap (§7).
    fn required_producers() -> &'static [SourceCrate] {
        &[
            SourceCrate::Agents,
            SourceCrate::Tactics,
            SourceCrate::Economy,
            SourceCrate::Engine,
            SourceCrate::Genetics,
            SourceCrate::Planet,
        ]
    }

    /// Return required producers silent for more than `gap_epochs` as of `now`
    /// (spec §7 loud-gap detector). The caller logs/render a visible gap marker —
    /// never a silently-empty saga.
    pub fn detect_gaps(&self, now: Epoch) -> Vec<(SourceCrate, Epoch)> {
        let mut gaps = Vec::new();
        for &src in Self::required_producers() {
            let last = self.last_seen_epoch.get(&src).copied().unwrap_or(Epoch(0));
            if now.0.saturating_sub(last.0) > self.config.gap_epochs {
                tracing::warn!(
                    "legends: no {:?} events for epoch {}..{}",
                    src,
                    last.0,
                    now.0
                );
                gaps.push((src, last));
            }
        }
        gaps
    }

    /// Same as [`SagaGraph::detect_gaps`] but typed with a human-readable
    /// reason string per gap (FR-CIV-LEGENDS-006). The empty-saga-with-reason
    /// contract: a `Saga` query on an entity from a silent producer must
    /// surface this reason, never a silent omission.
    pub fn gap_reports(&self, now: Epoch) -> Vec<GapReport> {
        self.detect_gaps(now)
            .into_iter()
            .map(|(src, last)| GapReport {
                source: src,
                last_seen_epoch: last,
                now_epoch: now,
                reason: format!("no {:?} events for epoch {}..{}", src, last.0, now.0),
            })
            .collect()
    }

    /// Reason for a *missing* saga (FR-CIV-LEGENDS-006). Returns
    /// `Some(reason)` when the entity is not in the graph or has no events;
    /// `None` only when the saga was empty because the sim has not produced
    /// a notable event for it yet. The UI / inspector surfaces the reason
    /// string instead of "no data".
    pub fn empty_saga_reason(&self, entity: LegendEntityId) -> Option<EmptySagaReason> {
        let Some(idx) = self.entity_idx(entity) else {
            return Some(EmptySagaReason::UnknownEntity);
        };
        let Some(e) = self.g[idx].as_entity() else {
            return Some(EmptySagaReason::NotAnEntity);
        };
        let events = self
            .g
            .edges_directed(idx, Direction::Outgoing)
            .filter_map(|edge| self.g[edge.target()].as_event())
            .count();
        if events == 0 {
            // Check whether the entity was recently pruned and whether the
            // producer it was registered with is currently silent.
            let src = e.sim_ref.map(|s| s.source);
            let now = self.cur_epoch;
            if let Some(src) = src {
                if self.detect_gaps(now).iter().any(|(s, _)| *s == src) {
                    return Some(EmptySagaReason::ProducerGap {
                        source: src,
                        reason: format!("producer {:?} is silent as of epoch {}", src, now.0),
                    });
                }
            }
            return Some(EmptySagaReason::NoEventsYet);
        }
        None
    }

    pub fn current_epoch(&self) -> Epoch {
        self.cur_epoch
    }

    // ---- internal accessors for the query module ----

    pub(crate) fn graph(&self) -> &StableDiGraph<LegendNode, LegendEdge> {
        &self.g
    }
    pub(crate) fn region_events(&self, region: RegionId) -> Option<&Vec<LegendEventId>> {
        self.region_buckets.get(&region)
    }
    pub(crate) fn significant_desc(&self) -> impl Iterator<Item = LegendEntityId> + '_ {
        self.significant_set.iter().rev().map(|(_, id)| *id)
    }

    /// Auto-generate a legend from a significant emergent event (FR-CIV-LEGENDS).
    /// Records the event with provenance (who/where/when-tick/cause) and an importance score.
    /// Only creates a legend if the event is "significant enough" (magnitude + principal significance).
    /// Returns true if the legend was created, false if it already existed or didn't meet threshold.
    pub fn create_legend_from_event(
        &mut self,
        event_id: LegendEventId,
        principal_entity: LegendEntityId,
    ) -> bool {
        // Already have a legend for this event
        if self.legends.contains_key(&event_id) {
            return false;
        }

        // Look up the event
        let Some(event_idx) = self.event_idx(event_id) else {
            return false;
        };
        let Some(event) = self.g[event_idx].as_event() else {
            return false;
        };

        // Look up the principal entity to get its significance
        let principal_significance = self
            .entity_idx(principal_entity)
            .and_then(|idx| self.g[idx].as_entity())
            .map(|e| e.significance)
            .unwrap_or(0.0);

        // Compute importance; only create legend if importance is above a threshold.
        let importance = crate::model::compute_legend_importance(event.magnitude, principal_significance);
        let significance_threshold = 0.3; // Legend creation threshold (tunable config)
        if importance < significance_threshold {
            return false;
        }

        // Create the legend entry
        let legend = LegendEntry::from_event(
            event_id,
            event,
            principal_entity,
            principal_significance,
            event.participants.clone(),
        );
        self.legends.insert(event_id, legend);
        true
    }

    /// Query all legends sorted by importance (descending).
    /// Returns top N legends by importance score.
    pub fn top_legends(&self, limit: usize) -> Vec<&LegendEntry> {
        let mut entries: Vec<_> = self.legends.values().collect();
        entries.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        entries.into_iter().take(limit).collect()
    }

    /// Get a single legend by event id.
    pub fn legend(&self, event_id: LegendEventId) -> Option<&LegendEntry> {
        self.legends.get(&event_id)
    }

    /// Get all legends.
    pub fn all_legends(&self) -> Vec<&LegendEntry> {
        self.legends.values().collect()
    }
}

/// Recency factor in (0,1]: nearer-in-epoch candidates score higher.
fn recency(cand: Epoch, now: Epoch, window: u64) -> f32 {
    let dist = now.0.saturating_sub(cand.0) as f32;
    let w = window.max(1) as f32;
    (1.0 - (dist / (w + 1.0))).clamp(0.0, 1.0)
}

/// Default entity kind to mint for a participant given its role. Coarse; the
/// producer-side kind registry refines this later (kept simple + opaque here).
fn entity_kind_for(role: Role) -> EntityKind {
    match role {
        Role::Founder | Role::Builder => EntityKind::Agent,
        _ => EntityKind::Agent,
    }
}
