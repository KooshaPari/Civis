//! Stable identifiers for the saga graph (spec §3.1).
//!
//! These are permanent across the whole game + saves and are NOT the same as a
//! sim runtime id (agents recycle slots). The entity-resolution map
//! ([`crate::graph::SagaGraph`]) bridges `(SourceCrate, SimRuntimeId)` →
//! [`LegendEntityId`].

use serde::{Deserialize, Serialize};

/// Stable, permanent id of a legend entity (survives sim-id recycling + saves).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LegendEntityId(pub u64);

/// Append-only, monotonic id of a legend event. Never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LegendEventId(pub u64);

/// Coarse game-time bucket — the unit the narrator and pre-sim work in.
/// Derived from `sim_tick / ticks_per_epoch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Epoch(pub u64);

/// Spatial region id (reuses the in-tree chunk/Voronoi spatial index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RegionId(pub u64);

/// Opaque id of an emergent polity cluster. Membership is emergent overlap;
/// the engine stores the id + provenance only, never an authored faction enum
/// (charter constraint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClusterId(pub u64);

/// Handle into the ai-rnd namer's name store. Entity may be unnamed until promoted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NameRef(pub u64);

/// A sim runtime id (recycled across the sim lifetime) — only meaningful paired
/// with the [`SourceCrate`] that minted it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SimRuntimeId(pub u64);

/// Back-pointer the inspector uses to pull live components for a legend entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimRef {
    pub source: SourceCrate,
    pub sim_id: SimRuntimeId,
}

/// Provenance of an event — which producer crate emitted it (for the loud-gap check).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceCrate {
    Agents,
    Tactics,
    Economy,
    Engine,
    Genetics,
    Planet,
    Protocol3d,
    Research,
    GodTools,
    PreSim,
}

/// Whether an event is from lived play or far-LOD zero-player pre-sim backstory (§4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Provenance {
    #[default]
    Lived,
    PreSim,
}

/// Pointer into the `.civreplay` stream for drill-down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RawEventRef {
    pub tick: u64,
    pub seq: u64,
}
