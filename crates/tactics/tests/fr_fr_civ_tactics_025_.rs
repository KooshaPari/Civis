//! Tests for FR-CIV-TACTICS-025-
//!
//! Epic: FR-CIV-TACTICS
//! Status: IMPL-NO-TEST
//!
//! FR-CIV-TACTICS-025-: Tactics and strategy — Doctrine fitness scoring.

use civ_tactics::{FactionEngagementStats, score_doctrine_fitness, Doctrine};

#[cfg(test)]
mod fr_fr_civ_tactics_025_ {
    use super::*;

    /// FR-CIV-TACTICS-025-: Doctrine fitness scores higher for more engagements.
    #[test]
    fn verify_fr_civ_tactics_025__basic() {
        let doctrine = Doctrine {
            id: 1,
            unit_composition: vec![3, 2, 1],
            score: 0.0,
        };
        let low_stats = FactionEngagementStats {
            engagements_as_shooter: 2,
            engagements_as_target: 1,
            voxels_removed: 50,
        };
        let high_stats = FactionEngagementStats {
            engagements_as_shooter: 20,
            engagements_as_target: 15,
            voxels_removed: 500,
        };
        let low_score = score_doctrine_fitness(&doctrine, &low_stats);
        let high_score = score_doctrine_fitness(&doctrine, &high_stats);
        assert!(
            high_score > low_score,
            "higher engagement stats should yield higher fitness"
        );
    }
}
