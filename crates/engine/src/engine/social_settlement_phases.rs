//! Social & settlement phase implementations extracted from engine.rs (Pass 7).
//!
//! Contains civic institutions, social mood, stratification, cohesion,
//! unrest, daily-path, and cluster phases.

use super::{InstitutionEvent, MembershipPayoffTotals};
use crate::settlement_helpers::{
    settlement_centroid_position, SettlementMembershipPayoff, SETTLEMENT_CLUSTER_RADIUS_FP,
};
use crate::social_types::{
    compute_gini, institution_kind_key, CohesionEvent, CohesionEventKind, CohesionSnapshot,
    FabricTier, MoodSnapshot, OrderEvent, OrderLevel, OrderSnapshot, StratBand, StratQuantiles,
    StratificationEvent, StratificationEventKind, StratificationReport, UnrestEvent, UnrestLevel,
    UnrestSnapshot, MOOD_CRIME_BASE, MOOD_HISTORY_CAP, MOOD_MAX, MOOD_MIN,
};
use crate::Simulation;
use civ_agents::{
    cluster::{cluster_by_colocation, MembershipPayoff},
    daily_path::{pick_target, DailyPathDecision, Poi, PoiKind, PoiRegistry},
    Civilian as AgentCivilian, ClusterId, ClusterMember, Needs, Position3d,
};
use civ_voxel::FIXED_SCALE;
use std::collections::{BTreeMap, BTreeSet};

impl Simulation {
    /// Civic-institutions phase (FR-CIV-GOV-001/002/003).
    pub(crate) fn phase_institutions(&mut self) {
        let mut new_events = Vec::new();
        let settlement_ids: Vec<u32> = self.settlements.keys().copied().collect();
        for sid in settlement_ids {
            let pop = self.settlements[&sid];
            if pop >= civ_institutions::TEMPLE_UNLOCK_POPULATION {
                let new_level = if pop >= civ_institutions::TEMPLE_L2_POPULATION {
                    2
                } else {
                    1
                };
                self.upsert_institution(
                    sid,
                    civ_institutions::InstitutionKind::Temple,
                    new_level,
                    &mut new_events,
                );
            }
            if pop >= civ_institutions::GARRISON_UNLOCK_POPULATION {
                let new_level = if pop >= civ_institutions::GARRISON_L2_POPULATION {
                    2
                } else {
                    1
                };
                self.upsert_institution(
                    sid,
                    civ_institutions::InstitutionKind::Garrison,
                    new_level,
                    &mut new_events,
                );
            }
        }
        self.last_tick_institution_events = new_events;
    }

    pub(crate) fn upsert_institution(
        &mut self,
        sid: u32,
        kind: civ_institutions::InstitutionKind,
        new_level: u8,
        events: &mut Vec<InstitutionEvent>,
    ) {
        let key = (sid, institution_kind_key(kind), new_level);
        if self.institution_levels_emitted.contains(&key) {
            return;
        }
        self.institution_levels_emitted.insert(key);
        // Keep the first institution kind as the settlement's active civic
        // record. The current public storage is one record per settlement;
        // later kinds still emit their own events but do not overwrite the
        // existing Temple/Garrison mood source. Upgrades for that active kind
        // replace its level monotonically.
        let replace_active = match self.institutions.get(&sid) {
            None => true,
            Some(active) => active.kind == kind && new_level > active.level,
        };
        if replace_active {
            self.institutions.insert(
                sid,
                civ_institutions::Institution {
                    kind,
                    level: new_level,
                },
            );
        }
        events.push(InstitutionEvent {
            kind,
            level: new_level,
            settlement_id: sid,
        });
    }

