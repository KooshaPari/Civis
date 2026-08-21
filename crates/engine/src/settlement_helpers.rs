//! Settlement helpers extracted from engine.rs for modularity.
//!
//! Emergent settlement analysis, diplomacy pair selection, trade route management,
//! and related helper functions.

use crate::culture::{culture_cooperation_signal, culture_openness_signal, FactionIdeologyState};
use crate::emergence_coupling::{
    religious_unity_peace_bonus, COHESION_BELIEF_DIVISOR, CULTURE_PEACE_SPAN,
    DIPLOMACY_BASE_CONFLICT_THRESHOLD, DIPLOMACY_COMPETITION_DRIFT, DIPLOMACY_TRADE_DRIFT,
    FACTION_RELATION_THRESHOLD_SPAN,
};
use crate::engine::{
    ClusterStocks, FabricTier, KinshipEdge, ResourceType, Resources, TradeRoute, WorldState,
};
use crate::fixed_math::Fixed;
use crate::religion::ReligiousProfile;
use crate::SCALE;
use civ_agents::culture::{cultural_distance, CultureProfile};
use civ_agents::diplomacy::GriefAccumulator;
use civ_agents::{
    Alignment, Civilian as AgentCivilian, ClusterId, ClusterMember, DiplomacyMatrix,
    DiplomacySignal, Position3d,
};
use civ_economy::{Good, SettlementTradeFlow};
use civ_voxel::{WorldCoord, FIXED_SCALE};
use hecs::World;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Minimum members for an emergent settlement (matches `phase_life` HUD filter).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const SETTLEMENT_MIN_MEMBERS: u32 = 2;
/// Co-location radius for emergent settlements (matches `phase_life` cluster radius).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const SETTLEMENT_CLUSTER_RADIUS_FP: i64 = (6 * FIXED_SCALE) / 100;
/// Contact radius between settlement pairs (2Ãƒâ€” cluster radius).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) const SETTLEMENT_CONTACT_RADIUS_FP: i64 = SETTLEMENT_CLUSTER_RADIUS_FP * 2;

#[allow(dead_code)] // Reserved for settlement membership analysis
pub(crate) struct SettlementMembershipPayoff<'a> {
    pub(crate) stock_by_cluster: &'a BTreeMap<u64, ClusterStocks>,
}

#[allow(dead_code)] // Reserved for settlement membership analysis
impl civ_agents::cluster::MembershipPayoff for SettlementMembershipPayoff<'_> {
    fn payoff(&self, _agent_id: u64, cluster: ClusterId) -> f32 {
        let food = self
            .stock_by_cluster
            .get(&cluster.0)
            .map(|stocks| stocks.get(civ_economy::Good::Food))
            .unwrap_or(0);
        (food as f32 / 10.0).clamp(-1.0, 1.0)
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_actors_by_settlement(
    actor_settlement: &BTreeMap<u64, u32>,
) -> BTreeMap<u32, BTreeSet<u64>> {
    let mut by_settlement: BTreeMap<u32, BTreeSet<u64>> = BTreeMap::new();
    for (&actor_id, &settlement_id) in actor_settlement {
        by_settlement
            .entry(settlement_id)
            .or_default()
            .insert(actor_id);
    }
    by_settlement
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_centroid_position(
    world: &World,
    settlement_id: u64,
) -> Option<Position3d> {
    let mut count = 0_i64;
    let mut sx = 0_i128;
    let mut sy = 0_i128;
    let mut sz = 0_i128;
    for (_, (member, pos)) in world.query::<(&ClusterMember, &Position3d)>().iter() {
        if member.cluster.0 == settlement_id {
            count += 1;
            sx += i128::from(pos.coord.x);
            sy += i128::from(pos.coord.y);
            sz += i128::from(pos.coord.z);
        }
    }
    (count > 0).then(|| Position3d {
        coord: WorldCoord {
            x: (sx / i128::from(count)) as i64,
            y: (sy / i128::from(count)) as i64,
            z: (sz / i128::from(count)) as i64,
        },
    })
}

/// Belief units required to raise the conflict threshold by one currency unit.
#[allow(dead_code)] // Reserved for future simulation integration
const BELIEF_PEACE_DIVISOR: u64 = 50;
/// Cap on the belief-driven peace bonus: shared faith can at most double a
/// society's tolerance for inequality Ã¢â‚¬â€ it never makes conflict impossible.
#[allow(dead_code)] // Reserved for future simulation integration
const BELIEF_PEACE_CAP: i64 = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
/// Unrest units required to erode the conflict threshold by one currency unit.
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_WAR_DIVISOR: u64 = 50;
/// Cap on how much unrest can erode the threshold (currency units).
#[allow(dead_code)] // Reserved for future simulation integration
const UNREST_WAR_CAP: i64 = 8_000;
/// Floor on the conflict threshold: even a furious, faithless society still
/// needs SOME wealth disparity to go to war Ã¢â‚¬â€ discontent alone is not casus belli.
#[allow(dead_code)] // Reserved for future simulation integration
const DIPLOMACY_MIN_CONFLICT_THRESHOLD: i64 = 2_000;

/// Downward-causation policy (FR-CIV-0100 Ã‚Â§3 emergence): collective belief and
/// societal unrest pull diplomacy in opposite directions. Shared faith RAISES
/// the wealth-disparity a faction pair tolerates before fighting (peace);
/// unrest LOWERS it (internal discontent spills into external aggression). The
/// threshold is bounded below by `DIPLOMACY_MIN_CONFLICT_THRESHOLD` so conflict
/// always needs some disparity, and above at `2x` base so peace is never absolute.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn diplomacy_conflict_threshold(belief: u64, unrest: u64) -> i64 {
    let peace = (belief / BELIEF_PEACE_DIVISOR).min(BELIEF_PEACE_CAP as u64) as i64;
    let war = (unrest / UNREST_WAR_DIVISOR).min(UNREST_WAR_CAP as u64) as i64;
    (DIPLOMACY_BASE_CONFLICT_THRESHOLD + peace - war).max(DIPLOMACY_MIN_CONFLICT_THRESHOLD)
}

/// Cohesion-driven peace bonus for diplomacy threshold (FR-CIV-RELIGION-002).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn cohesion_peace_bonus(cohesion: u64) -> i64 {
    (cohesion / COHESION_BELIEF_DIVISOR).min(BELIEF_PEACE_CAP as u64 / 2) as i64
}

