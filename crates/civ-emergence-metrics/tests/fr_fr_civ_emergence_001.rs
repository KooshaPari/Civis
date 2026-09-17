//! Tests for FR-CIV-EMERGENCE-001 — branching ratio metric.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_001 {
    use civ_emergence_metrics::branching::{
        BranchingRegime, AvalancheRecord, rolling_mean_sigma, classify_regime,
        DEFAULT_BRANCHING_WINDOW, SIGMA_SUBCRITICAL, SIGMA_EDGE_LOW,
    };

    #[test]
    fn verify_fr_civ_emergence_001_basic() {
        // Single avalanche with 10 descendants / 10 actors = sigma 1.0
        let records = vec![AvalancheRecord { descendants: 10, actors: 10 }];
        let sigma = rolling_mean_sigma(&records, DEFAULT_BRANCHING_WINDOW);
        assert!((sigma - 1.0).abs() < 0.01, "sigma should be ~1.0 for equal descendants/actors");
    }

    #[test]
    fn rolling_mean_uses_window() {
        let records = vec![
            AvalancheRecord { descendants: 0, actors: 10 }, // sigma 0.0
            AvalancheRecord { descendants: 0, actors: 10 }, // sigma 0.0
            AvalancheRecord { descendants: 10, actors: 10 }, // sigma 1.0
        ];
        let sigma = rolling_mean_sigma(&records, 2);
        // Window of 2: last 2 records have sigma 0.0 and 1.0 → mean 0.5
        assert!((sigma - 0.5).abs() < 0.01, "rolling mean of last 2 should be 0.5, got {sigma}");
    }

    #[test]
    fn zero_actors_returns_zero() {
        let records = vec![AvalancheRecord { descendants: 5, actors: 0 }];
        let sigma = rolling_mean_sigma(&records, DEFAULT_BRANCHING_WINDOW);
        assert_eq!(sigma, 0.0, "zero actors should yield zero sigma");
    }

    #[test]
    fn classify_heat_death_below_threshold() {
        let regime = classify_regime(SIGMA_SUBCRITICAL - 0.1);
        assert_eq!(regime, BranchingRegime::HeatDeath);
    }

    #[test]
    fn classify_edge_of_chaos() {
        let mid = (SIGMA_EDGE_LOW + 0.99) / 2.0;
        let regime = classify_regime(mid);
        assert_eq!(regime, BranchingRegime::EdgeOfChaos);
    }

    #[test]
    fn classify_supercritical() {
        let regime = classify_regime(1.5);
        assert_eq!(regime, BranchingRegime::Supercritical);
    }
}
