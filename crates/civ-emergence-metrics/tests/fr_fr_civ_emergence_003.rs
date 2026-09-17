//! Tests for FR-CIV-EMERGENCE-003 — dashboard summary metrics.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_003 {
    use civ_emergence_metrics::dashboard::{
        cluster_entropy, ideology_homophily, sentience_fraction,
        psyche_stability, diplomacy_tension, EmergenceDashboard,
    };

    #[test]
    fn verify_fr_civ_emergence_003_basic() {
        // Uniform population → high entropy
        let pop = [25.0, 25.0, 25.0, 25.0];
        let e = cluster_entropy(&pop);
        assert!(e > 0.9, "uniform should have high entropy, got {e}");
    }

    #[test]
    fn cluster_entropy_single_cluster_is_zero() {
        let pop = [100.0];
        let e = cluster_entropy(&pop);
        assert_eq!(e, 0.0);
    }

    #[test]
    fn sentience_fraction_basic() {
        let f = sentience_fraction(3, 10);
        assert!((f - 0.3).abs() < 0.01);
    }

    #[test]
    fn sentience_fraction_zero_total() {
        let f = sentience_fraction(0, 0);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn dashboard_compute_default_is_zero() {
        let d = EmergenceDashboard::default();
        assert_eq!(d.cluster_entropy, 0.0);
        assert_eq!(d.ideology_homophily, 0.0);
    }
}