/// Combined religionÃ¢â€ â€™diplomacy threshold: belief, cohesion, unrest, patron veneration.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn diplomacy_peace_threshold(belief: u64, cohesion: u64, unrest: u64, has_patron: bool) -> i64 {
    diplomacy_conflict_threshold(belief, unrest)
        + cohesion_peace_bonus(cohesion)
        + religious_unity_peace_bonus(has_patron)
}

/// Macro belief plus emergent cluster doctrine strength (FR-CIV-RELIGION / REL-003).
#[allow(dead_code)] // Reserved for future simulation integration
pub fn institution_belief_signal(
    macro_belief: u64,
    cluster_beliefs: &BTreeMap<u64, [f32; 4]>,
) -> u64 {
    if cluster_beliefs.is_empty() {
        return macro_belief;
    }
    let cluster_pulse: u64 = cluster_beliefs
        .values()
        .map(|centroid| (centroid[0] * 2_000.0) as u64)
        .sum();
    macro_belief.saturating_add(cluster_pulse / cluster_beliefs.len() as u64)
}

/// Cluster-divergence temple boost: isolated clusters that develop distinct doctrines
/// independently accelerate local institution growth (FR-CIV-RELIGION cluster divergence).
///
/// `divergence` is the max pairwise belief distance `[0.0, 1.0]` from
/// [`civ_agents::max_cluster_belief_divergence`]. Returns extra belief units to
/// add to the macro signal, capped so the boost can at most double the signal.
#[allow(dead_code)] // Reserved for future simulation integration
pub fn institution_divergence_boost(macro_signal: u64, divergence: f32) -> u64 {
    let bonus = (macro_signal as f32 * divergence.clamp(0.0, 1.0)) as u64;
    macro_signal.saturating_add(bonus)
}

/// Pairwise treasury gap between two factions (currency units).
#[allow(dead_code)] // Reserved for future simulation integration
fn faction_pair_treasury_disparity(treasury: &HashMap<u32, Fixed>, a: u32, b: u32) -> i64 {
    let va = treasury
        .get(&a)
        .map(|t| t.to_bits() / crate::SCALE)
        .unwrap_or(0);
    let vb = treasury
        .get(&b)
        .map(|t| t.to_bits() / crate::SCALE)
        .unwrap_or(0);
    (va - vb).abs()
}

/// N9: maximum reduction to the conflict threshold from maximum aggression.
const AGGRESSION_CONFLICT_BOOST: i64 = 3_000;

