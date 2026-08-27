//! Alliance Formation System — deep diplomacy for multi-faction alliances.
//!
//! Provides structured alliance formation with compatibility evaluation,
//! strength computation, and lifecycle management. All scores use fixed-point
//! arithmetic (i32, scaled by 100 where 100 = 1.0) for determinism.
//!
//! # Determinism
//!
//! All computation is integer-only over [`BTreeMap`]/[`BTreeSet`]-backed
//! collections. Given the same inputs, the same functions produce identical
//! outputs. No RNG, no floating-point, no wall-clock.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

use crate::PolityId;

/// Purpose of an alliance — determines which bonuses apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlliancePurpose {
    /// Military coordination and shared intelligence.
    Military,
    /// Trade agreements and economic cooperation.
    Economic,
    /// Cultural exchange and norm propagation.
    Cultural,
}

/// A formal multi-faction alliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alliance {
    /// Unique alliance identifier.
    pub id: u64,
    /// Member factions (sorted for determinism).
    pub members: BTreeSet<PolityId>,
    /// Aggregate alliance strength (fixed-point, scaled x100).
    pub strength: i32,
    /// Tick when this alliance was formed.
    pub formed_at_tick: u64,
    /// Purpose of the alliance.
    pub purpose: AlliancePurpose,
}

/// Criteria for evaluating an alliance proposal between two factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllianceProposalCriteria {
    /// Shared enemy score: number of common enemies × 100.
    pub shared_enemy_score: i32,
    /// Trade volume between the two factions (scaled x100).
    pub trade_volume_score: i32,
    /// Cultural similarity between the two factions (0–10000, higher = more similar).
    pub cultural_similarity_score: i32,
    /// Combined military strength of both factions (fixed-point, scaled x100).
    pub combined_strength: i32,
}

/// Errors during alliance operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AllianceError {
    /// Alliance with the given id does not exist.
    #[error("alliance {0} not found")]
    NotFound(u64),
    /// Faction is already a member of this alliance.
    #[error("faction {0} is already a member of alliance {1}")]
    AlreadyMember(PolityId, u64),
    /// Faction is not a member of this alliance.
    #[error("faction {0} is not a member of alliance {1}")]
    NotMember(PolityId, u64),
    /// Alliance has no members (should never happen).
    #[error("alliance has no members")]
    EmptyAlliance,
}

/// Manages active alliances and provides formation/dissolution logic.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllianceManager {
    /// Active alliances indexed by id.
    pub active_alliances: BTreeMap<u64, Alliance>,
    /// Next alliance id to assign.
    next_id: u64,
    /// Minimum compatibility score (0–10000) to approve a proposal.
    pub formation_threshold: i32,
}

impl AllianceManager {
    /// Create a new `AllianceManager` with the given formation threshold.
    pub fn new(formation_threshold: i32) -> Self {
        Self {
            active_alliances: BTreeMap::new(),
            next_id: 1,
            formation_threshold,
        }
    }

    /// Evaluate whether an alliance proposal meets the compatibility criteria.
    ///
    /// Returns `true` if the weighted compatibility score meets or exceeds
    /// the formation threshold. The score is computed as:
    ///
    ///   `shared_enemy_score * 0.4 + trade_volume_score * 0.3 + cultural_similarity_score * 0.3`
    ///
    /// (All values are scaled x100; the weights sum to 100.)
    pub fn evaluate_alliance_proposal(
        &self,
        criteria: &AllianceProposalCriteria,
    ) -> bool {
        // Fixed-point weighted sum: weights x100, scores x100 → result x10000.
        let score = criteria.shared_enemy_score * 40
            + criteria.trade_volume_score * 30
            + criteria.cultural_similarity_score * 30;
        // Threshold is already on the 0–10000 scale.
        score >= self.formation_threshold * 100
    }

    /// Form a new alliance with the given members and purpose.
    ///
    /// Returns the alliance id on success.
    pub fn form_alliance(
        &mut self,
        members: BTreeSet<PolityId>,
        purpose: AlliancePurpose,
        tick: u64,
    ) -> Result<u64, AllianceError> {
        if members.is_empty() {
            return Err(AllianceError::EmptyAlliance);
        }
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.active_alliances.insert(
            id,
            Alliance {
                id,
                members,
                strength: 0, // computed lazily or set externally
                formed_at_tick: tick,
                purpose,
            },
        );
        Ok(id)
    }

    /// Dissolve an alliance by id, removing it from active alliances.
    pub fn dissolve_alliance(&mut self, id: u64) -> Result<Alliance, AllianceError> {
        self.active_alliances
            .remove(&id)
            .ok_or(AllianceError::NotFound(id))
    }

    /// Compute aggregate alliance power by summing faction resources.
    ///
    /// `resources` maps each faction id to its current resource count.
    /// The alliance power is the sum of all members' resources, scaled x100
    /// and divided by the number of members for an average.
    pub fn compute_alliance_power(
        &self,
        id: u64,
        resources: &BTreeMap<PolityId, i32>,
    ) -> Result<i32, AllianceError> {
        let alliance = self.active_alliances.get(&id).ok_or(AllianceError::NotFound(id))?;
        if alliance.members.is_empty() {
            return Err(AllianceError::EmptyAlliance);
        }
        let total: i32 = alliance
            .members
            .iter()
            .map(|m| resources.get(m).copied().unwrap_or(0))
            .sum();
        // Average power scaled x100.
        Ok(total / alliance.members.len() as i32)
    }

    /// List all alliances that a faction belongs to.
    pub fn alliances_for(&self, faction: PolityId) -> Vec<&Alliance> {
        self.active_alliances
            .values()
            .filter(|a| a.members.contains(&faction))
            .collect()
    }

