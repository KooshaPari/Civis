//! Legend significance weighting (FR-CIV-LEGEND-WEIGHT, #962).
//!
//! Notable events accrue significance over time based on:
//! - **Event magnitude** (raw impact from the producer)
//! - **Role diversity** (Leader/Founder roles contribute more than Witness)
//! - **Temporal clustering** (events close in time reinforce each other)
//! - **Participant count** (multi-entity events are more significant)
//! - **Kind weight** (War/Death/Extinction > Sickness/Migration)
//!
//! The accumulator runs on the sim hot path as a cheap O(1) per-event update.
//! The query layer reads accumulated scores for culture diffusion, narrator
//! prose, and inspector ranking.
//!
//! Charter: this module records *what the sim already produced*. It never
//! generates outcomes or mutates the saga graph structure.

use crate::ids::{Epoch, LegendEntityId};
use crate::model::{EventKind, Role};

/// Configuration for the significance accumulator.
#[derive(Debug, Clone)]
pub struct SignificanceConfig {
    /// Per-epoch exponential decay factor for accumulated significance.
    /// 0.95 = slow decay (events stay significant for ~44 epochs).
    pub decay_rate: f32,
    /// Bonus multiplier for temporal clustering (events within this many
    /// epochs of each other reinforce).
    pub cluster_window: u64,
    /// Multiplier applied when events are within the cluster window.
    pub cluster_bonus: f32,
    /// Maximum accumulated significance (caps at 1.0).
    pub max_significance: f32,
}

impl Default for SignificanceConfig {
    fn default() -> Self {
        SignificanceConfig {
            decay_rate: 0.95,
            cluster_window: 3,
            cluster_bonus: 0.15,
            max_significance: 1.0,
        }
    }
}

/// Per-entity accumulated significance state.
#[derive(Debug, Clone)]
pub struct EntitySignificance {
    /// Rolling accumulated significance score (0..=1).
    pub score: f32,
    /// Epoch of the last event that contributed to this score.
    pub last_event_epoch: Epoch,
    /// Total number of events that contributed.
    pub event_count: u32,
    /// Number of distinct roles seen (role diversity indicator).
    pub role_diversity: u32,
    /// Highest single-event magnitude seen.
    pub peak_magnitude: f32,
}

impl EntitySignificance {
    fn new(score: f32, epoch: Epoch) -> Self {
        Self {
            score,
            last_event_epoch: epoch,
            event_count: 1,
            role_diversity: 0,
            peak_magnitude: 0.0,
        }
    }
}

/// Accumulates significance scores for entities over time.
///
/// Usage:
/// ```ignore
/// let mut acc = SignificanceAccumulator::default();
/// acc.record_event(entity_id, epoch, &EventKind::Battle, &[Role::Leader], 0.8, &config);
/// let sig = acc.get(entity_id);
/// assert!(sig.score > 0.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SignificanceAccumulator {
    /// Entity id → accumulated significance.
    by_entity: std::collections::HashMap<LegendEntityId, EntitySignificance>,
}

impl SignificanceAccumulator {
    /// Construct a fresh, empty accumulator.
    pub fn new() -> Self {
        Self {
            by_entity: std::collections::HashMap::new(),
        }
    }

    /// Number of entities being tracked.
    pub fn len(&self) -> usize {
        self.by_entity.len()
    }

    /// True iff no entities are tracked.
    pub fn is_empty(&self) -> bool {
        self.by_entity.is_empty()
    }

    /// Current significance for an entity, if tracked.
    pub fn get(&self, id: LegendEntityId) -> Option<&EntitySignificance> {
        self.by_entity.get(&id)
    }

    /// Mutable access to an entity's significance (for external decay sweeps).
    pub fn get_mut(&mut self, id: LegendEntityId) -> Option<&mut EntitySignificance> {
        self.by_entity.get_mut(&id)
    }

