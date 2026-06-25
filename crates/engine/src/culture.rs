//! Emergent faction-level culture and ideology behavior coupling for FR-CULTURE.
//!
//! This module tracks a lightweight per-faction culture vector used by diplomacy
//! and related downstream systems. Values are derived from cluster culture vectors
//! plus environment/historical/religion signals and updated every emergence tick.

use std::collections::{BTreeMap, BTreeSet};

use rand::Rng;
use civ_agents::culture::{cultural_distance, CultureProfile};
use civ_planet::Climate;
use crate::era::CivAge;

const DIM: usize = 4;
const MAX_DRIFT_RATE: f32 = 0.09;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FactionIdeologyState {
    /// Core values vector (values). Divergence pressure is strongest here.
    pub values: [f32; DIM],
    /// Social norms / institutions vector (norms). More stable than values.
    pub norms: [f32; DIM],
    /// Cooperative signal in `[0, 1]`.
    pub cooperation: f32,
    /// Aggression signal in `[0, 1]`.
    pub aggression: f32,
    /// Openness (exchange + tolerance) signal in `[0, 1]`.
    pub openness: f32,
    /// Tradition retention / historical continuity in `[0, 1]`.
    pub tradition: f32,
}

impl Default for FactionIdeologyState {
    fn default() -> Self {
        Self {
            values: [0.5; DIM],
            norms: [0.5; DIM],
            cooperation: 0.5,
            aggression: 0.0,
            openness: 0.5,
            tradition: 0.5,
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn era_weight(age: &CivAge) -> f32 {
    match age {
        CivAge::Stone => 0.0,
        CivAge::Bronze => 0.15,
        CivAge::Iron => 0.30,
        CivAge::Classical => 0.45,
        CivAge::Medieval => 0.65,
        CivAge::Industrial => 0.85,
    }
}

fn cluster_values_for_faction(
    cluster_cultures: &BTreeMap<u64, CultureProfile>,
    dominant: &BTreeMap<u64, u32>,
    member_counts: &BTreeMap<u64, u32>,
) -> BTreeMap<u32, ([f32; DIM], [f32; DIM]) {
    let mut sums: BTreeMap<u32, ([f32; DIM], [f32; DIM], f32)> = BTreeMap::new();
    for (cluster_id, faction_id) in dominant {
        let members = member_counts.get(cluster_id).copied().unwrap_or(0);
        if members < 2 {
            continue;
        }
        let Some(profile) = cluster_cultures.get(cluster_id) else {
            continue;
        };
        let e = sums.entry(*faction_id).or_insert(([0.0; DIM], [0.0; DIM], 0.0));
        let weight = members as f32;
        for i in 0..DIM {
            e.0[i] += profile.traits[i] * weight;
            e.1[i] += profile.language[i] * weight;
        }
        e.2 += weight;
    }

    let mut out = BTreeMap::new();
    for (faction_id, (sum_values, sum_norms, weight)) in sums {
        if weight <= 0.0 {
            continue;
        }
        let mut values = [0.0f32; DIM];
        let mut norms = [0.0f32; DIM];
        for i in 0..DIM {
            values[i] = sum_values[i] / weight;
            norms[i] = sum_norms[i] / weight;
        }
        out.insert(faction_id, (values, norms));
    }
    out
}

fn faction_isolation_pressure(
    target_faction_id: u32,
    dominant: &BTreeMap<u64, u32>,
    cluster_member_counts: &BTreeMap<u64, u32>,
    settlement_contacts: &BTreeSet<(u64, u64)>,
) -> f32 {
    let mut target_members = 0u32;
    let mut contacting_members = 0.0f32;

    let mut target_settlements = BTreeSet::new();
    for (&settlement_id, &faction_id) in dominant {
        if faction_id == target_faction_id {
            target_settlements.insert(settlement_id);
            target_members = target_members.saturating_add(
                cluster_member_counts.get(&settlement_id).copied().unwrap_or(0),
            );
        }
    }

    for &(left, right) in settlement_contacts {
        let Some(&fa) = dominant.get(&left) else {
            continue;
        };
        let Some(&fb) = dominant.get(&right) else {
            continue;
        };
        if fa == fb {
            continue;
        }

        if fa == target_faction_id && target_settlements.contains(&left) {
            let local_members = cluster_member_counts.get(&left).copied().unwrap_or(0) as f32;
            let foreign_members = cluster_member_counts.get(&right).copied().unwrap_or(0) as f32;
            if foreign_members > 0.0 {
                contacting_members += local_members / (foreign_members + 1.0);
            }
        }
        if fb == target_faction_id && target_settlements.contains(&right) {
            let local_members = cluster_member_counts.get(&right).copied().unwrap_or(0) as f32;
            let foreign_members = cluster_member_counts.get(&left).copied().unwrap_or(0) as f32;
            if foreign_members > 0.0 {
                contacting_members += local_members / (foreign_members + 1.0);
            }
        }
    }

    if target_members == 0 {
        return 0.0;
    }
    let ratio = (contacting_members / target_members as f32).clamp(0.0, 1.0);
    (1.0 - ratio).clamp(0.0, 1.0)
}

/// Advance per-faction ideology from cluster culture, environment, history,
/// religion and era state.
pub(crate) fn advance_faction_ideologies(
    tick: u64,
    cluster_cultures: &BTreeMap<u64, CultureProfile>,
    dominant: &BTreeMap<u64, u32>,
    cluster_member_counts: &BTreeMap<u64, u32>,
    settlement_contacts: &BTreeSet<(u64, u64)>,
    climate: &Climate,
    religion_by_faction: &BTreeMap<u32, f32>,
    faction_ages: &BTreeMap<u32, CivAge>,
    prior: &BTreeMap<u32, FactionIdeologyState>,
    rng: &mut impl Rng,
) -> BTreeMap<u32, FactionIdeologyState> {
    let base_profiles = cluster_values_for_faction(cluster_cultures, dominant, cluster_member_counts);

    let mut next = BTreeMap::new();
    for (faction_id, (base_values, base_norms)) in base_profiles {
        let prior_state = prior
            .get(&faction_id)
            .copied()
            .unwrap_or_else(FactionIdeologyState::default);

        let isolation = faction_isolation_pressure(
            faction_id,
            dominant,
            cluster_member_counts,
            settlement_contacts,
        );

        let history_age = ((tick as f32) / 600.0).fract();
        let climate_push = climate.day_phase * 0.10 + climate.moon_phase * 0.02 + climate.tide_offset.abs() * 0.04;
        let religion = clamp01(*religion_by_faction.get(&faction_id).unwrap_or(&0.5));
        let era = faction_ages
            .get(&faction_id)
            .map(era_weight)
            .unwrap_or(0.0);

        let mut values = [0.0f32; DIM];
        let mut norms = [0.0f32; DIM];
        for i in 0..DIM {
            let drift_strength = (0.02 + isolation * 0.035 + history_age * 0.01).min(MAX_DRIFT_RATE);
            let noise = (rng.gen_range(-0.5f32..0.5f32) * 2.0 * drift_strength);
            let toward_base = (base_values[i] - prior_state.values[i]) * (0.30 + climate_push);
            let tradition_pull = prior_state.tradition * 0.35;
            let religion_pull = (religion - 0.5) * 0.04;
            values[i] = clamp01(
                prior_state.values[i]
                    + toward_base * 0.4
                    + noise
                    + tradition_pull * 0.08
                    + religion_pull * (1.0 + era)
                    + history_age * 0.01,
            );

            let norm_noise = (rng.gen_range(-0.5f32..0.5f32) * 0.015 * (1.0 - 0.65 * isolation));
            let toward_norm = (base_norms[i] - prior_state.norms[i]) * 0.22;
            norms[i] = clamp01(prior_state.norms[i] + toward_norm * (0.5 + climate_push) + norm_noise);
        }

        let values_coherence = 1.0 - cultural_distance(values, norms);
        let openness = clamp01(
            0.15 + (1.0 - isolation) * 0.70 + values_coherence * 0.08 + era * 0.15 + religion * 0.12,
        );
        let cooperation = clamp01(
            0.10 + openness * 0.55 + values_coherence * 0.35 + (1.0 - isolation) * 0.20,
        );
        let aggression = clamp01(
            0.10 + isolation * 0.70 + (1.0 - religion) * 0.20 + (1.0 - openness) * 0.15 + (1.0 - era) * 0.10,
        );
        let tradition = clamp01(prior_state.tradition * 0.80 + era * 0.1 + (1.0 - isolation) * 0.05 + 0.05);

        next.insert(
            faction_id,
            FactionIdeologyState {
                values,
                norms,
                cooperation,
                aggression,
                openness,
                tradition,
            },
        );
    }

    next
}

fn cooperation_signal_from_state(a: &FactionIdeologyState, b: &FactionIdeologyState) -> f32 {
    let norm_distance = cultural_distance(a.norms, b.norms);
    let value_distance = cultural_distance(a.values, b.values);
    let bounded = ((1.0 - norm_distance) * 0.6 + (1.0 - value_distance) * 0.4).clamp(0.0, 1.0);
    ((a.cooperation + b.cooperation) * 0.5 * bounded).clamp(0.0, 1.0)
}

fn openness_signal_from_state(a: &FactionIdeologyState, b: &FactionIdeologyState) -> f32 {
    ((a.openness + b.openness) * 0.5).clamp(0.0, 1.0)
}

pub(crate) fn culture_cooperation_signal(
    ideologies: &BTreeMap<u32, FactionIdeologyState>,
    faction_a: u32,
    faction_b: u32,
) -> f32 {
    let Some(a) = ideologies.get(&faction_a) else {
        return 0.0;
    };
    let Some(b) = ideologies.get(&faction_b) else {
        return 0.0;
    };
    cooperation_signal_from_state(a, b)
}

pub(crate) fn culture_openness_signal(
    ideologies: &BTreeMap<u32, FactionIdeologyState>,
    faction_a: u32,
    faction_b: u32,
) -> f32 {
    let Some(a) = ideologies.get(&faction_a) else {
        return 0.0;
    };
    let Some(b) = ideologies.get(&faction_b) else {
        return 0.0;
    };
    openness_signal_from_state(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_agents::culture::CultureProfile;
    use civ_planet::{Climate, compute_climate};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn build_environment(tick: u64) -> Climate {
        let (planet, moon) = civ_planet::defaults_earthlike();
        compute_climate(tick, &planet, &moon)
    }

    #[test]
    fn two_isolated_factions_diverge_values_over_time() {
        let mut profiles = BTreeMap::from([
            (1_u64, CultureProfile::new([0.15, 0.14, 0.13, 0.12])),
            (2_u64, CultureProfile::new([0.85, 0.84, 0.83, 0.82])),
        ]);
        let dominant = BTreeMap::from([(1_u64, 0_u32), (2_u64, 1_u32)]);
        let members = BTreeMap::from([(1_u64, 8_u32), (2_u64, 8_u32)]);
        let contacts = BTreeSet::new();
        let religion = BTreeMap::from([(0_u32, 0.35_f32), (1_u32, 0.45_f32)]);
        let mut era = BTreeMap::new();
        era.insert(0_u32, CivAge::Stone);
        era.insert(1_u32, CivAge::Stone);
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let mut prior = BTreeMap::new();

        let first = advance_faction_ideologies(
            0,
            &profiles,
            &dominant,
            &members,
            &contacts,
            &build_environment(0),
            &religion,
            &era,
            &prior,
            &mut rng,
        );

        let snapshot_a0 = first.get(&0).expect("faction 0 should exist").values;
        let snapshot_b0 = first.get(&1).expect("faction 1 should exist").values;

        for tick in 1..48 {
            prior = advance_faction_ideologies(
                tick,
                &profiles,
                &dominant,
                &members,
                &contacts,
                &build_environment(tick),
                &religion,
                &era,
                &prior,
                &mut rng,
            );
        }

        let snapshot_a = prior.get(&0).expect("faction 0 should exist").values;
        let snapshot_b = prior.get(&1).expect("faction 1 should exist").values;

        assert_ne!(snapshot_a0, snapshot_a);
        assert_ne!(snapshot_b0, snapshot_b);
        assert!(cultural_distance(snapshot_a, snapshot_b) > 0.08);

        let cooperation = culture_cooperation_signal(&prior, 0, 1);
        assert!(cooperation >= 0.0);
        let openness = culture_openness_signal(&prior, 0, 1);
        assert!(openness <= 1.0);
    }
}
