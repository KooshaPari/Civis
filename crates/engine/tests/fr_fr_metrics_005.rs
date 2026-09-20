//! Tests for FR-METRICS-005 — Metrics (tyranny and legitimacy indices)
//!
//! Epic: FR-METRICS
//! Verifies tyranny and legitimacy index calculations are complementary.

#[cfg(test)]
mod fr_fr_metrics_005 {
    /// FR-METRICS-005: Tyranny + legitimacy approximates 1.0.
    #[test]
    fn tyranny_plus_legitimacy_approx_one() {
        let m = civ_engine::metrics::compute(1000.0, 500.0);
        let sum = m.tyranny_index + m.legitimacy_index;
        assert!((sum - 1.0).abs() < 0.01);
    }

    /// FR-METRICS-005: When consumption equals budget, tyranny is high.
    #[test]
    fn equal_budget_consumption_high_tyranny() {
        let m = civ_engine::metrics::compute(100.0, 100.0);
        assert!(m.tyranny_index > 0.9);
        assert!(m.legitimacy_index < 0.1);
    }

    /// FR-METRICS-005: Low consumption yields low tyranny.
    #[test]
    fn low_consumption_low_tyranny() {
        let m = civ_engine::metrics::compute(10000.0, 10.0);
        assert!(m.tyranny_index < 0.01);
        assert!(m.legitimacy_index > 0.99);
    }

    /// FR-METRICS-005: Fixed-point tyranny matches float tyranny.
    #[test]
    fn fixed_point_tyranny_matches_float() {
        use civ_engine::Fixed;
        let fm = civ_engine::metrics::compute(100.0, 100.0);
        let fpm = civ_engine::metrics::compute_fixed(Fixed::from_num(100), Fixed::from_num(100));
        assert!((fm.tyranny_index - fpm.tyranny_index.to_num::<f64>()).abs() < 0.01);
        assert!((fm.legitimacy_index - fpm.legitimacy_index.to_num::<f64>()).abs() < 0.01);
    }
}