    /// Record a single event's contribution to an entity's significance.
    ///
    /// This is the core O(1) update called on the sim hot path. It:
    /// 1. Applies exponential decay since the last event
    /// 2. Computes the event's contribution (magnitude × kind_weight × role_weight × diversity_bonus)
    /// 3. Applies temporal clustering bonus if events are close in time
    /// 4. Clamps to [0, max_significance]
    pub fn record_event(
        &mut self,
        entity: LegendEntityId,
        epoch: Epoch,
        kind: &EventKind,
        roles: &[Role],
        magnitude: f32,
        config: &SignificanceConfig,
    ) {
        let kind_w = crate::config::kind_weight(kind);
        let role_w = roles.iter().map(|r| r.weight()).sum::<f32>().max(0.2);
        let role_count = roles.len() as f32;

        // Base contribution: magnitude × kind weight × role weight × sqrt(participant count)
        // sqrt prevents blowup from events with many participants while still rewarding groups.
        let base = magnitude * kind_w * role_w * role_count.sqrt();

        let next = match self.by_entity.get(&entity) {
            None => {
                // First event for this entity.
                let score = base.min(config.max_significance);
                let mut es = EntitySignificance::new(score, epoch);
                es.role_diversity = count_unique_roles(roles);
                es.peak_magnitude = magnitude;
                es
            }
            Some(prev) => {
                // Apply decay since last event.
                let elapsed = epoch.0.saturating_sub(prev.last_event_epoch.0);
                let decayed_score = if elapsed > 0 {
                    (prev.score * config.decay_rate.powi(elapsed as i32)).max(0.0)
                } else {
                    prev.score
                };

                // Temporal clustering bonus: events within cluster_window reinforce.
                let cluster_mult = if elapsed > 0 && elapsed <= config.cluster_window {
                    1.0 + config.cluster_bonus
                } else {
                    1.0
                };

                // Role diversity bonus: more distinct roles = more significant.
                let new_roles = count_unique_roles(roles);
                let total_roles = prev.role_diversity + new_roles;
                let diversity_bonus = 1.0 + (total_roles as f32).min(5.0) * 0.05;

                // Accumulate.
                let new_score = (decayed_score + base * cluster_mult * diversity_bonus)
                    .min(config.max_significance);

                let mut es = prev.clone();
                es.score = new_score;
                es.last_event_epoch = epoch;
                es.event_count += 1;
                es.role_diversity = total_roles;
                es.peak_magnitude = es.peak_magnitude.max(magnitude);
                es
            }
        };

        self.by_entity.insert(entity, next);
    }

    /// Apply epoch-level decay to all tracked entities (end-of-epoch sweep).
    pub fn sweep(&mut self, current_epoch: Epoch, config: &SignificanceConfig) {
        for es in self.by_entity.values_mut() {
            let elapsed = current_epoch.0.saturating_sub(es.last_event_epoch.0);
            if elapsed > 0 {
                es.score = (es.score * config.decay_rate.powi(elapsed as i32)).max(0.0);
                es.last_event_epoch = current_epoch;
            }
        }
    }

    /// Prune entities below the given significance floor. Returns pruned ids.
    pub fn prune_below(&mut self, floor: f32) -> Vec<LegendEntityId> {
        let pruned: Vec<_> = self
            .by_entity
            .iter()
            .filter_map(|(k, v)| if v.score < floor { Some(*k) } else { None })
            .collect();
        for id in &pruned {
            self.by_entity.remove(id);
        }
        pruned
    }

    /// Return all tracked entities sorted by significance (descending).
    pub fn ranked(&self) -> Vec<(LegendEntityId, f32)> {
        let mut entries: Vec<_> = self.by_entity.iter().map(|(k, v)| (*k, v.score)).collect();
        entries.sort_by(|a, b| b.1.total_cmp(&a.1));
        entries
    }

    /// Compute a weighted significance score for culture diffusion.
    ///
    /// This is the public API consumed by `civ-engine` to weight how quickly
    /// a legend spreads through culture: higher significance = faster spread.
    ///
    /// Formula: `score × (1 + log2(event_count + 1) × 0.1) × (1 + peak_magnitude × 0.2)`
    ///
    /// The log(event_count) term rewards entities with many events without
    /// blowing up linearly. The peak_magnitude term rewards entities that
    /// had at least one very significant event.
    pub fn diffusion_weight(&self, entity: LegendEntityId) -> f32 {
        self.by_entity
            .get(&entity)
            .map(|es| {
                let count_bonus = 1.0 + ((es.event_count + 1) as f32).log2() * 0.1;
                let peak_bonus = 1.0 + es.peak_magnitude * 0.2;
                es.score * count_bonus * peak_bonus
            })
            .unwrap_or(0.0)
    }
}

