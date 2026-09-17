//! Tests for FR-CIV-WAR-030
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-030: Doctrine-GA evolution across the loop.
//! Tests that doctrine fitness scoring rewards higher engagement stats.

use civ_tactics::{FactionEngagementStats, score_doctrine_fitness, Doctrine};

#[cfg(test)]
mod fr_fr_civ_war_030 {
    use super::*;

    /// FR-CIV-WAR-030: Higher engagement stats yield higher doctrine fitness.
    #[test]
    fn verify_fr_civ_war_030_basic() {
        let doctrine = Doctrine {
            id: 42,
            unit_composition: vec![5, 3, 2],
            score: 0.0,
        };
        let low = FactionEngagementStats {
            engagements_as_shooter: 1,
            engagements_as_target: 0,
            voxels_removed: 10,
        };
        let high = FactionEngagementStats {
            engagements_as_shooter: 50,
            engagements_as_target: 30,
            voxels_removed: 2000,
        };
        let s_low = score_doctrine_fitness(&doctrine, &low);
        let s_high = score_doctrine_fitness(&doctrine, &high);
        assert!(s_high > s_low, "more engaged faction should score higher fitness");
    }

    /// FR-CIV-WAR-030: Doctrine fitness is deterministic for same inputs.
    #[test]
    fn doctrine_fitness_deterministic() {
        let doctrine = Doctrine {
            id: 7,
            unit_composition: vec![10, 10],
            score: 0.0,
        };
        let stats = FactionEngagementStats {
            engagements_as_shooter: 10,
            engagements_as_target: 5,
            voxels_removed: 100,
        };
        let s1 = score_doctrine_fitness(&doctrine, &stats);
        let s2 = score_doctrine_fitness(&doctrine, &stats);
        assert_eq!(s1, s2, "fitness must be deterministic");
    }
}
