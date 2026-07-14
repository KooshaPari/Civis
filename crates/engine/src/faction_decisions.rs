//! Per-tick faction decision step from emergent thresholds (FR-FACTION-decisions).
//!
//! Each faction reads existing emergent state (cohesion, unrest level, diplomatic
//! relation score, resource surplus/deficit) and picks exactly ONE action via simple
//! thresholds, setting an intent/flag on existing faction/diplomacy state.
//!
//! This is the "sim→game leap": factions transition from passive emergence to
//! active decision-makers responding to world state.

use std::cmp::Ordering;
use std::collections::HashMap;

use civ_agents::DiplomacySignal;

use crate::engine::{DiplomacyEvent, DiplomacyKind, MilitaryUnit, Simulation};

/// Decision action a faction may take based on emergent state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionDecision {
    /// High unrest detected: raise internal unrest-response.
    RaiseUnrestResponse,
    /// Negative relation + military advantage: flag hostility intent.
    FlagHostility,
    /// Resource surplus + positive relation: flag trade-open intent.
    FlagTradeOpen,
    /// No strong signal; maintain status quo.
    Maintain,
}

/// Evaluates faction decision each tick based on emergent thresholds.
///
/// Deterministic (engine RNG). Called once per tick after cohesion/unrest phases
/// have populated their snapshots.
///
/// # Decision Logic
///
/// - **High Unrest** (>0.7): Raise unrest-response action
/// - **Very Negative Relation** (score < -0.6) + **Military Advantage**: Flag hostility
/// - **Surplus Food** (>1000) + **Positive Relation** (>0.3): Flag trade-open
/// - Otherwise: Maintain status quo
pub fn compute_faction_decisions(sim: &Simulation) -> Vec<(u32, FactionDecision)> {
    let mut decisions = Vec::new();
    let military_counts = military_unit_counts(sim);

    // Iterate all known faction resource entries.
    for (&faction_id, _resources) in &sim.state.faction_resources {
        let decision = evaluate_faction(sim, faction_id, &military_counts);
        decisions.push((faction_id, decision));
    }

    decisions
}

/// Evaluate a single faction's decision based on emergent state.
fn evaluate_faction(
    sim: &Simulation,
    faction_id: u32,
    military_counts: &HashMap<u32, usize>,
) -> FactionDecision {
    // 1. Check unrest level across settlements controlled by this faction.
    let max_unrest = sim
        .last_tick_unrest_snapshots
        .values()
        .map(|snapshot| snapshot.level.to_rank() as f32 / 3.0)
        .fold(0.0, f32::max);

    if max_unrest > 0.7 {
        return FactionDecision::RaiseUnrestResponse;
    }

    // 2. Check cohesion and resource state.
    let avg_cohesion = sim
        .last_tick_cohesion_snapshots()
        .values()
        .map(|snapshot| match snapshot.fabric {
            crate::engine::FabricTier::Tight => 1.0,
            crate::engine::FabricTier::Loosened => 0.7,
            crate::engine::FabricTier::Strained => 0.4,
            crate::engine::FabricTier::Fractured => 0.1,
        })
        .sum::<f32>()
        / (sim.last_tick_cohesion_snapshots().len() as f32).max(1.0);

    let resources = sim
        .state
        .faction_resources
        .get(&faction_id)
        .cloned()
        .unwrap_or_default();

    // 3. Mean diplomatic score for pairs involving this faction.
    let relation_score = sim
        .faction_relations
        .mean_score_involving(faction_id)
        .unwrap_or(0.0);

    // 4. Military advantage: strictly more units than every other faction.
    let has_military_advantage = has_military_advantage(faction_id, military_counts);

    // Decision thresholds:
    if relation_score < -0.6 && has_military_advantage {
        FactionDecision::FlagHostility
    } else if resources.food.to_num::<f32>() > 1000.0 && relation_score > 0.3 && avg_cohesion > 0.5
    {
        FactionDecision::FlagTradeOpen
    } else {
        FactionDecision::Maintain
    }
}

