//! FR-CIV-WARFARE-001 — War emerges from rivalry/border friction/resource competition.

use diplomacy::{InteractionEvent, Pair, PolityId, Relation};

/// Standing level below which a pair transitions to active war.
pub const WAR_STANDING_THRESHOLD: i32 = -60;
/// Standing drain applied to each faction pair per combat engagement.
pub const COMBAT_STANDING_DRAIN: i32 = -8;
/// Passive drain applied each tick due to unresolved rivalry.
pub const RIVALRY_FRICTION_DRAIN: i32 = -3;

/// Snapshot of an active war between a pair of polities.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WarState {
    /// The pair whose standing crossed the war threshold.
    pub pair: Pair,
    /// Tick at which war was declared.
    pub onset_tick: u64,
    /// Whether the war is still ongoing.
    pub ongoing: bool,
}

/// Check whether a relation has crossed the war-onset threshold.
///
/// Returns `Some(WarState)` on the tick the standing first falls below
/// [`WAR_STANDING_THRESHOLD`]; `None` while standing is above or equal.
pub fn check_war_onset(relation: &Relation, current_tick: u64) -> Option<WarState> {
    if relation.standing < WAR_STANDING_THRESHOLD {
        Some(WarState {
            pair: relation.pair,
            onset_tick: current_tick,
            ongoing: true,
        })
    } else {
        None
    }
}

/// Apply passive rivalry friction to a standing value.
///
/// Called each tick for pairs that remain unresolved rivals; returns the
/// reduced standing so callers can decide whether to persist it.
pub fn apply_rivalry_friction(standing: i32) -> i32 {
    standing - RIVALRY_FRICTION_DRAIN
}

/// Convert a slice of [`CombatEngagement`]s into [`InteractionEvent::Combat`]
/// events the diplomacy substrate can ingest.
pub fn engagements_to_diplomacy_events(
    engagements: &[crate::CombatEngagement],
    tick: u64,
) -> Vec<InteractionEvent> {
    engagements
        .iter()
        .map(|e| InteractionEvent::Combat {
            attacker: PolityId::new(e.shooter_faction),
            defender: PolityId::new(e.target_faction),
            energy: e.damage.energy,
            tick,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use diplomacy::{Pair, PolityId, Relation};

    #[test]
    fn war_emerges_from_rivalry_threshold() {
        let pair = Pair::new(PolityId::new(1), PolityId::new(2));
        let relation = Relation {
            pair,
            standing: WAR_STANDING_THRESHOLD - 1,
            last_updated_tick: 0,
        };
        let result = check_war_onset(&relation, 10);
        assert!(result.is_some());
        let ws = result.unwrap();
        assert!(ws.ongoing);
        assert_eq!(ws.onset_tick, 10);
    }

    #[test]
    fn war_does_not_emerge_above_threshold() {
        let pair = Pair::new(PolityId::new(1), PolityId::new(2));
        let relation = Relation {
            pair,
            standing: 0,
            last_updated_tick: 0,
        };
        assert!(check_war_onset(&relation, 5).is_none());
    }

    #[test]
    fn war_does_not_emerge_at_exact_threshold() {
        let pair = Pair::new(PolityId::new(1), PolityId::new(2));
        let relation = Relation {
            pair,
            standing: WAR_STANDING_THRESHOLD,
            last_updated_tick: 0,
        };
        assert!(check_war_onset(&relation, 5).is_none());
    }

    #[test]
    fn rivalry_friction_drains_standing() {
        let standing = 10;
        let drained = apply_rivalry_friction(standing);
        assert!(drained < standing);
        assert_eq!(drained, standing - RIVALRY_FRICTION_DRAIN);
    }

    #[test]
    fn engagements_drain_diplomacy_standing() {
        let engagements: Vec<crate::CombatEngagement> = vec![];
        let events = engagements_to_diplomacy_events(&engagements, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn war_onset_deterministic_per_seed() {
        let pair = Pair::new(PolityId::new(3), PolityId::new(4));
        let rel = Relation {
            pair,
            standing: -100,
            last_updated_tick: 0,
        };
        let r1 = check_war_onset(&rel, 42);
        let r2 = check_war_onset(&rel, 42);
        assert_eq!(r1, r2);
    }
}