    pub(crate) fn phase_social_mood(&mut self) {
        let mut snapshots: Vec<MoodSnapshot> = Vec::with_capacity(self.settlements.len());
        for (&settlement_id, &population) in &self.settlements {
            let stocked = self
                .settlement_food_stocked
                .get(&settlement_id)
                .cloned()
                .unwrap_or(0);
            let capacity = self
                .settlement_housing_capacity
                .get(&settlement_id)
                .cloned()
                .unwrap_or(0);
            let crime_pressure = self
                .settlement_crime_pressure
                .get(&settlement_id)
                .cloned()
                .unwrap_or(0);

            let (temple_bonus, garrison_bonus) =
                institution_mood_bonuses(self.institutions.get(&settlement_id));
            let parts = compute_mood_parts(
                population,
                stocked,
                capacity,
                crime_pressure,
                temple_bonus,
                garrison_bonus,
            );

            let prev = self
                .mood_history
                .iter()
                .rev()
                .find(|s| s.settlement_id == settlement_id)
                .map(|s| s.mood)
                .unwrap_or(0);
            let mood_delta = parts.total - prev;

            snapshots.push(MoodSnapshot {
                settlement_id,
                mood: parts.total,
                mood_delta,
                food_score: parts.food_score,
                housing_score: parts.housing_score,
                crime_score: parts.crime_score,
                temple_bonus: parts.temple_bonus,
                garrison_bonus: parts.garrison_bonus,
            });
        }

        snapshots.sort_by_key(|s| s.settlement_id);

        for snap in &snapshots {
            let history = self
                .mood_history_by_settlement
                .entry(snap.settlement_id)
                .or_insert_with(Vec::new);
            history.push(*snap);
            if history.len() > MOOD_HISTORY_CAP {
                let drop = history.len() - MOOD_HISTORY_CAP;
                history.drain(0..drop);
            }
        }
        self.mood_history.extend(snapshots.iter().copied());
        if self.mood_history.len() > MOOD_HISTORY_CAP * 8 {
            let drop = self.mood_history.len() - MOOD_HISTORY_CAP * 8;
            self.mood_history.drain(0..drop);
        }
        self.last_tick_mood = snapshots;
    }

    /// Stratification phase (FR-CIV-GOV-200 family).
    pub fn phase_stratification(&mut self) {
        let tick = self.state.tick;
        let settlement_ids: Vec<u32> = self.settlements.keys().copied().collect();

        for settlement_id in settlement_ids {
            let Some(household_ids) = self.settlement_households.get(&settlement_id) else {
                continue;
            };
            let household_ids: Vec<u64> = household_ids.iter().copied().collect();

            let mut wealths: Vec<(u64, i64)> = Vec::with_capacity(household_ids.len());
            for hid in &household_ids {
                let wealth = self.household_wealth.get(hid).copied().unwrap_or(0);
                wealths.push((*hid, wealth));
            }

            wealths.sort_by_key(|(_, w)| *w);
            let n = wealths.len();
            let q20_idx = n / 5;
            let q40_idx = (2 * n) / 5;
            let q60_idx = (3 * n) / 5;
            let q80_idx = (4 * n) / 5;

            let mut quantiles = StratQuantiles::default();
            let mut poor = 0u32;
            let mut middle = 0u32;
            let mut rich = 0u32;
            let mut elite = 0u32;
            for (i, (hid, w)) in wealths.iter().enumerate() {
                let band = if i < q20_idx {
                    poor += 1;
                    StratBand::Poor
                } else if i < q40_idx {
                    middle += 1;
                    StratBand::Middle
                } else if i < q60_idx {
                    middle += 1;
                    StratBand::Middle
                } else if i < q80_idx {
                    rich += 1;
                    StratBand::Rich
                } else {
                    elite += 1;
                    StratBand::Elite
                };
                quantiles.add(band);

                let score = *w;
                let prev_score = self.household_score.get(hid).copied().unwrap_or(0);
                let score_delta = score - prev_score;

                let key = (settlement_id, *hid, band);
                if !self.stratification_bands_emitted.contains(&key) {
                    self.stratification_bands_emitted.insert(key);
                    let kind = if score > prev_score {
                        StratificationEventKind::Promoted
                    } else if score < prev_score {
                        StratificationEventKind::Demoted
                    } else {
                        StratificationEventKind::Unchanged
                    };
                    self.last_tick_stratification.push(StratificationEvent {
                        household_id: *hid,
                        kind,
                        band,
                        score,
                        score_delta,
                    });
                }

                self.household_score.insert(*hid, score);
                self.household_bands.insert(*hid, band);
            }

            let gini = compute_gini(&wealths.iter().map(|(_, w)| *w).collect::<Vec<_>>());
            quantiles.poor = poor;
            quantiles.middle = middle;
            quantiles.rich = rich;
            quantiles.elite = elite;
            self.last_tick_stratification_reports.insert(
                settlement_id,
                StratificationReport {
                    settlement_id,
                    quantiles,
                    gini,
                    class_mobility_count: 0,
                    tick,
                },
            );
        }
    }