fn military_unit_counts(sim: &Simulation) -> HashMap<u32, usize> {
    let mut counts = HashMap::new();
    for (_, unit) in sim.world.query::<&MilitaryUnit>().iter() {
        *counts.entry(unit.faction_id).or_insert(0) += 1;
    }
    counts
}

fn has_military_advantage(faction_id: u32, military_counts: &HashMap<u32, usize>) -> bool {
    let mine = military_counts.get(&faction_id).copied().unwrap_or(0);
    if mine == 0 {
        return false;
    }
    let max_other = military_counts
        .iter()
        .filter(|(id, _)| **id != faction_id)
        .map(|(_, count)| *count)
        .max()
        .unwrap_or(0);
    mine > max_other
}

/// Apply hostility/trade-open intents into relation scores and diplomacy events
/// (FR-FACTION-decisions-apply). Called at the end of `phase_faction_decisions`.
pub fn apply_faction_decision_intents(sim: &mut Simulation) {
    let tick = sim.state.tick;
    let hostility: Vec<u32> = sim
        .state
        .last_tick_faction_hostility_intents
        .iter()
        .copied()
        .collect();
    for faction_id in hostility {
        let Some(target) = pick_hostility_target(sim, faction_id) else {
            continue;
        };
        let outcome = sim.faction_relations.apply_signal(
            faction_id,
            target,
            DiplomacySignal {
                combat_grievance: 0.35,
                ..DiplomacySignal::default()
            },
        );
        sim.emit_relation_threshold_event(faction_id, target, outcome);
        sim.push_diplomacy_event(DiplomacyEvent {
            tick,
            faction_a: faction_id,
            faction_b: target,
            kind: DiplomacyKind::Conflict,
        });
    }

    let trade_open: Vec<u32> = sim
        .state
        .last_tick_faction_trade_open_intents
        .iter()
        .copied()
        .collect();
    for faction_id in trade_open {
        let Some(target) = pick_trade_target(sim, faction_id) else {
            continue;
        };
        let outcome = sim.faction_relations.apply_signal(
            faction_id,
            target,
            DiplomacySignal {
                trade_volume: 0.5,
                need_complementarity: 0.5,
                ..DiplomacySignal::default()
            },
        );
        sim.emit_relation_threshold_event(faction_id, target, outcome);
        sim.push_diplomacy_event(DiplomacyEvent {
            tick,
            faction_a: faction_id,
            faction_b: target,
            kind: DiplomacyKind::TradeAgreement,
        });
    }
}

fn pair_score(sim: &Simulation, faction_id: u32, other: u32) -> f32 {
    sim.faction_relations
        .record(faction_id, other)
        .map(|record| record.score)
        .unwrap_or(0.0)
}

fn other_factions(sim: &Simulation, faction_id: u32) -> Vec<u32> {
    let mut ids: Vec<u32> = sim
        .state
        .factions
        .keys()
        .copied()
        .filter(|id| *id != faction_id)
        .collect();
    ids.sort_unstable();
    ids
}

fn pick_hostility_target(sim: &Simulation, faction_id: u32) -> Option<u32> {
    other_factions(sim, faction_id).into_iter().min_by(|left, right| {
        pair_score(sim, faction_id, *left)
            .partial_cmp(&pair_score(sim, faction_id, *right))
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.cmp(right))
    })
}

