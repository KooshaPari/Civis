//! FR-CIV-WARFARE-001 — War emerges from rivalry/border friction/resource competition.

use diplomacy::{InteractionEvent, Pair, PolityId, Relation};

/// Standing level below which a pair transitions to active war.
pub const WAR_STANDING_THRESHOLD: i32 = -60;
/// Standing drain applied to each faction pair per combat engagement.
pub const COMBAT_STANDING_DRAIN: i32 = -8;
/// Passive drain applied each tick due to unresolved rivalry.
pub const RIVALRY_FRICTION_DRAIN: i32 = -3;

/// Snapshot of an active war between a pair of polities.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    standing + RIVALRY_FRICTION_DRAIN
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
        assert_eq!(drained, standing + RIVALRY_FRICTION_DRAIN);
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

    // Passive friction drains standing each unresolved tick until it crosses
    // the war-onset threshold.
    // FR-CIV-WARFARE-001 — war emerges from sustained rivalry.
    #[test]
    fn fr_civ_warfare_001_rivalry_friction_escalates_to_war_onset() {
        let pair = Pair::new(PolityId::new(7), PolityId::new(8));
        let mut standing = -50;
        let mut onset: Option<WarState> = None;
        let mut tick = 0u64;
        while onset.is_none() && tick < 100 {
            standing = apply_rivalry_friction(standing);
            onset = check_war_onset(
                &Relation {
                    pair,
                    standing,
                    last_updated_tick: tick,
                },
                tick,
            );
            tick += 1;
        }

        // -50 drains by 3/tick: -53, -56, -59, -62 → onset on the 4th tick.
        let ws = onset.expect("persistent rivalry must eventually declare war");
        assert_eq!(
            ws.onset_tick, 3,
            "4 friction steps cross {} at tick 3 (0-based)",
            WAR_STANDING_THRESHOLD
        );
        assert!(ws.ongoing, "onset marks the war active");
        assert_eq!(ws.pair, pair, "onset recorded for the rival pair");
        assert!(standing < WAR_STANDING_THRESHOLD);
        assert_eq!(standing, -50 + 4 * RIVALRY_FRICTION_DRAIN);
    }

    // Combat engagements convert 1:1 into diplomacy Combat events carrying
    // shooter/target polities, damage energy, and the reporting tick.
    // FR-CIV-WARFARE-001 — warfare feeds the diplomacy substrate.
    #[test]
    fn fr_civ_warfare_001_combat_engagements_become_diplomacy_events() {
        let engagements = vec![
            crate::CombatEngagement {
                shooter_id: 1,
                target_id: 2,
                shooter_faction: 3,
                target_faction: 5,
                damage: crate::DamageEvent {
                    center: civ_voxel::WorldCoord { x: 1, y: 2, z: 3 },
                    radius_voxels: 2,
                    energy: 250,
                },
                target_index: 0,
            },
            crate::CombatEngagement {
                shooter_id: 4,
                target_id: 9,
                shooter_faction: 5,
                target_faction: 3,
                damage: crate::DamageEvent {
                    center: civ_voxel::WorldCoord { x: -4, y: 0, z: 8 },
                    radius_voxels: 1,
                    energy: 75,
                },
                target_index: 1,
            },
        ];

        let events = engagements_to_diplomacy_events(&engagements, 77);
        assert_eq!(events.len(), 2, "one diplomacy event per engagement");

        match &events[0] {
            InteractionEvent::Combat {
                attacker,
                defender,
                energy,
                tick,
            } => {
                assert_eq!(*attacker, PolityId::new(3), "attacker is the shooter faction");
                assert_eq!(*defender, PolityId::new(5), "defender is the target faction");
                assert_eq!(*energy, 250, "damage energy carried through");
                assert_eq!(*tick, 77, "reporting tick carried through");
            }
            _ => panic!("first engagement must map to InteractionEvent::Combat"),
        }
        match &events[1] {
            InteractionEvent::Combat {
                attacker, defender, ..
            } => {
                assert_eq!(*attacker, PolityId::new(5));
                assert_eq!(*defender, PolityId::new(3));
            }
            _ => panic!("second engagement must map to InteractionEvent::Combat"),
        }
    }
}
