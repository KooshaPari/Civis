//! Emergent customary law (FR-LAW).
//!
//! Laws are **not** scripted policy bundles. They crystallize when a faction
//! accumulates repeated disputes of the same kind (theft, violence, resource
//! conflict). Stronger civic institutions later codify and enforce them.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Recurring conflict categories that seed customary law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DisputeKind {
    /// Property / theft pressure.
    Theft,
    /// Interpersonal or military violence.
    Violence,
    /// Commons, food, or trade scarcity disputes.
    ResourceDispute,
}

impl DisputeKind {
    /// Stable slug used on snapshots and replay buses.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            DisputeKind::Theft => "custom/theft_restitution",
            DisputeKind::Violence => "custom/violence_ban",
            DisputeKind::ResourceDispute => "custom/resource_allocation",
        }
    }
}

/// A single emergent customary law for one faction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomaryLaw {
    /// Monotonic id within the faction's law set.
    pub id: u32,
    /// Which recurring dispute produced this norm.
    pub dispute_kind: DisputeKind,
    /// Human-readable slug (e.g. `custom/theft_restitution`).
    pub label: String,
    /// Whether institutions have codified this norm.
    pub codified: bool,
    /// `0` = oral custom only; `1+` = institution enforcement tier.
    pub enforcement_level: u8,
    /// Simulation tick when the norm first emerged.
    pub emerged_at_tick: u64,
}

/// Per-faction dispute tallies and emerged laws.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionLawState {
    /// Running count of disputes per kind (never decremented).
    pub dispute_counts: BTreeMap<DisputeKind, u32>,
    /// Customary laws that have crystallized from dispute pressure.
    pub laws: Vec<CustomaryLaw>,
    /// Next law id allocator for this faction.
    pub next_law_id: u32,
}

/// Disputes required before a customary law emerges for that kind.
pub const DISPUTE_THRESHOLD_FOR_LAW: u32 = 3;

/// Minimum summed institution level before a law is codified.
pub const CODIFICATION_INSTITUTION_STRENGTH: u32 = 2;

/// Snapshot slice surfaced on [`crate::engine::SimulationSnapshot`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionLawSnapshot {
    pub faction_id: u32,
    pub laws: Vec<CustomaryLaw>,
    pub institutional_strength: u32,
    pub dispute_counts: BTreeMap<DisputeKind, u32>,
}

/// Governance-phase event for HUD / replay consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LawEvent {
    pub faction_id: u32,
    pub law_id: u32,
    pub kind: LawEventKind,
    pub dispute_kind: DisputeKind,
    pub tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LawEventKind {
    Emerged,
    Codified,
}

impl FactionLawState {
    /// Record one dispute occurrence for this faction.
    pub fn record_dispute(&mut self, kind: DisputeKind) {
        *self.dispute_counts.entry(kind).or_insert(0) += 1;
    }

    /// Attempt to crystallize new laws and codify existing ones given
    /// institutional strength. Returns events emitted this tick.
    pub fn tick_laws(
        &mut self,
        tick: u64,
        institution_strength: u32,
    ) -> Vec<LawEvent> {
        let mut events = Vec::new();
        self.try_emerge_laws(tick, &mut events);
        self.try_codify_laws(tick, institution_strength, &mut events);
        events
    }

    fn try_emerge_laws(&mut self, tick: u64, events: &mut Vec<LawEvent>) {
        for kind in [
            DisputeKind::Theft,
            DisputeKind::Violence,
            DisputeKind::ResourceDispute,
        ] {
            let count = self.dispute_counts.get(&kind).copied().unwrap_or(0);
            if count < DISPUTE_THRESHOLD_FOR_LAW {
                continue;
            }
            if self.laws.iter().any(|law| law.dispute_kind == kind) {
                continue;
            }
            let id = self.next_law_id;
            self.next_law_id = self.next_law_id.saturating_add(1);
            let law = CustomaryLaw {
                id,
                dispute_kind: kind,
                label: kind.slug().to_string(),
                codified: false,
                enforcement_level: 0,
                emerged_at_tick: tick,
            };
            events.push(LawEvent {
                faction_id: 0,
                law_id: id,
                kind: LawEventKind::Emerged,
                dispute_kind: kind,
                tick,
            });
            self.laws.push(law);
        }
    }

    fn try_codify_laws(
        &mut self,
        tick: u64,
        institution_strength: u32,
        events: &mut Vec<LawEvent>,
    ) {
        if institution_strength < CODIFICATION_INSTITUTION_STRENGTH {
            return;
        }
        let enforcement = institution_strength.min(4) as u8;
        for law in &mut self.laws {
            if law.codified {
                continue;
            }
            law.codified = true;
            law.enforcement_level = enforcement;
            events.push(LawEvent {
                faction_id: 0,
                law_id: law.id,
                kind: LawEventKind::Codified,
                dispute_kind: law.dispute_kind,
                tick,
            });
        }
    }

    /// Build a snapshot for wire export.
    #[must_use]
    pub fn snapshot(
        &self,
        faction_id: u32,
        institution_strength: u32,
    ) -> FactionLawSnapshot {
        FactionLawSnapshot {
            faction_id,
            laws: self.laws.clone(),
            institutional_strength: institution_strength,
            dispute_counts: self.dispute_counts.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_disputes_emerge_law_at_threshold() {
        let mut state = FactionLawState::default();
        for _ in 0..DISPUTE_THRESHOLD_FOR_LAW {
            state.record_dispute(DisputeKind::Theft);
        }
        let events = state.tick_laws(10, 0);
        assert_eq!(state.laws.len(), 1);
        assert_eq!(state.laws[0].dispute_kind, DisputeKind::Theft);
        assert!(!state.laws[0].codified);
        assert!(
            events
                .iter()
                .any(|e| e.kind == LawEventKind::Emerged && e.dispute_kind == DisputeKind::Theft)
        );
    }

    #[test]
    fn institutions_codify_emerged_laws() {
        let mut state = FactionLawState::default();
        for _ in 0..DISPUTE_THRESHOLD_FOR_LAW {
            state.record_dispute(DisputeKind::Violence);
        }
        state.tick_laws(1, 0);
        let events = state.tick_laws(2, CODIFICATION_INSTITUTION_STRENGTH);
        assert!(state.laws[0].codified);
        assert!(state.laws[0].enforcement_level >= 1);
        assert!(events.iter().any(|e| e.kind == LawEventKind::Codified));
    }
}
