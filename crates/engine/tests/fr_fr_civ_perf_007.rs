//! Tests for FR-CIV-PERF-007
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-007: Fixed-point metrics computation.

#[cfg(test)]
mod fr_fr_civ_perf_007 {
    /// Verify FR-CIV-PERF-007: MetricsFixed matches float compute within tolerance.
    #[test]
    fn verify_fr_civ_perf_007_basic() {
        use civ_engine::Fixed;
        let budget = Fixed::from_num(1000i64);
        let consumption = Fixed::from_num(500i64);
        let m = civ_engine::metrics::compute_fixed(budget, consumption);
        // waste = 500/10 = 50
        assert_eq!(m.waste_joules, Fixed::from_num(50i64));
        // surplus = 1000 - 500 = 500
        assert_eq!(m.surplus_joules, Fixed::from_num(500i64));
    }

    /// Verify float metrics compute correctly.
    #[test]
    fn float_metrics_basic() {
        let m = civ_engine::metrics::compute(1000.0, 500.0);
        assert_eq!(m.waste_joules, 50.0);
        assert_eq!(m.surplus_joules, 500.0);
        assert!(m.tyranny_index > 0.0 && m.tyranny_index <= 1.0);
        assert!(m.legitimacy_index > 0.0 && m.legitimacy_index <= 1.0);
    }

    /// Verify metrics handles over-budget consumption.
    #[test]
    fn over_budget_clamps() {
        let m = civ_engine::metrics::compute(100.0, 150.0);
        assert_eq!(m.surplus_joules, 0.0);
        assert_eq!(m.tyranny_index, 1.0);
        assert_eq!(m.legitimacy_index, 0.0);
    }
}
