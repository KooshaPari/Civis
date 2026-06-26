//! Tunable thresholds + weights (spec §5, NFR-CIV-LEGENDS-CONFIG-04).
//!
//! No value here is a charter *outcome*; they tune *what counts as notable*, not
//! *what happens*. All fields are `.env`-overridable so nothing is hardcoded.

use crate::ids::Epoch;
use crate::model::EventKind;

/// Significance / causal / pruning configuration.
#[derive(Debug, Clone)]
pub struct LegendsConfig {
    /// `significance >= this` ⇒ entity is promoted to historically significant.
    pub promotion_threshold: f32,
    /// Per-epoch exponential decay factor for the rolling significance accumulator.
    pub decay: f32,
    /// A non-promoted entity at/below this score with no link to a promoted entity is pruned.
    pub prune_floor: f32,
    /// Upper bound on retained graph nodes (NFR-SCALE-02).
    pub max_graph_nodes: usize,
    /// Recency window (epochs) within which a prior event is a causal candidate.
    pub causal_window_epochs: u64,
    /// Minimum confidence for a `CausedBy` edge to be attached.
    pub causal_min_confidence: f32,
    /// Cap on `CausedBy` edges per event (keeps the DAG sparse + "why" short).
    pub max_causes_per_event: usize,
    /// Causal scoring weights: (shared-participant, proximity, affinity).
    pub causal_weights: (f32, f32, f32),
    /// Sim ticks per coarse epoch bucket.
    pub ticks_per_epoch: u64,
    /// A required producer silent for longer than this (epochs) triggers a loud gap.
    pub gap_epochs: u64,
}

impl Default for LegendsConfig {
    fn default() -> Self {
        LegendsConfig {
            promotion_threshold: 1.0,
            decay: 0.9,
            prune_floor: 0.01,
            max_graph_nodes: 200_000,
            causal_window_epochs: 8,
            causal_min_confidence: 0.25,
            max_causes_per_event: 3,
            causal_weights: (0.5, 0.3, 0.2),
            ticks_per_epoch: 64,
            gap_epochs: 4,
        }
    }
}

impl LegendsConfig {
    /// Map a sim tick to its coarse epoch bucket.
    pub fn epoch_of(&self, tick: u64) -> Epoch {
        Epoch(tick / self.ticks_per_epoch.max(1))
    }
}

/// Significance weight for an event kind (spec §5.1). Death/War/Speciation > Sickness.
pub fn kind_weight(kind: &EventKind) -> f32 {
    match kind {
        EventKind::Death
        | EventKind::WarDeclared
        | EventKind::WarEnded
        | EventKind::SpeciationEvent
        | EventKind::Extinction
        | EventKind::Betrayal
        | EventKind::Disaster => 1.0,
        EventKind::Battle | EventKind::Siege | EventKind::SettlementFounded => 0.8,
        EventKind::Famine
        | EventKind::EconomicBoom
        | EventKind::Bust
        | EventKind::CulturalSpeciation
        | EventKind::IdeologyShift
        | EventKind::Discovery
        | EventKind::Treaty
        | EventKind::LawObserved => 0.6,
        EventKind::Birth | EventKind::Migration | EventKind::Raid | EventKind::GodAct => 0.4,
        EventKind::PriceShock | EventKind::Abandoned => 0.3,
        EventKind::Sickness => 0.2,
        EventKind::Promotion => 0.0, // bookkeeping; never re-feeds significance
        EventKind::GreatWork => 0.7,
        EventKind::Plague => 0.9,
        EventKind::Other(_) => 0.5,
    }
}

/// Advisory cause→effect kind-affinity (spec §4.4 #3). The ONLY authored content
/// in the engine; it is *advisory weighting* that ranks candidates the sim already
/// produced — it never creates events. Returns 0..1.
pub fn kind_affinity(cause: &EventKind, effect: &EventKind) -> f32 {
    use EventKind::*;
    match (cause, effect) {
        (Famine, Migration) => 1.0,
        (Disaster, Bust) => 1.0,
        (Disaster, Famine) => 0.9,
        (WarEnded, SettlementFounded) => 0.8,
        (Battle, Death) => 0.8,
        (Siege, Famine) => 0.7,
        (Bust, Migration) => 0.6,
        (WarDeclared, Battle) => 0.9,
        (SpeciationEvent, CulturalSpeciation) => 0.5,
        (Treaty, Betrayal) => 0.8,
        (Plague, Famine) => 0.7,
        (Plague, Migration) => 0.6,
        (GreatWork, IdeologyShift) => 0.5,
        (Betrayal, WarDeclared) => 0.9,
        (Treaty, WarEnded) => 0.7,
        _ => 0.0,
    }
}
