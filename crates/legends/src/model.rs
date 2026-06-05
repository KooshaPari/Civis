//! Node / edge / event data model for the saga graph (spec §3.2–§3.4, §4.1).

use crate::ids::*;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// A short data tag shared across structures regardless of author (charter).
pub type Tag = String;

/// Open, producer-owned event taxonomy (spec §3.4). The engine treats this as an
/// opaque key + display label, so adding kinds needs no engine change.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventKind {
    Birth,
    Death,
    Sickness,
    Migration,
    Battle,
    Siege,
    Raid,
    WarDeclared,
    WarEnded,
    EconomicBoom,
    Bust,
    PriceShock,
    Famine,
    IdeologyShift,
    CulturalSpeciation,
    SpeciationEvent,
    Extinction,
    Disaster,
    SettlementFounded,
    Abandoned,
    Discovery,
    LawObserved,
    GodAct,
    /// "X rose to prominence" — emitted by the engine itself on promotion (§4.3).
    Promotion,
    /// Escape hatch so producers can extend the taxonomy without an engine change.
    Other(String),
}

impl EventKind {
    /// Human-readable display label (the only thing the engine knows about a kind).
    pub fn label(&self) -> String {
        match self {
            EventKind::Other(s) => s.clone(),
            other => format!("{other:?}"),
        }
    }
}

/// The kind of an entity node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    Agent,
    Lineage,
    Species,
    Settlement,
    PolityCluster,
    War,
    Disaster,
    Artifact,
    Discovery,
}

/// Participant role in an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Aggressor,
    Defender,
    Victim,
    Leader,
    Founder,
    Builder,
    Witness,
    Cause,
    Effect,
}

impl Role {
    /// Weight a role contributes to significance (spec §5.1). Leader/Founder > Witness.
    pub fn weight(self) -> f32 {
        match self {
            Role::Leader | Role::Founder => 1.0,
            Role::Aggressor | Role::Defender => 0.8,
            Role::Builder | Role::Cause => 0.7,
            Role::Victim | Role::Effect => 0.5,
            Role::Witness => 0.2,
        }
    }
}

/// A graph node: either a historical Entity or an Event (spec §3.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LegendNode {
    Entity(EntityNode),
    Event(EventNode),
}

impl LegendNode {
    pub fn as_entity(&self) -> Option<&EntityNode> {
        match self {
            LegendNode::Entity(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_event(&self) -> Option<&EventNode> {
        match self {
            LegendNode::Event(e) => Some(e),
            _ => None,
        }
    }
}

/// An entity node — a measured-significant participant in history (spec §3.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityNode {
    pub id: LegendEntityId,
    pub kind: EntityKind,
    /// Filled by the ai-rnd namer on promotion. `None` until then.
    pub name: Option<NameRef>,
    pub born_epoch: Epoch,
    /// `None` = still extant.
    pub died_epoch: Option<Epoch>,
    /// 0..1 rolling decayed score (§5); `>= PROMOTION_THRESHOLD` ⇒ significant.
    pub significance: f32,
    /// Crossed the significance threshold at least once (monotonic, §4.3).
    pub promoted: bool,
    pub home_region: Option<RegionId>,
    pub cluster: Option<ClusterId>,
    /// Back-pointer so the inspector can pull live components.
    pub sim_ref: Option<SimRef>,
    pub tags: SmallVec<[Tag; 2]>,
}

/// An event node (spec §3.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventNode {
    pub id: LegendEventId,
    pub epoch: Epoch,
    pub region: Option<RegionId>,
    pub kind: EventKind,
    /// Normalized 0..1 raw impact (feeds significance).
    pub magnitude: f32,
    /// Resolved entity ids (subjects/objects of the event).
    pub participants: SmallVec<[LegendEntityId; 4]>,
    /// blake3 of (kind, participants, magnitude, epoch) → narrator prose cache key.
    pub summary_key: [u8; 32],
    pub source_crate: SourceCrate,
    pub provenance: Provenance,
    pub raw_ref: Option<RawEventRef>,
}

/// Edge types (spec §3.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LegendEdge {
    // event → event (the causal DAG spine)
    /// X happened because Y (heuristic-scored, §4.4).
    CausedBy {
        confidence: f32,
    },
    /// Temporal succession in the same thread (no causality claim).
    Succeeded,
    // entity ↔ event
    /// e.g. Agent A fought in Battle B.
    ParticipatedIn {
        role: Role,
    },
    // entity → entity (relationship spine, lightly held)
    DescendsFrom,
    MemberOf,
    Founded,
    Destroyed,
    Ruled,
    Built,
}

/// Producer contract: the minimal payload emitted onto the `crates/watch` bus
/// (spec §4.1). Producers depend only on this shape, never on the legends crate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawSimEvent {
    pub tick: u64,
    pub region: Option<RegionId>,
    pub kind: EventKind,
    pub source: SourceCrate,
    /// `(source, sim_runtime_id, role)` for each participant.
    pub participants: SmallVec<[(SourceCrate, SimRuntimeId, Role); 4]>,
    /// Crate-local raw impact; the engine normalizes to 0..1.
    pub raw_magnitude: f32,
    pub provenance: Provenance,
    pub raw_ref: Option<RawEventRef>,
}

impl RawSimEvent {
    /// Convenience constructor for a lived event with no spatial region.
    pub fn new(tick: u64, kind: EventKind, source: SourceCrate, raw_magnitude: f32) -> Self {
        RawSimEvent {
            tick,
            region: None,
            kind,
            source,
            participants: SmallVec::new(),
            raw_magnitude,
            provenance: Provenance::Lived,
            raw_ref: None,
        }
    }

    pub fn with_participant(mut self, src: SourceCrate, sim_id: SimRuntimeId, role: Role) -> Self {
        self.participants.push((src, sim_id, role));
        self
    }

    pub fn with_region(mut self, region: RegionId) -> Self {
        self.region = Some(region);
        self
    }
}

/// Compute the stable narrator prose-cache key for an event (spec §3.2 `summary_key`).
pub fn summary_key(
    kind: &EventKind,
    participants: &[LegendEntityId],
    magnitude: f32,
    epoch: Epoch,
) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(kind.label().as_bytes());
    for p in participants {
        h.update(&p.0.to_le_bytes());
    }
    // Quantize magnitude so trivially-different floats still cache-hit.
    h.update(&((magnitude * 1000.0) as u64).to_le_bytes());
    h.update(&epoch.0.to_le_bytes());
    *h.finalize().as_bytes()
}