/// N9: conflict-threshold reduction driven by mean pairwise aggression.
/// Aggressive species are quicker to fight: a mean aggression of 1.0 reduces
/// the threshold by [`AGGRESSION_CONFLICT_BOOST`] currency units.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn aggression_threshold_reduction(mean: f32) -> i64 {
    (mean.clamp(0.0, 1.0) * AGGRESSION_CONFLICT_BOOST as f32) as i64
}

/// Threshold bias from emergent faction relation (`relation * 5000`, clamped).
#[allow(dead_code)] // Reserved for future simulation integration
fn diplomacy_relation_threshold_bias(relation_score: f32) -> i64 {
    (relation_score.clamp(-1.0, 1.0) * FACTION_RELATION_THRESHOLD_SPAN as f32).round() as i64
}

/// Peace bonus from pairwise cultural similarity (N2 Ã¢â‚¬â€ culture Ã¢â€ â€™ diplomacy).
///
/// Culturally similar factions tolerate more treasury disparity before conflict;
/// divergent pairs add zero bonus (neutral default).
#[allow(dead_code)] // Reserved for future simulation integration
fn diplomacy_culture_threshold_bias(
    cultures: &BTreeMap<u64, CultureProfile>,
    faction_a: u32,
    faction_b: u32,
) -> i64 {
    let Some(pa) = cultures.get(&u64::from(faction_a)) else {
        return 0;
    };
    let Some(pb) = cultures.get(&u64::from(faction_b)) else {
        return 0;
    };
    let distance = cultural_distance(pa.traits, pb.traits);
    let similarity = 1.0 - distance;
    (similarity * CULTURE_PEACE_SPAN).round() as i64
}

/// Dominant explicit faction alignment per multi-member settlement cluster (N3).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_dominant_factions(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
) -> BTreeMap<u64, u32> {
    let mut faction_counts: BTreeMap<u64, BTreeMap<u32, u32>> = BTreeMap::new();
    for (_, (civ, member)) in world.query::<(&AgentCivilian, &ClusterMember)>().iter() {
        let cluster_id = member.cluster.0;
        let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
        if members < SETTLEMENT_MIN_MEMBERS {
            continue;
        }
        if let Alignment::Faction(faction_id) = civ.alignment {
            *faction_counts
                .entry(cluster_id)
                .or_default()
                .entry(faction_id)
                .or_insert(0) += 1;
        }
    }

    let mut dominant = BTreeMap::new();
    for (cluster_id, counts) in faction_counts {
        let mut best_faction = None;
        let mut best_count = 0u32;
        for (&faction_id, &count) in &counts {
            let replace = match best_faction {
                None => true,
                Some(prev) => count > best_count || (count == best_count && faction_id < prev),
            };
            if replace {
                best_faction = Some(faction_id);
                best_count = count;
            }
        }
        if let Some(faction_id) = best_faction {
            dominant.insert(cluster_id, faction_id);
        }
    }
    dominant
}

/// Member-weighted per-faction language centroid (FR-CIV-LANG-001 / FR-CIV-PSYCHE-912).
///
/// `cluster_cultures` is `BTreeMap<u64, CultureProfile>` keyed by cluster id; each
/// profile carries a `language: [f32; 4]` vector. `dominant` maps cluster id
/// (u64) to its dominant faction id (u32) as returned by
/// [`settlement_dominant_factions`]. `member_counts` is the cluster membership
/// rollup from `phase_life`. Clusters with fewer than 2 members are ignored so
/// lone wanderers cannot anchor a faction's centroid.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn faction_language_centroids(
    cultures: &std::collections::BTreeMap<u64, CultureProfile>,
    dominant: &std::collections::BTreeMap<u64, u32>,
    member_counts: &std::collections::BTreeMap<u64, u32>,
) -> std::collections::BTreeMap<u32, [f32; 4]> {
    let mut sums: std::collections::BTreeMap<u32, ([f32; 4], f32)> = Default::default();
    for (cluster_id, faction_id) in dominant {
        let mc = match member_counts.get(cluster_id) {
            Some(&m) if m >= 2 => m,
            _ => continue,
        };
        let lang = match cultures.get(cluster_id) {
            Some(c) => c.language,
            None => continue,
        };
        let e = sums.entry(*faction_id).or_insert(([0.0; 4], 0.0));
        for a in 0..4 {
            e.0[a] += lang[a] * mc as f32;
        }
        e.1 += mc as f32;
    }
    sums.into_iter()
        .map(|(f, (s, w))| {
            let mut c = [0.0f32; 4];
            if w > 0.0 {
                for a in 0..4 {
                    c[a] = (s[a] / w).clamp(0.0, 1.0);
                }
            }
            (f, c)
        })
        .collect()
}