    /// Cohesion phase (FR-CIV-COHESION-001).
    pub(crate) fn phase_cohesion(&mut self) {
        self.last_tick_cohesion_events.clear();
        let mut new_snapshots: BTreeMap<u32, CohesionSnapshot> = BTreeMap::new();

        let mut settlement_actors: BTreeMap<u32, Vec<u64>> = BTreeMap::new();
        for (&actor_id, &sid) in &self.actor_settlement {
            settlement_actors.entry(sid).or_default().push(actor_id);
        }

        for (&settlement_id, actor_ids) in &settlement_actors {
            let mut kin_count: u64 = 0;
            let mut trust_sum: i64 = 0;
            let mut fragmentations: u64 = 0;

            for &actor_id in actor_ids {
                if let Some(edges) = self.kinship.get(&actor_id) {
                    kin_count += edges.len() as u64;
                }
                if let Some(trusts) = self.trust.get(&actor_id) {
                    trust_sum += trusts.values().sum::<i64>();
                }

                let hardship = self.actor_hardship.get(&actor_id).copied().unwrap_or(0);
                let (has_temple, has_garrison) = self
                    .actor_institutions
                    .get(&actor_id)
                    .copied()
                    .unwrap_or((false, false));

                let edge_count = self.kinship.get(&actor_id).map_or(0, |e| e.len());
                let trust = self
                    .trust
                    .get(&actor_id)
                    .map_or(0i64, |t| t.values().sum::<i64>());
                let fabric_score: i64 = (edge_count as i64 * 10) + trust - hardship.max(0)
                    + if has_temple { 30 } else { 0 }
                    + if has_garrison { 20 } else { 0 };

                let prev_fabric = self.last_actor_fabric.get(&actor_id).copied().unwrap_or(0);
                let delta = fabric_score - prev_fabric;
                self.last_actor_fabric.insert(actor_id, fabric_score);

                if delta.abs() > 5 {
                    let kind = if delta > 0 {
                        CohesionEventKind::Strengthened
                    } else {
                        CohesionEventKind::Weakened
                    };
                    self.last_tick_cohesion_events.push(CohesionEvent {
                        actor_id,
                        settlement_id,
                        kind,
                        score: fabric_score,
                        score_delta: delta,
                        fabric: FabricTier::from_score(fabric_score),
                    });
                }

                if fabric_score < -50 {
                    fragmentations += 1;
                    self.last_tick_cohesion_events.push(CohesionEvent {
                        actor_id,
                        settlement_id,
                        kind: CohesionEventKind::Fragmented,
                        score: fabric_score,
                        score_delta: delta,
                        fabric: FabricTier::from_score(fabric_score),
                    });
                }
            }

            let snapshot = CohesionSnapshot {
                settlement_id,
                fabric: FabricTier::from_score(
                    actor_ids
                        .iter()
                        .map(|a| self.last_actor_fabric.get(a).copied().unwrap_or(0))
                        .sum::<i64>()
                        / actor_ids.len().max(1) as i64,
                ),
                kin_count,
                trust_sum,
                fragmentation_events: fragmentations as u32,
                fragmentations,
                faction_count: settlement_actors
                    .get(&settlement_id)
                    .map_or(0, |ids| ids.len() as u64),
            };
            new_snapshots.insert(settlement_id, snapshot);
        }
        self.last_tick_cohesion = new_snapshots.clone();
        self.last_tick_cohesion_snapshots = new_snapshots;
    }

