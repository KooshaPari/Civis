//! Cluster specialization via comparative advantage (FR-ECON-EMERGE-003).
//!
//! Clusters accumulate affinity for goods they produce relatively efficiently.
//! Affinity scores grow when a cluster's production efficiency for a good
//! exceeds the mean across all goods, implementing emergent comparative advantage.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::stocks::{Good, GOODS};

/// Per-cluster affinity scores for each good.
///
/// Higher affinity signals comparative advantage. Scores are unbounded positive
/// reals; callers may normalise for display purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecializationProfile {
    /// Cluster this profile belongs to.
    pub cluster_id: u32,
    /// Affinity score per good. Missing entries are treated as `0.0`.
    pub affinities: BTreeMap<Good, f32>,
}

impl SpecializationProfile {
    /// Create a new profile with all affinities initialised to zero.
    pub fn new(cluster_id: u32) -> Self {
        let affinities = GOODS.iter().map(|&g| (g, 0.0_f32)).collect();
        Self {
            cluster_id,
            affinities,
        }
    }

    /// Returns the affinity for `good`, defaulting to `0.0`.
    pub fn affinity(&self, good: Good) -> f32 {
        self.affinities.get(&good).copied().unwrap_or(0.0)
    }
}

/// Update specialization affinities from one tick of production efficiency data.
///
/// `production_efficiency` maps each good to the cluster's production efficiency
/// for that good in `[0.0, ∞)`. A value of `1.0` means average efficiency.
///
/// Algorithm: compute the mean efficiency across all present goods; for each
/// good where the cluster's efficiency exceeds the mean, increase its affinity
/// by the excess (`efficiency - mean`). Affinities never decrease via this call
/// (comparative advantage is sticky — clusters specialise over time).
pub fn update_specialization(
    profile: &mut SpecializationProfile,
    production_efficiency: &BTreeMap<Good, f32>,
) {
    if production_efficiency.is_empty() {
        return;
    }

    let mean = {
        let sum: f32 = production_efficiency.values().sum();
        sum / production_efficiency.len() as f32
    };

    for (&good, &efficiency) in production_efficiency {
        let excess = efficiency - mean;
        if excess > 0.0 {
            *profile.affinities.entry(good).or_insert(0.0) += excess;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn efficiency_map(pairs: &[(Good, f32)]) -> BTreeMap<Good, f32> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn affinity_increases_for_above_average_goods() {
        let mut profile = SpecializationProfile::new(0);
        // Food efficiency 3.0, Water 1.0 → mean 2.0 → food excess 1.0
        let eff = efficiency_map(&[(Good::Food, 3.0), (Good::Water, 1.0)]);
        update_specialization(&mut profile, &eff);
        assert!(
            profile.affinity(Good::Food) > 0.0,
            "food affinity must increase when above average"
        );
        assert_eq!(
            profile.affinity(Good::Water),
            0.0,
            "water affinity must not increase when below average"
        );
    }

    #[test]
    fn affinity_accumulates_across_ticks() {
        let mut profile = SpecializationProfile::new(0);
        let eff = efficiency_map(&[(Good::Wood, 5.0), (Good::Metal, 1.0)]);
        update_specialization(&mut profile, &eff);
        let after_first = profile.affinity(Good::Wood);
        update_specialization(&mut profile, &eff);
        assert!(
            profile.affinity(Good::Wood) > after_first,
            "affinity must accumulate across ticks"
        );
    }

    #[test]
    fn all_equal_efficiency_produces_no_change() {
        let mut profile = SpecializationProfile::new(0);
        let eff = efficiency_map(&[
            (Good::Food, 2.0),
            (Good::Water, 2.0),
            (Good::Wood, 2.0),
        ]);
        update_specialization(&mut profile, &eff);
        for good in GOODS {
            assert_eq!(
                profile.affinity(good),
                0.0,
                "uniform efficiency should not change any affinity"
            );
        }
    }

    #[test]
    fn empty_efficiency_map_is_a_no_op() {
        let mut profile = SpecializationProfile::new(0);
        profile.affinities.insert(Good::Food, 3.0);
        update_specialization(&mut profile, &BTreeMap::new());
        assert_eq!(profile.affinity(Good::Food), 3.0);
    }
}