/// Member-weighted per-faction religion signal for culture drift (FR-CIV-CULTURE).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn faction_religion_signals(
    religious_profiles: &BTreeMap<u32, ReligiousProfile>,
    dominant: &BTreeMap<u64, u32>,
    member_counts: &BTreeMap<u64, u32>,
) -> BTreeMap<u32, f32> {
    let mut sums: BTreeMap<u32, (f32, f32)> = BTreeMap::new();
    for (&settlement_id, &faction_id) in dominant {
        let Some(profile) = religious_profiles.get(&(settlement_id as u32)) else {
            continue;
        };
        let weight = member_counts
            .get(&settlement_id)
            .copied()
            .unwrap_or(1)
            .max(1) as f32;
        let signal = (profile.monitoring * 0.40
            + profile.mythic_coherence * 0.40
            + profile.uncertainty_reduction * 0.20)
            .clamp(0.0, 1.0);
        let entry = sums.entry(faction_id).or_insert((0.0, 0.0));
        entry.0 += signal * weight;
        entry.1 += weight;
    }

    sums.into_iter()
        .filter_map(|(faction_id, (sum, weight))| {
            (weight > 0.0).then_some((faction_id, (sum / weight).clamp(0.0, 1.0)))
        })
        .collect()
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn fabric_tier_signal(fabric: FabricTier) -> f32 {
    match fabric {
        FabricTier::Tight => 1.0,
        FabricTier::Loosened => 0.7,
        FabricTier::Strained => 0.35,
        FabricTier::Fractured => 0.1,
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_actor_hardship_signal(
    settlement_id: u32,
    settlement_actors: &BTreeMap<u32, BTreeSet<u64>>,
    actor_hardship: &BTreeMap<u64, i64>,
) -> f32 {
    let Some(actors) = settlement_actors.get(&settlement_id) else {
        return 0.0;
    };
    if actors.is_empty() {
        return 0.0;
    }
    let sum: i64 = actors
        .iter()
        .map(|actor_id| actor_hardship.get(actor_id).copied().unwrap_or(0).max(0))
        .sum();
    ((sum as f32 / actors.len() as f32) / 1000.0).clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_kinship_density_signal(
    settlement_id: u32,
    settlement_actors: &BTreeMap<u32, BTreeSet<u64>>,
    kinship: &BTreeMap<u64, Vec<KinshipEdge>>,
) -> f32 {
    let Some(actors) = settlement_actors.get(&settlement_id) else {
        return 1.0;
    };
    if actors.len() <= 1 {
        return 1.0;
    }
    let possible = (actors.len() * (actors.len() - 1)) as f32;
    let internal_edges = actors
        .iter()
        .filter_map(|actor_id| kinship.get(actor_id))
        .flat_map(|edges| edges.iter())
        .filter(|edge| actors.contains(&edge.target))
        .count() as f32;
    (internal_edges / possible).clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_trade_contact_signal(
    settlement_id: u32,
    flows: &[SettlementTradeFlow],
) -> f32 {
    let volume: i64 = flows
        .iter()
        .filter(|flow| {
            flow.from_settlement == u64::from(settlement_id)
                || flow.to_settlement == u64::from(settlement_id)
        })
        .map(|flow| flow.qty.max(0))
        .sum();
    (volume as f32 / 100.0).clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_religion_spread_edges(
    flows: &[SettlementTradeFlow],
) -> BTreeMap<(u32, u32), f32> {
    let mut edges = BTreeMap::new();
    for flow in flows {
        if flow.from_settlement == flow.to_settlement || flow.qty <= 0 {
            continue;
        }
        // Cast SettlementId (u64) Ã¢â€ â€™ u32; the caller enforces the
        // `u32::MAX` truncation guard before insertion in
        // `spread_religion_between_settlements`.
        let a = flow.from_settlement.min(flow.to_settlement) as u32;
        let b = flow.from_settlement.max(flow.to_settlement) as u32;
        let strength = (flow.qty as f32 / 100.0).clamp(0.05, 1.0);
        edges
            .entry((a, b))
            .and_modify(|existing: &mut f32| *existing = existing.max(strength))
            .or_insert(strength);
    }
    edges
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn accumulate_profile_diffusion(
    left: &ReligiousProfile,
    right: &ReligiousProfile,
    rate: f32,
    deltas: &mut BTreeMap<u32, (f32, f32, f32)>,
    left_id: u32,
    right_id: u32,
) {
    let dm = (right.monitoring - left.monitoring) * rate;
    let dc = (right.mythic_coherence - left.mythic_coherence) * rate;
    let du = (right.uncertainty_reduction - left.uncertainty_reduction) * rate;
    let left_delta = deltas.entry(left_id).or_insert((0.0, 0.0, 0.0));
    left_delta.0 += dm;
    left_delta.1 += dc;
    left_delta.2 += du;
    let right_delta = deltas.entry(right_id).or_insert((0.0, 0.0, 0.0));
    right_delta.0 -= dm;
    right_delta.1 -= dc;
    right_delta.2 -= du;
}

/// Canonical settlement contact edges when any cross-cluster agents are within radius (N3).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_contact_pairs(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    contact_radius_fp: i64,
) -> BTreeSet<(u64, u64)> {
    let contact_radius_sq = i128::from(contact_radius_fp) * i128::from(contact_radius_fp);
    let mut by_cluster: BTreeMap<u64, Vec<(i64, i64)>> = BTreeMap::new();
    for (_, (member, pos)) in world.query::<(&ClusterMember, &Position3d)>().iter() {
        let cluster_id = member.cluster.0;
        let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
        if members < SETTLEMENT_MIN_MEMBERS {
            continue;
        }
        by_cluster
            .entry(cluster_id)
            .or_default()
            .push((pos.coord.x, pos.coord.z));
    }

    let cluster_ids: Vec<u64> = by_cluster.keys().copied().collect();
    let mut contacts = BTreeSet::new();
    for i in 0..cluster_ids.len() {
        for j in (i + 1)..cluster_ids.len() {
            let ca = cluster_ids[i];
            let cb = cluster_ids[j];
            let Some(agents_a) = by_cluster.get(&ca) else {
                continue;
            };
            let Some(agents_b) = by_cluster.get(&cb) else {
                continue;
            };
            let in_contact = agents_a.iter().any(|&(ax, az)| {
                agents_b.iter().any(|&(bx, bz)| {
                    let dx = i128::from(ax) - i128::from(bx);
                    let dz = i128::from(az) - i128::from(bz);
                    dx * dx + dz * dz <= contact_radius_sq
                })
            });
            if in_contact {
                contacts.insert((ca.min(cb), ca.max(cb)));
            }
        }
    }
    contacts
}

/// Faction pairs implied by contacting settlements with different dominant factions (N3).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn diplomacy_faction_pairs_from_settlement_contact(
    dominant: &BTreeMap<u64, u32>,
    contacts: &BTreeSet<(u64, u64)>,
) -> Vec<(u32, u32)> {
    let mut pairs = BTreeSet::new();
    for &(ca, cb) in contacts {
        let Some(&fa) = dominant.get(&ca) else {
            continue;
        };
        let Some(&fb) = dominant.get(&cb) else {
            continue;
        };
        if fa != fb {
            pairs.insert((fa.min(fb), fa.max(fb)));
        }
    }
    pairs.into_iter().collect()
}

/// Select diplomacy faction pair from settlement contact, then presence, then registry (N3).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn diplomacy_pair_from_settlement_overlap(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    registered_factions: &[u32],
    tick: u64,
) -> (u32, u32) {
    let dominant = settlement_dominant_factions(world, cluster_member_counts);
    let contacts =
        settlement_contact_pairs(world, cluster_member_counts, SETTLEMENT_CONTACT_RADIUS_FP);
    let pairs = diplomacy_faction_pairs_from_settlement_contact(&dominant, &contacts);
    if !pairs.is_empty() {
        let idx = (tick as usize / 500) % pairs.len();
        return pairs[idx];
    }

    let present: Vec<u32> = dominant
        .values()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if present.len() >= 2 {
        let idx = tick as usize % present.len();
        let a = present[idx];
        let b = present[(idx + 1) % present.len()];
        return (a, b);
    }

    let idx = tick as usize;
    let a = registered_factions[idx % registered_factions.len()];
    let b = registered_factions[(idx + 1) % registered_factions.len()];
    (a, b)
}

/// Scales every stored relation toward neutral without overshooting zero.
///
/// [`DiplomacyMatrix`] has no native decay; calibrated `apply_signal` calls
/// achieve `score * factor` per pair (FR-CIV-0100 criticality).
#[allow(dead_code)] // Reserved for future simulation integration
fn decay_faction_relations(matrix: &mut DiplomacyMatrix, factor: f32) {
    let factor = factor.clamp(0.0, 1.0);
    let pairs = matrix.snapshot();
    for (a, b, record) in pairs {
        let score = record.score;
        if score == 0.0 {
            continue;
        }
        let target = score * factor;
        let delta = target - score;
        if delta > 0.0 {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    trade_volume: delta / DIPLOMACY_TRADE_DRIFT,
                    ..Default::default()
                },
            );
        } else {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    resource_competition: (-delta) / DIPLOMACY_COMPETITION_DRIFT,
                    ..Default::default()
                },
            );
        }
    }
}