    /// Unrest phase (FR-CIV-UNREST-001).
    ///
    /// FR-CIV-phase-reconcile: this phase runs BEFORE `phase_social_mood`, so
    /// the per-tick `last_tick_mood` buffer is empty when we look at it. We
    /// therefore compute the per-settlement mood inline from the same inputs
    /// (`settlement_food_stocked`, `settlement_housing_capacity`,
    /// `settlement_crime_pressure`, `institutions`) that `phase_social_mood`
    /// consumes — see [`compute_mood_parts`]. This keeps the same-tick mood
    /// coupling (unrest reacts to the current tick's mood) while letting
    /// `last_tick_mood` stay empty at the top of every tick, which is the
    /// invariant the long-running `population_curve` test depends on.
    pub(crate) fn phase_unrest(&mut self) {
        self.last_tick_unrest_events.clear();
        let mut new_snapshots: BTreeMap<u32, UnrestSnapshot> = BTreeMap::new();

        let mut settlement_ids: BTreeSet<u32> = BTreeSet::new();
        for (&sid, _) in &self.settlement_gini {
            settlement_ids.insert(sid);
        }
        for (_, &sid) in &self.actor_settlement {
            settlement_ids.insert(sid);
        }

        for &settlement_id in &settlement_ids {
            // FR-CIV-phase-reconcile: compute mood inline (see fn doc above).
            // `phase_social_mood` runs later this tick and will repopulate
            // `self.last_tick_mood`, but we cannot depend on that ordering.
            let mood: i32 = self.mood_parts_for_unrest(settlement_id);

            let gini_x100 = self
                .settlement_gini
                .get(&settlement_id)
                .copied()
                .unwrap_or(0i32);

            let fabric = self
                .last_tick_cohesion_snapshots
                .get(&settlement_id)
                .map_or(FabricTier::Tight, |s| s.fabric);

            let fabric_x100: i32 = match &fabric {
                FabricTier::Tight => 0,
                FabricTier::Loosened => 50,
                FabricTier::Strained => 100,
                FabricTier::Fractured => 200,
            };

            let score: i32 = (200i32.saturating_sub(mood))
                .saturating_add(gini_x100.saturating_div(4))
                .saturating_add(fabric_x100.saturating_div(2))
                .max(0);

            let level = UnrestLevel::from_score(score);
            let prev_level = self
                .last_tick_unrest_levels
                .get(&settlement_id)
                .cloned()
                .unwrap_or(UnrestLevel::Stable);
            let level_delta = level.to_rank() as i32 - prev_level.to_rank() as i32;

            let mut riots_count: u64 = 0;
            let mut migrants_count: u64 = 0;

            if prev_level < UnrestLevel::Rioting && level >= UnrestLevel::Rioting {
                riots_count = (score as u64).saturating_div(30).max(1);
                self.last_tick_unrest_events.push(UnrestEvent {
                    settlement_id,
                    level: level.clone(),
                    score,
                    score_delta: level_delta,
                    mood,
                    gini_x100,
                    fabric,
                });
            }
            if prev_level <= UnrestLevel::Restless && level >= UnrestLevel::Rioting {
                migrants_count = (score as u64).saturating_div(50).max(1);
            }
            if level <= UnrestLevel::Stable && prev_level > UnrestLevel::Stable {
                self.last_tick_unrest_events.push(UnrestEvent {
                    settlement_id,
                    level: level.clone(),
                    score,
                    score_delta: level_delta,
                    mood,
                    gini_x100,
                    fabric,
                });
            }

            let snapshot = UnrestSnapshot {
                settlement_id,
                level,
                score,
                events_count: riots_count.saturating_add(migrants_count) as u32,
                riots_count,
                migrants_count,
                mob_size: riots_count.saturating_add(migrants_count),
            };
            new_snapshots.insert(settlement_id, snapshot);
            self.last_tick_unrest_levels.insert(settlement_id, level);
        }
        self.last_tick_unrest_snapshots = new_snapshots;
        // Keep the public per-tick event stream in sync with the internal
        // phase buffer consumed by the engine's wire/snapshot adapters.
        self.last_tick_unrest = self.last_tick_unrest_events.clone();
    }

