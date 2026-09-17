//! Tests for FR-CIV-EMERGENCE-005 — power law fit.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_005 {
    use civ_emergence_metrics::power_law::PowerLawFit;
    use civ_emergence_metrics::Histogram;

    #[test]
    fn verify_fr_civ_emergence_005_basic() {
        // Zipf-like distribution: 100, 50, 33, 25, 20, ...
        let counts: Vec<u64> = (1..=10).map(|r| (100 / r) as u64).collect();
        let h = Histogram::from_counts(counts);
        let fit = PowerLawFit::new();
        let result = fit.compute_rank_frequency(&h);
        // Zipf has alpha ~1.0, R² > 0.8
        assert!(
            result.alpha > 0.5,
            "Zipf should have positive alpha, got {}",
            result.alpha
        );
        assert!(
            result.r_squared > 0.5,
            "Zipf should have decent R², got {}",
            result.r_squared
        );
    }

    #[test]
    fn uniform_distribution_low_alpha() {
        let h = Histogram::from_counts(vec![10; 10]);
        let fit = PowerLawFit::new();
        let result = fit.compute_rank_frequency(&h);
        // Uniform has alpha near 0
        assert!(
            result.alpha.abs() < 1.0,
            "uniform should have low alpha, got {}",
            result.alpha
        );
    }

    #[test]
    fn too_few_points_returns_zero() {
        let h = Histogram::from_counts(vec![5]);
        let fit = PowerLawFit::new();
        let result = fit.compute_rank_frequency(&h);
        assert_eq!(result.alpha, 0.0);
        assert_eq!(result.r_squared, 0.0);
    }
}