/// Sustained [`DiplomacyKind::TradeAgreement`] events before an emergent route is born.
#[allow(dead_code)] // Reserved for future simulation integration
const TRADE_ROUTE_AGREEMENT_BIRTH_THRESHOLD: u32 = 2;
/// Minimum pairwise relation score required to birth an emergent route.
#[allow(dead_code)] // Reserved for future simulation integration
const TRADE_ROUTE_MIN_RELATION: f32 = 0.0;
/// Hard cap on total trade routes (bootstrap + emergent) to bound memory and tick cost.
#[allow(dead_code)] // Reserved for future simulation integration
const MAX_TRADE_ROUTES: usize = 64;
/// Ticks without resource flow before an emergent route is removed.
#[allow(dead_code)] // Reserved for future simulation integration
const TRADE_ROUTE_UNUSED_DECAY_TICKS: u32 = 2_000;

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn canonical_faction_pair(a: u32, b: u32) -> (u32, u32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Map a registered faction id to the diplomacy matrix cluster key (N3 bridge).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn faction_cluster_id(faction: u32) -> u32 {
    faction
}

/// Round-robin pair selection over the static faction registry (tests / fallback).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn diplomacy_faction_pair(faction_ids: &[u32], tick: u64) -> (u32, u32) {
    if faction_ids.len() < 2 {
        return (0, 0);
    }
    let idx = tick as usize % faction_ids.len();
    let a = faction_ids[idx];
    let b = faction_ids[(idx + 1) % faction_ids.len()];
    (a, b)
}