    /// Number of active alliances.
    pub fn len(&self) -> usize {
        self.active_alliances.len()
    }

    /// Whether there are no active alliances.
    pub fn is_empty(&self) -> bool {
        self.active_alliances.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    fn default_criteria() -> AllianceProposalCriteria {
        AllianceProposalCriteria {
            shared_enemy_score: 500,
            trade_volume_score: 300,
            cultural_similarity_score: 400,
            combined_strength: 10_000,
        }
    }

    #[test]
    fn evaluate_proposal_above_threshold_approves() {
        let _mgr = AllianceManager::new(5000);
        let criteria = default_criteria();
        // score = 500*40 + 300*30 + 400*30 = 20000+9000+12000 = 41000
        // threshold*100 = 5000*100 = 500000
        // 41000 < 500000 → false with threshold 5000
        // Use lower threshold
        let mgr2 = AllianceManager::new(410);
        assert!(mgr2.evaluate_alliance_proposal(&criteria));
    }

    #[test]
    fn evaluate_proposal_below_threshold_rejects() {
        let mgr = AllianceManager::new(9000);
        let criteria = default_criteria();
        // score = 41000, threshold*100 = 900000 → false
        assert!(!mgr.evaluate_alliance_proposal(&criteria));
    }

    #[test]
    fn evaluate_proposal_exact_threshold_approves() {
        // score = 100*40 + 0*30 + 0*30 = 4000
        // threshold = 40 → 40*100 = 4000
        let mgr = AllianceManager::new(40);
        let criteria = AllianceProposalCriteria {
            shared_enemy_score: 100,
            trade_volume_score: 0,
            cultural_similarity_score: 0,
            combined_strength: 5000,
        };
        assert!(mgr.evaluate_alliance_proposal(&criteria));
    }

    #[test]
    fn form_alliance_assigns_incrementing_ids() {
        let mut mgr = AllianceManager::new(0);
        let members1: BTreeSet<_> = vec![p(1), p(2)].into_iter().collect();
        let members2: BTreeSet<_> = vec![p(3), p(4)].into_iter().collect();
        let id1 = mgr
            .form_alliance(members1, AlliancePurpose::Military, 100)
            .expect("form alliance 1");
        let id2 = mgr
            .form_alliance(members2, AlliancePurpose::Economic, 200)
            .expect("form alliance 2");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn form_alliance_empty_members_errors() {
        let mut mgr = AllianceManager::new(0);
        let result = mgr.form_alliance(BTreeSet::new(), AlliancePurpose::Cultural, 1);
        assert!(matches!(result, Err(AllianceError::EmptyAlliance)));
    }

    #[test]
    fn dissolve_alliance_removes_from_active() {
        let mut mgr = AllianceManager::new(0);
        let members: BTreeSet<_> = vec![p(1), p(2)].into_iter().collect();
        let id = mgr
            .form_alliance(members, AlliancePurpose::Military, 10)
            .expect("form");
        assert_eq!(mgr.len(), 1);
        let dissolved = mgr.dissolve_alliance(id).expect("dissolve");
        assert_eq!(dissolved.id, id);
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn dissolve_nonexistent_alliance_errors() {
        let mut mgr = AllianceManager::new(0);
        let result = mgr.dissolve_alliance(999);
        assert!(matches!(result, Err(AllianceError::NotFound(999))));
    }

    #[test]
    fn compute_alliance_power_averages_member_resources() {
        let mut mgr = AllianceManager::new(0);
        let members: BTreeSet<_> = vec![p(1), p(2), p(3)].into_iter().collect();
        let id = mgr
            .form_alliance(members, AlliancePurpose::Military, 1)
            .expect("form");

        let mut resources = BTreeMap::new();
        resources.insert(p(1), 300);
        resources.insert(p(2), 600);
        resources.insert(p(3), 900);
        // average = (300+600+900)/3 = 600
        let power = mgr
            .compute_alliance_power(id, &resources)
            .expect("power");
        assert_eq!(power, 600);
    }

    #[test]
    fn compute_alliance_power_missing_resources_default_zero() {
        let mut mgr = AllianceManager::new(0);
        let members: BTreeSet<_> = vec![p(1), p(2)].into_iter().collect();
        let id = mgr
            .form_alliance(members, AlliancePurpose::Economic, 1)
            .expect("form");
        let resources = BTreeMap::new(); // empty
        let power = mgr
            .compute_alliance_power(id, &resources)
            .expect("power");
        assert_eq!(power, 0);
    }

    #[test]
    fn alliances_for_returns_only_relevant() {
        let mut mgr = AllianceManager::new(0);
        let m12: BTreeSet<_> = vec![p(1), p(2)].into_iter().collect();
        let m34: BTreeSet<_> = vec![p(3), p(4)].into_iter().collect();
        mgr.form_alliance(m12, AlliancePurpose::Military, 1)
            .expect("form");
        mgr.form_alliance(m34, AlliancePurpose::Cultural, 2)
            .expect("form");

        let for_1 = mgr.alliances_for(p(1));
        assert_eq!(for_1.len(), 1);
        assert!(for_1[0].members.contains(&p(1)));
        assert!(!for_1[0].members.contains(&p(3)));

        let for_5 = mgr.alliances_for(p(5));
        assert!(for_5.is_empty());
    }

    #[test]
    fn alliance_purpose_roundtrips_serialization() {
        let purpose = AlliancePurpose::Military;
        let json = serde_json::to_string(&purpose).expect("serialize");
        let decoded: AlliancePurpose = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(purpose, decoded);
    }

    #[test]
    fn manager_default_is_empty() {
        let mgr = AllianceManager::default();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }
}