fn pick_trade_target(sim: &Simulation, faction_id: u32) -> Option<u32> {
    other_factions(sim, faction_id).into_iter().max_by(|left, right| {
        pair_score(sim, faction_id, *left)
            .partial_cmp(&pair_score(sim, faction_id, *right))
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.cmp(left))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        CohesionSnapshot, FabricTier, Fixed, MilitaryUnit, Position, Simulation, UnitType,
        UnrestLevel, UnrestSnapshot,
    };
    use civ_agents::DiplomacySignal;

    #[test]
    fn high_unrest_faction_picks_unrest_action() {
        let mut sim = Simulation::with_seed(42);
        sim.last_tick_unrest_snapshots.insert(
            7,
            UnrestSnapshot {
                settlement_id: 7,
                level: UnrestLevel::Revolting,
                score: 300,
                events_count: 0,
                riots_count: 0,
                migrants_count: 0,
                mob_size: 0,
            },
        );
        let decisions = compute_faction_decisions(&sim);
        assert!(decisions
            .iter()
            .all(|(_, d)| *d == FactionDecision::RaiseUnrestResponse));
    }

    #[test]
    fn hostile_militarized_faction_flags_hostility() {
        let mut sim = Simulation::with_seed(42);
        // Drive relation score below -0.6 for faction 0 vs 1.
        for _ in 0..2 {
            sim.faction_relations.apply_signal(
                0u32,
                1u32,
                DiplomacySignal {
                    combat_grievance: 0.8,
                    ..DiplomacySignal::default()
                },
            );
        }
        sim.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(10),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 0, y: 0 },
            faction_id: 0,
        },));
        sim.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(10),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 1, y: 0 },
            faction_id: 0,
        },));
        sim.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(5),
            hp: Fixed::from_num(5),
            max_hp: Fixed::from_num(5),
            morale: Fixed::from_num(1),
            position: Position { x: 2, y: 0 },
            faction_id: 1,
        },));

        let decisions = compute_faction_decisions(&sim);
        let d0 = decisions
            .iter()
            .find(|(id, _)| *id == 0)
            .map(|(_, d)| *d);
        assert_eq!(d0, Some(FactionDecision::FlagHostility));
    }

    #[test]
    fn prosperous_friendly_faction_picks_trade() {
        let mut sim = Simulation::with_seed(42);
        sim.state.faction_resources.entry(0).or_default().food = Fixed::from_num(1500);
        sim.last_tick_cohesion_snapshots_mut().insert(
            1,
            CohesionSnapshot {
                settlement_id: 1,
                fabric: FabricTier::Tight,
                kin_count: 10,
                trust_sum: 100,
                fragmentation_events: 0,
                fragmentations: 0,
                faction_count: 1,
            },
        );
        sim.faction_relations.apply_signal(
            0u32,
            1u32,
            DiplomacySignal {
                trade_volume: 0.8,
                ..DiplomacySignal::default()
            },
        );

        let decisions = compute_faction_decisions(&sim);
        let d0 = decisions
            .iter()
            .find(|(id, _)| *id == 0)
            .map(|(_, d)| *d);
        assert_eq!(d0, Some(FactionDecision::FlagTradeOpen));
    }

    #[test]
    fn hostility_intent_applies_relation_score_and_conflict_event() {
        let mut sim = Simulation::with_seed(42);
        sim.faction_relations.apply_signal(
            0u32,
            1u32,
            DiplomacySignal {
                combat_grievance: 0.4,
                ..DiplomacySignal::default()
            },
        );
        let before = sim
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .unwrap_or(0.0);
        sim.state.last_tick_faction_hostility_intents.insert(0);

        apply_faction_decision_intents(&mut sim);

        let after = sim
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("hostility intent must materialize a relation row");
        assert!(after < before);
        assert!(sim.diplomacy_events().iter().any(|event| {
            event.kind == DiplomacyKind::Conflict
                && event.faction_a == 0
                && event.faction_b == 1
        }));
    }

    #[test]
    fn trade_open_intent_surfaces_trade_agreement_event() {
        let mut sim = Simulation::with_seed(42);
        sim.faction_relations.apply_signal(
            0u32,
            1u32,
            DiplomacySignal {
                trade_volume: 0.8,
                ..DiplomacySignal::default()
            },
        );
        sim.state.last_tick_faction_trade_open_intents.insert(0);

        apply_faction_decision_intents(&mut sim);

        let after = sim
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("trade intent must materialize a relation row");
        assert!(after > 0.0);
        assert!(sim.diplomacy_events().iter().any(|event| {
            event.kind == DiplomacyKind::TradeAgreement
                && event.faction_a == 0
                && event.faction_b == 1
        }));
    }
}