#[allow(dead_code)] // Reserved for future simulation integration
fn all_registered_faction_pairs(faction_ids: &[u32]) -> Vec<(u32, u32)> {
    let mut pairs = Vec::new();
    for i in 0..faction_ids.len() {
        for j in (i + 1)..faction_ids.len() {
            pairs.push(canonical_faction_pair(faction_ids[i], faction_ids[j]));
        }
    }
    pairs
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn rollup_cluster_member_counts(world: &World) -> BTreeMap<u64, u32> {
    let mut counts = BTreeMap::new();
    for (_, member) in world.query::<&ClusterMember>().iter() {
        *counts.entry(member.cluster.0).or_insert(0) += 1;
    }
    counts
}

/// Per-cluster member count (alias of [`rollup_cluster_member_counts`]).
/// Stub: same shape so callers can treat settlement/cluster membership
/// uniformly until the engine fully merges the two.
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn settlement_member_counts(world: &World) -> BTreeMap<u64, u32> {
    rollup_cluster_member_counts(world)
}

/// Map a `ResourceType` to its market-state key (coerced to a `&'static str`
/// for the market bus).
#[must_use]
#[allow(dead_code)] // Reserved for future simulation integration
pub fn resource_market_key(resource: ResourceType, _region: u32) -> &'static str {
    match resource {
        ResourceType::Food => "food",
        ResourceType::Wood => "wood",
        ResourceType::Metal => "metal",
        ResourceType::Energy => "energy",
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn treasury_disparity_whole(treasury: &HashMap<u32, Fixed>, a: u32, b: u32) -> i64 {
    let ta = treasury.get(&a).copied().unwrap_or(Fixed::ZERO);
    let tb = treasury.get(&b).copied().unwrap_or(Fixed::ZERO);
    (ta.to_bits() - tb.to_bits()).abs() / crate::SCALE
}

#[allow(dead_code)] // Reserved for future simulation integration
fn mean_pair_aggression(aggression: &BTreeMap<u32, f32>, a: u32, b: u32) -> f32 {
    let aa = aggression.get(&a).copied().unwrap_or(0.0);
    let ab = aggression.get(&b).copied().unwrap_or(0.0);
    (aa + ab) * 0.5
}

#[allow(dead_code)] // Reserved for future simulation integration
fn shared_religion_cohesion(cultures: &BTreeMap<u64, CultureProfile>, a: u32, b: u32) -> f32 {
    let Some(pa) = cultures.get(&u64::from(a)) else {
        return 0.0;
    };
    let Some(pb) = cultures.get(&u64::from(b)) else {
        return 0.0;
    };
    let similarity = 1.0 - cultural_distance(pa.traits, pb.traits);
    similarity.clamp(0.0, 1.0)
}

#[allow(dead_code)] // Reserved for future simulation integration
fn shared_religious_unity(cultures: &BTreeMap<u64, CultureProfile>, a: u32, b: u32) -> bool {
    shared_religion_cohesion(cultures, a, b) >= 0.7
}

#[allow(dead_code)] // Reserved for future simulation integration
fn resource_competition_signal(ra: &Resources, rb: &Resources) -> f32 {
    let overlaps = [
        (ra.food, rb.food),
        (ra.wood, rb.wood),
        (ra.metal, rb.metal),
        (ra.energy, rb.energy),
    ];
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for (a, b) in overlaps {
        if a.to_bits() > 0 && b.to_bits() > 0 {
            let af = a.to_bits() as f32;
            let bf = b.to_bits() as f32;
            sum += af.min(bf) / af.max(bf);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f32).clamp(0.0, 1.0)
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn need_complementarity_signal(ra: &Resources, rb: &Resources) -> f32 {
    let pairs = [
        (ra.food, rb.food),
        (ra.wood, rb.wood),
        (ra.metal, rb.metal),
        (ra.energy, rb.energy),
    ];
    let mut sum = 0.0f32;
    let mut count = 0u32;
    for (a, b) in pairs {
        if a.to_bits() == 0 && b.to_bits() == 0 {
            continue;
        }
        let af = a.to_bits() as f32;
        let bf = b.to_bits() as f32;
        let gap = (af - bf).abs();
        let scale = af.max(bf).max(1.0);
        sum += (gap / scale).clamp(0.0, 1.0);
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f32).clamp(0.0, 1.0)
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn scarcity_pressure_signal(energy_budget: Fixed, ra: &Resources, rb: &Resources) -> f32 {
    const SCARCITY_GATE: i64 = 100;
    let budget_scarce = i64::from(energy_budget.to_bits()) / crate::SCALE < SCARCITY_GATE;
    let a_scarce = i64::from(ra.energy.to_bits()) / crate::SCALE < SCARCITY_GATE
        && i64::from(ra.food.to_bits()) / crate::SCALE < SCARCITY_GATE;
    let b_scarce = i64::from(rb.energy.to_bits()) / crate::SCALE < SCARCITY_GATE
        && i64::from(rb.food.to_bits()) / crate::SCALE < SCARCITY_GATE;
    if budget_scarce && a_scarce && b_scarce {
        1.0
    } else if (a_scarce && !b_scarce) || (b_scarce && !a_scarce) {
        -0.5
    } else {
        0.0
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn trade_volume_signal(routes: &[TradeRoute], a: u32, b: u32) -> f32 {
    let volume: i64 = routes
        .iter()
        .filter(|route| {
            (route.from_faction == a && route.to_faction == b)
                || (route.from_faction == b && route.to_faction == a)
        })
        .map(|route| i64::from(route.volume.to_bits()))
        .sum();
    ((volume as f64) / 1_000_000.0).clamp(0.0, 1.0) as f32
}

#[allow(dead_code)] // Reserved for future simulation integration
fn proximity_signal(
    a: u32,
    b: u32,
    world: &World,
    member_counts: &BTreeMap<u64, u32>,
    routes: &[TradeRoute],
) -> f32 {
    let dominant = settlement_dominant_factions(world, member_counts);
    let contacts = settlement_contact_pairs(world, member_counts, SETTLEMENT_CONTACT_RADIUS_FP);
    let contact_pairs = diplomacy_faction_pairs_from_settlement_contact(&dominant, &contacts);
    let pair = canonical_faction_pair(a, b);
    if contact_pairs.contains(&pair) {
        return 1.0;
    }
    if routes.iter().any(|route| {
        (route.from_faction == a && route.to_faction == b)
            || (route.from_faction == b && route.to_faction == a)
    }) {
        return 0.7;
    }
    if a.abs_diff(b) == 1 {
        return 0.5;
    }
    0.0
}

#[allow(dead_code)] // Reserved for future simulation integration
#[allow(clippy::too_many_arguments)]
fn diplomacy_signal_for_pair(
    a: u32,
    b: u32,
    state: &WorldState,
    world: &World,
    member_counts: &BTreeMap<u64, u32>,
    cultures: &BTreeMap<u64, CultureProfile>,
    faction_ideologies: &BTreeMap<u32, FactionIdeologyState>,
    grief: &GriefAccumulator,
) -> DiplomacySignal {
    let ra = state.faction_resources.get(&a).cloned().unwrap_or_default();
    let rb = state.faction_resources.get(&b).cloned().unwrap_or_default();
    let religion = shared_religion_cohesion(cultures, a, b);
    let cooperation = culture_cooperation_signal(faction_ideologies, a, b);
    let openness = culture_openness_signal(faction_ideologies, a, b);
    let competition = resource_competition_signal(&ra, &rb) * (1.0 - religion * 0.35);
    DiplomacySignal {
        resource_competition: competition,
        trade_volume: trade_volume_signal(&state.trade_routes, a, b),
        proximity: proximity_signal(a, b, world, member_counts, &state.trade_routes),
        combat_grievance: grief.get(a, b),
        need_complementarity: need_complementarity_signal(&ra, &rb)
            + religion * 0.25
            + cooperation * 0.55,
        scarcity_pressure: scarcity_pressure_signal(state.energy_budget_joules, &ra, &rb)
            - openness * 0.20,
    }
}

/// Deterministic goods label from exporter faction id (stable, integer-only).
#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn emergent_route_goods(from: u32) -> &'static str {
    match from % 3 {
        0 => "grain",
        1 => "ore",
        _ => "cloth",
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
fn record_trade_agreement_streak(streak: &mut BTreeMap<(u32, u32), u32>, a: u32, b: u32) {
    let pair = canonical_faction_pair(a, b);
    *streak.entry(pair).or_default() += 1;
}

#[allow(dead_code)] // Reserved for future simulation integration
fn reset_trade_agreement_streak(streak: &mut BTreeMap<(u32, u32), u32>, a: u32, b: u32) {
    streak.remove(&canonical_faction_pair(a, b));
}

#[allow(dead_code)] // Reserved for future simulation integration
fn remove_emergent_routes_between(state: &mut WorldState, a: u32, b: u32) {
    let to_remove: Vec<(u32, u32, String)> = state
        .emergent_trade_route_keys
        .iter()
        .filter(|(from, to, _)| (*from == a && *to == b) || (*from == b && *to == a))
        .cloned()
        .collect();
    for key in &to_remove {
        state.emergent_trade_route_keys.remove(key);
        state.trade_route_idle_ticks.remove(key);
    }
    state.trade_routes.retain(|route| {
        let key = (route.from_faction, route.to_faction, route.goods.clone());
        !to_remove.contains(&key)
    });
}

#[allow(dead_code)] // Reserved for future simulation integration
fn decay_idle_emergent_trade_routes(state: &mut WorldState, flowed: &BTreeSet<(u32, u32, String)>) {
    let emergent: Vec<(u32, u32, String)> =
        state.emergent_trade_route_keys.iter().cloned().collect();
    let mut to_remove = Vec::new();
    for key in emergent {
        if flowed.contains(&key) {
            state.trade_route_idle_ticks.insert(key.clone(), 0);
            continue;
        }
        let idle = state.trade_route_idle_ticks.entry(key.clone()).or_insert(0);
        *idle = idle.saturating_add(1);
        if *idle >= TRADE_ROUTE_UNUSED_DECAY_TICKS {
            to_remove.push(key);
        }
    }
    for key in &to_remove {
        state.emergent_trade_route_keys.remove(key);
        state.trade_route_idle_ticks.remove(key);
    }
    if !to_remove.is_empty() {
        state.trade_routes.retain(|route| {
            let key = (route.from_faction, route.to_faction, route.goods.clone());
            !to_remove.contains(&key)
        });
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn route_resource(goods: &str) -> ResourceType {
    match goods {
        "grain" => ResourceType::Food,
        "timber" => ResourceType::Wood,
        "ore" | "tools" => ResourceType::Metal,
        "cloth" | "salt" => ResourceType::Energy,
        _ => ResourceType::Food,
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn resource_amount(resources: &Resources, resource: ResourceType) -> Fixed {
    match resource {
        ResourceType::Food => resources.food,
        ResourceType::Wood => resources.wood,
        ResourceType::Metal => resources.metal,
        ResourceType::Energy => resources.energy,
    }
}

#[allow(dead_code)] // Reserved for future simulation integration
pub(crate) fn adjust_resource(resources: &mut Resources, resource: ResourceType, delta: Fixed) {
    match resource {
        ResourceType::Food => resources.food += delta,
        ResourceType::Wood => resources.wood += delta,
        ResourceType::Metal => resources.metal += delta,
        ResourceType::Energy => resources.energy += delta,
    }
}
