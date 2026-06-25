//! Governance tick: dispute ingestion → emergent customary law (FR-LAW).

use civ_institutions::Institution;
use std::collections::BTreeMap;

use crate::institution::faction_institution_strength;
use crate::law::{
    DisputeKind, FactionLawSnapshot, FactionLawState, LawEvent, DISPUTE_THRESHOLD_FOR_LAW,
};

/// Organic dispute signals derived from settlement substrate each tick.
#[derive(Debug, Clone, Default)]
pub struct GovernanceSignals {
    pub crime_pressure: BTreeMap<u32, i32>,
    pub unrest_scores: BTreeMap<u32, i32>,
    pub resource_stress: BTreeMap<u32, i32>,
}

/// Pending explicit disputes queued by tests or god-tools.
pub type PendingDisputes = Vec<(u32, DisputeKind)>;

/// Run one governance tick for all factions.
pub fn tick_governance(
    tick: u64,
    settlement_factions: &BTreeMap<u32, u32>,
    institutions: &BTreeMap<u32, Institution>,
    signals: &GovernanceSignals,
    faction_laws: &mut BTreeMap<u32, FactionLawState>,
    pending: &mut PendingDisputes,
) -> Vec<LawEvent> {
  ingest_organic_disputes(settlement_factions, signals, faction_laws);
  ingest_pending_disputes(pending, faction_laws);

  let mut all_events = Vec::new();
  let faction_ids: Vec<u32> = faction_laws.keys().copied().collect();
  for faction_id in faction_ids {
    let strength =
      faction_institution_strength(faction_id, settlement_factions, institutions);
    if let Some(state) = faction_laws.get_mut(&faction_id) {
      let mut events = state.tick_laws(tick, strength);
      for event in &mut events {
        event.faction_id = faction_id;
      }
      all_events.extend(events);
    }
  }
  all_events
}

fn ingest_organic_disputes(
    settlement_factions: &BTreeMap<u32, u32>,
    signals: &GovernanceSignals,
    faction_laws: &mut BTreeMap<u32, FactionLawState>,
) {
    for (sid, faction_id) in settlement_factions {
        if signals.crime_pressure.get(sid).copied().unwrap_or(0) >= 40 {
            faction_laws
                .entry(*faction_id)
                .or_default()
                .record_dispute(DisputeKind::Theft);
        }
        if signals.unrest_scores.get(sid).copied().unwrap_or(0) >= 50 {
            faction_laws
                .entry(*faction_id)
                .or_default()
                .record_dispute(DisputeKind::Violence);
        }
        if signals.resource_stress.get(sid).copied().unwrap_or(0) >= 30 {
            faction_laws
                .entry(*faction_id)
                .or_default()
                .record_dispute(DisputeKind::ResourceDispute);
        }
    }
}

fn ingest_pending_disputes(
    pending: &mut PendingDisputes,
    faction_laws: &mut BTreeMap<u32, FactionLawState>,
) {
    for (faction_id, kind) in pending.drain(..) {
        faction_laws
            .entry(faction_id)
            .or_default()
            .record_dispute(kind);
    }
}

/// Build per-faction law snapshots for the main simulation snapshot.
#[must_use]
pub fn faction_law_snapshots(
    settlement_factions: &BTreeMap<u32, u32>,
    institutions: &BTreeMap<u32, Institution>,
    faction_laws: &BTreeMap<u32, FactionLawState>,
) -> BTreeMap<u32, FactionLawSnapshot> {
    faction_laws
        .iter()
        .map(|(&faction_id, state)| {
            let strength =
                faction_institution_strength(faction_id, settlement_factions, institutions);
            (faction_id, state.snapshot(faction_id, strength))
        })
        .collect()
}

/// Ensure a faction has law state before recording disputes.
pub fn ensure_faction(faction_laws: &mut BTreeMap<u32, FactionLawState>, faction_id: u32) {
    faction_laws.entry(faction_id).or_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disputatious_faction_accumulates_more_laws_than_peaceful() {
        let mut disputatious = BTreeMap::new();
        disputatious.insert(0, FactionLawState::default());
        let mut peaceful = BTreeMap::new();
        peaceful.insert(0, FactionLawState::default());

        let mut pending_hot = Vec::new();
        for _ in 0..10 {
            pending_hot.push((0, DisputeKind::Theft));
            pending_hot.push((0, DisputeKind::Violence));
            pending_hot.push((0, DisputeKind::ResourceDispute));
        }

        let owners = BTreeMap::new();
        let institutions = BTreeMap::new();
        let signals = GovernanceSignals::default();

        tick_governance(
            1,
            &owners,
            &institutions,
            &signals,
            &mut disputatious,
            &mut pending_hot,
        );
        tick_governance(
            10,
            &owners,
            &institutions,
            &signals,
            &mut peaceful,
            &mut Vec::new(),
        );

        let hot_laws = disputatious.get(&0).map(|s| s.laws.len()).unwrap_or(0);
        let calm_laws = peaceful.get(&0).map(|s| s.laws.len()).unwrap_or(0);
        assert!(
            hot_laws > calm_laws,
            "disputatious faction should develop more laws ({hot_laws}) than peaceful ({calm_laws})"
        );
        assert_eq!(hot_laws, 3, "expected one law per dispute kind");
        assert_eq!(calm_laws, 0, "peaceful faction should have no laws");
    }
}