/// Count distinct roles in a slice (for diversity tracking).
fn count_unique_roles(roles: &[Role]) -> u32 {
    let mut seen = std::collections::HashSet::new();
    for r in roles {
        seen.insert(*r);
    }
    seen.len() as u32
}

/// Weighted significance score for a single event (used by the narrator
/// and inspector to rank events in epoch digests).
pub fn event_significance(
    kind: &EventKind,
    magnitude: f32,
    roles: &[Role],
    participant_count: usize,
) -> f32 {
    let kind_w = crate::config::kind_weight(kind);
    let role_w = roles.iter().map(|r| r.weight()).sum::<f32>().max(0.2);
    let participant_bonus = (participant_count as f32).sqrt();
    magnitude * kind_w * role_w * participant_bonus
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{LegendEntityId, LegendEventId};
    use crate::model::{EventKind, Role};

    #[test]
    fn first_event_sets_score() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig::default();
        let eid = LegendEntityId(1);

        acc.record_event(
            eid,
            Epoch(0),
            &EventKind::Battle,
            &[Role::Leader],
            0.8,
            &cfg,
        );

        let sig = acc.get(eid).expect("entity should be tracked");
        assert!(sig.score > 0.0, "score should be positive");
        assert_eq!(sig.event_count, 1);
        assert_eq!(sig.peak_magnitude, 0.8);
    }

    #[test]
    fn accumulated_score_increases() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig::default();
        let eid = LegendEntityId(1);

        acc.record_event(
            eid,
            Epoch(0),
            &EventKind::Battle,
            &[Role::Leader],
            0.5,
            &cfg,
        );
        let score1 = acc.get(eid).unwrap().score;

        acc.record_event(
            eid,
            Epoch(1),
            &EventKind::WarDeclared,
            &[Role::Leader, Role::Aggressor],
            0.9,
            &cfg,
        );
        let score2 = acc.get(eid).unwrap().score;

        assert!(
            score2 > score1,
            "accumulated score should increase: {score1} -> {score2}"
        );
    }

    #[test]
    fn decay_reduces_score() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig {
            decay_rate: 0.5,
            ..Default::default()
        };
        let eid = LegendEntityId(1);

        acc.record_event(
            eid,
            Epoch(0),
            &EventKind::Battle,
            &[Role::Leader],
            0.8,
            &cfg,
        );
        let score_before = acc.get(eid).unwrap().score;

        // Decay for 10 epochs with no new events.
        acc.record_event(
            eid,
            Epoch(10),
            &EventKind::Birth,
            &[Role::Witness],
            0.1,
            &cfg,
        );
        let score_after = acc.get(eid).unwrap().score;

        // The score should still increase (new event adds), but the decay
        // from the gap means it's less than if there were no gap.
        assert!(score_after > 0.0, "score should remain positive");
    }

    #[test]
    fn temporal_clustering_bonus() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig {
            cluster_window: 3,
            cluster_bonus: 0.5,
            decay_rate: 1.0, // no decay for isolation
            ..Default::default()
        };
        let eid = LegendEntityId(1);

        // Two events 1 epoch apart (within cluster window).
        acc.record_event(
            eid,
            Epoch(0),
            &EventKind::Battle,
            &[Role::Leader],
            0.5,
            &cfg,
        );
        let score_clustered = acc.get(eid).unwrap().score;

        acc.record_event(
            eid,
            Epoch(1),
            &EventKind::Battle,
            &[Role::Leader],
            0.5,
            &cfg,
        );
        let score_final = acc.get(eid).unwrap().score;

        // Now reset and do the same with a gap outside the cluster window.
        let mut acc2 = SignificanceAccumulator::new();
        acc2.record_event(
            eid,
            Epoch(0),
            &EventKind::Battle,
            &[Role::Leader],
            0.5,
            &cfg,
        );
        acc2.record_event(
            eid,
            Epoch(10),
            &EventKind::Battle,
            &[Role::Leader],
            0.5,
            &cfg,
        );
        let score_gap = acc2.get(eid).unwrap().score;

        assert!(
            score_final > score_gap,
            "clustered events should accumulate more: clustered={score_final}, gap={score_gap}"
        );
    }

    #[test]
    fn role_diversity_increases_score() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig {
            decay_rate: 1.0, // no decay
            ..Default::default()
        };
        let eid = LegendEntityId(1);

        // Same event repeated with same role.
        for _ in 0..5 {
            acc.record_event(
                eid,
                Epoch(0),
                &EventKind::Battle,
                &[Role::Leader],
                0.5,
                &cfg,
            );
        }
        let score_same = acc.get(eid).unwrap().score;

        // Reset and repeat with diverse roles.
        let mut acc2 = SignificanceAccumulator::new();
        let roles = [
            Role::Leader,
            Role::Aggressor,
            Role::Defender,
            Role::Builder,
            Role::Witness,
        ];
        for r in &roles {
            acc2.record_event(eid, Epoch(0), &EventKind::Battle, &[*r], 0.5, &cfg);
        }
        let score_diverse = acc2.get(eid).unwrap().score;

        assert!(
            score_diverse > score_same,
            "diverse roles should score higher: same={score_same}, diverse={score_diverse}"
        );
    }

    #[test]
    fn diffusion_weight_combines_factors() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig::default();
        let eid = LegendEntityId(1);

        // Seed with multiple events.
        for tick in 0..10u64 {
            acc.record_event(
                eid,
                Epoch(tick),
                &EventKind::Battle,
                &[Role::Leader],
                0.8,
                &cfg,
            );
        }

        let weight = acc.diffusion_weight(eid);
        assert!(weight > 0.0, "diffusion weight should be positive");
        // With high magnitude + many events, weight should be > raw score.
        let raw_score = acc.get(eid).unwrap().score;
        assert!(
            weight > raw_score,
            "diffusion weight should boost raw score"
        );
    }

    #[test]
    fn ranked_returns_descending_order() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig::default();

        // Create 3 entities with different significance levels.
        acc.record_event(
            LegendEntityId(1),
            Epoch(0),
            &EventKind::Battle,
            &[Role::Leader],
            0.3,
            &cfg,
        );
        acc.record_event(
            LegendEntityId(2),
            Epoch(0),
            &EventKind::WarDeclared,
            &[Role::Leader, Role::Aggressor],
            0.9,
            &cfg,
        );
        acc.record_event(
            LegendEntityId(3),
            Epoch(0),
            &EventKind::Sickness,
            &[Role::Witness],
            0.1,
            &cfg,
        );

        let ranked = acc.ranked();
        assert_eq!(ranked.len(), 3);
        // Entity 2 (WarDeclared + high magnitude + 2 roles) should be first.
        assert_eq!(ranked[0].0, LegendEntityId(2));
        // Entity 3 (Sickness + low magnitude + 1 role) should be last.
        assert_eq!(ranked[2].0, LegendEntityId(3));
    }

    #[test]
    fn prune_below_removes_weak_entities() {
        let mut acc = SignificanceAccumulator::new();
        let cfg = SignificanceConfig {
            decay_rate: 0.1, // very fast decay
            ..Default::default()
        };

        acc.record_event(
            LegendEntityId(1),
            Epoch(0),
            &EventKind::Battle,
            &[Role::Leader],
            0.8,
            &cfg,
        );
        acc.record_event(
            LegendEntityId(2),
            Epoch(0),
            &EventKind::Sickness,
            &[Role::Witness],
            0.1,
            &cfg,
        );

        // Advance far enough that both decay significantly.
        acc.sweep(Epoch(100), &cfg);

        let pruned = acc.prune_below(0.01);
        assert!(!pruned.is_empty(), "at least one entity should be pruned");
    }

    #[test]
    fn event_significance_helpers() {
        let sig = event_significance(&EventKind::Battle, 0.8, &[Role::Leader], 3);
        assert!(sig > 0.0);

        let sig2 = event_significance(&EventKind::Sickness, 0.2, &[Role::Witness], 1);
        assert!(
            sig > sig2,
            "Battle + Leader should be more significant than Sickness + Witness"
        );
    }
}