    /// Daily-path phase (FR-CIV-LIFE-001 / FR-CIV-LIFE-002).
    pub(crate) fn phase_daily_path(&mut self) {
        use civ_agents::daily_path::path_step;

        self.last_tick_daily_path.clear();

        let mut registry = PoiRegistry::default();
        for (&settlement_id, &population) in self.settlements.iter() {
            let Some(pos) = settlement_centroid_position(&self.world, u64::from(settlement_id))
            else {
                continue;
            };
            for (offset, kind) in [
                (0_u64, PoiKind::FoodSource),
                (1, PoiKind::WaterSource),
                (2, PoiKind::Shelter),
                (3, PoiKind::SafeZone),
                (4, PoiKind::SocialHub),
                (5, PoiKind::Clinic),
            ] {
                registry.add(Poi {
                    id: u64::from(settlement_id) * 10 + offset,
                    kind,
                    pos,
                    capacity: population.max(1),
                });
            }
        }

        for (_, (pos, needs)) in self.world.query::<(&Position3d, &Needs)>().iter() {
            let life_needs = civ_needs::Needs {
                food: needs.food,
                water: needs.food,
                rest: needs.shelter,
                safety: needs.safety,
                social: needs.belonging,
                health: needs.safety,
            };
            if let Some(poi) = pick_target(&life_needs, &registry, pos) {
                let step = path_step(pos, &poi.pos, FIXED_SCALE / 4);
                self.last_tick_daily_path.push(DailyPathDecision {
                    poi_kind: poi.kind,
                    target_x: (step.coord.x / FIXED_SCALE) as i32,
                    target_z: (step.coord.z / FIXED_SCALE) as i32,
                });
            }
        }
    }

    /// Cluster-phase.
    pub(crate) fn phase_cluster(&mut self) {
        let positions: Vec<(u64, Position3d)> = self
            .world
            .query::<(&AgentCivilian, &Position3d)>()
            .iter()
            .map(|(_, (civilian, pos))| (civilian.id, *pos))
            .collect();
        let clusters = cluster_by_colocation(&positions, SETTLEMENT_CLUSTER_RADIUS_FP);
        let cluster_by_agent: BTreeMap<u64, ClusterId> = clusters.into_iter().collect();
        let payoff = SettlementMembershipPayoff {
            stock_by_cluster: &self.cluster_stocks,
        };
        let mut totals: BTreeMap<u64, MembershipPayoffTotals> = BTreeMap::new();

        let mut updates = Vec::new();
        for (entity, civilian) in self.world.query::<&AgentCivilian>().iter() {
            let cluster = cluster_by_agent
                .get(&civilian.id)
                .copied()
                .unwrap_or(ClusterId(civilian.id));
            let value = payoff.payoff(civilian.id, cluster);
            let entry = totals.entry(cluster.0).or_insert(MembershipPayoffTotals {
                cluster_id: cluster.0,
                members: 0,
                total_payoff: 0.0,
            });
            entry.members = entry.members.saturating_add(1);
            entry.total_payoff += value;
            updates.push((entity, cluster));
        }

        for (entity, cluster) in updates {
            if self.world.get::<&ClusterMember>(entity).is_ok() {
                let mut member = self
                    .world
                    .get::<&mut ClusterMember>(entity)
                    .expect("cluster member checked above");
                member.cluster = cluster;
            } else {
                let _ = self.world.insert_one(entity, ClusterMember { cluster });
            }
        }

        self.last_tick_cluster_payoffs = totals.into_values().collect();
    }

    /// FR-CIV-phase-reconcile: read the settlement's mood inputs and return
    /// the total mood (post-saturation, signed) that `phase_unrest` will
    /// use for its unrest score. This is the per-settlement inline mood
    /// computation that replaces reading `self.last_tick_mood` (which is
    /// empty when `phase_unrest` runs because `phase_social_mood` hasn't
    /// fired yet this tick).
    fn mood_parts_for_unrest(&self, settlement_id: u32) -> i32 {
        let population = self.settlements.get(&settlement_id).copied().unwrap_or(0);
        let stocked = self
            .settlement_food_stocked
            .get(&settlement_id)
            .copied()
            .unwrap_or(0);
        let capacity = self
            .settlement_housing_capacity
            .get(&settlement_id)
            .copied()
            .unwrap_or(0);
        let crime_pressure = self
            .settlement_crime_pressure
            .get(&settlement_id)
            .copied()
            .unwrap_or(0);
        let (temple_bonus, garrison_bonus) =
            institution_mood_bonuses(self.institutions.get(&settlement_id));
        let parts = compute_mood_parts(
            population,
            stocked,
            capacity,
            crime_pressure,
            temple_bonus,
            garrison_bonus,
        );
        parts.total as i32
    }

    /// Order / government-stability phase (FR-CIV-ORDER-001).
    ///
    /// Runs **after** `phase_unrest` and `phase_institutions` so it can read
    /// the just-computed per-settlement unrest score and the institution kind
    /// + level that will govern legitimacy. The phase computes
    ///
    /// `revolt_likelihood(unrest_norm, legitimacy)` per settlement using the
    /// same helper the unit tests in [`crate::emergence`] cover, then bins
    /// the result into [`OrderLevel`] and emits an event on level changes.
    ///
    /// Legitimacy model:
    ///   * No institution: `0.0`
    ///   * Temple L1/L2: `0.35 / 0.55` (cultural legitimacy cushion)
    ///   * Garrison L1/L2: `0.20 / 0.30` (coercive legitimacy cushion)
    ///
    /// `unrest_norm` is `min(unrest_score / 500, 1.0)`. A settlement that
    /// escalates into `UnrestLevel::Rioting` (score >= 150) and holds no
    /// institution crosses `OrderLevel::Strained` within a single tick.
    pub(crate) fn phase_order(&mut self) {
        self.last_tick_order_events.clear();
        let mut new_snapshots: BTreeMap<u32, OrderSnapshot> = BTreeMap::new();

        // Iterate over the union of (settlements with an unrest snapshot) ∪
        // (settlements that hold an institution this tick). Either set alone
        // must drive an order snapshot.
        let mut settlement_ids: BTreeSet<u32> = BTreeSet::new();
        for &sid in self.last_tick_unrest_snapshots.keys() {
            settlement_ids.insert(sid);
        }
        for &sid in self.institutions.keys() {
            settlement_ids.insert(sid);
        }

        for &settlement_id in &settlement_ids {
            let unrest_score = self
                .last_tick_unrest_snapshots
                .get(&settlement_id)
                .map_or(0, |s| s.score);
            let unrest_norm: f32 = (unrest_score as f32 / 500.0).clamp(0.0, 1.0);

            // Institution-derived legitimacy cushion (FR-CIV-ORDER-001 spec).
            let (legitimacy, institution_level, institution_kind) =
                match self.institutions.get(&settlement_id) {
                    Some(inst) if inst.kind == civ_institutions::InstitutionKind::Temple => {
                        let base = if inst.level >= 2 { 0.55 } else { 0.35 };
                        (base, inst.level, 1u8)
                    }
                    Some(inst) if inst.kind == civ_institutions::InstitutionKind::Garrison => {
                        let base = if inst.level >= 2 { 0.30 } else { 0.20 };
                        (base, inst.level, 2u8)
                    }
                    _ => (0.0_f32, 0u8, 0u8),
                };

            let likelihood = crate::emergence::revolt_likelihood(unrest_norm, legitimacy);
            let level = OrderLevel::from_revolt_likelihood(likelihood);

            let prev_level = self
                .last_tick_order_levels
                .get(&settlement_id)
                .copied()
                .unwrap_or(OrderLevel::Holding);
            let level_delta = level.to_rank() as i32 - prev_level.to_rank() as i32;

            // Emit only on level change (mirrors `phase_unrest` event policy)
            // so wire consumers don't see a flood of "still Holding" rows.
            if level != prev_level {
                self.last_tick_order_events.push(OrderEvent {
                    settlement_id,
                    level,
                    level_delta,
                    revolt_likelihood: likelihood,
                    unrest_score,
                    legitimacy,
                    institution_level,
                    institution_kind,
                });
            }

            let snapshot = OrderSnapshot {
                settlement_id,
                level,
                revolt_likelihood: likelihood,
                unrest_score,
                legitimacy,
                institution_level,
                institution_kind,
            };
            new_snapshots.insert(settlement_id, snapshot);
            self.last_tick_order_levels.insert(settlement_id, level);
        }
        self.last_tick_order_snapshots = new_snapshots;
        // Keep the public per-tick event stream in sync with the internal
        // phase buffer (same pattern as `phase_unrest`).
        self.last_tick_order = self.last_tick_order_events.clone();
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-phase-reconcile: shared mood helpers.
// `phase_social_mood` and `phase_unrest` both need to compute the same
// per-settlement mood total from the same inputs. To keep them in lockstep,
// the formula lives in one place: [`compute_mood_parts`].
// ---------------------------------------------------------------------------

/// Per-settlement mood sub-scores produced by [`compute_mood_parts`].
///
/// All sub-scores are pre-institution-bonus, pre-saturation where noted.
/// `total` is the post-saturation mood value used by both
/// `phase_social_mood` (as `MoodSnapshot.mood`) and `phase_unrest` (as the
/// `mood` field of its unrest score).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MoodParts {
    food_score: i64,
    housing_score: i64,
    crime_score: i64,
    temple_bonus: i32,
    garrison_bonus: i32,
    total: i64,
}

/// Compute the per-settlement mood sub-scores and the saturated total from
/// the same inputs `phase_social_mood` reads. Used by both `phase_social_mood`
/// (to populate `last_tick_mood`) and `phase_unrest` (to read the mood for
/// its unrest score — see FR-CIV-phase-reconcile note in [`Simulation::phase_unrest`]).
fn compute_mood_parts(
    population: u32,
    food_stocked: i64,
    housing_capacity: u32,
    crime_pressure: i32,
    temple_bonus: i32,
    garrison_bonus: i32,
) -> MoodParts {
    let food_score = (food_stocked / 200).clamp(MOOD_MIN, MOOD_MAX);
    let housing_signed = (housing_capacity as i64)
        .saturating_sub(population as i64)
        .saturating_mul(2);
    let housing_score = housing_signed.clamp(MOOD_MIN, MOOD_MAX);
    let crime_signed = MOOD_CRIME_BASE.saturating_sub(4 * crime_pressure as i64);
    let crime_score = crime_signed.clamp(0, MOOD_CRIME_BASE);
    let total = food_score
        .saturating_add(housing_score)
        .saturating_add(crime_score)
        .saturating_add(temple_bonus as i64)
        .saturating_add(garrison_bonus as i64)
        .clamp(MOOD_MIN, MOOD_MAX);
    MoodParts {
        food_score,
        housing_score,
        crime_score,
        temple_bonus,
        garrison_bonus,
        total,
    }
}

/// Resolve `(temple_bonus, garrison_bonus)` from a settlement's optional
/// institution record. `phase_social_mood` and `phase_unrest` both need
/// this mapping; centralising it keeps the bonus formula in lockstep.
fn institution_mood_bonuses(
    institution: Option<&civ_institutions::Institution>,
) -> (i32, i32) {
    match institution {
        Some(inst) if inst.kind == civ_institutions::InstitutionKind::Temple => {
            (25 + 25 * (inst.level as i32), 0)
        }
        Some(inst) if inst.kind == civ_institutions::InstitutionKind::Garrison => {
            (0, 15 + 15 * (inst.level as i32))
        }
        _ => (0, 0),
    }
}
