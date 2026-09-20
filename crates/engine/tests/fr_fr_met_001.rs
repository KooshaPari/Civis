//! Tests for FR-MET-001 — Metrics (energy metrics)
//!
//! Epic: FR-MET
//! Verifies the simulation metrics computation: waste, surplus, tyranny, legitimacy.

#[cfg(test)]
mod fr_fr_met_001 {
    /// FR-MET-001: Metrics computes waste as 10% of consumption.
    #[test]
    fn waste_is_ten_percent_of_consumption() {
        let m = civ_engine::metrics::compute(1000.0, 500.0);
        assert_eq!(m.waste_joules, 50.0);
    }

    /// FR-MET-001: Surplus is budget minus consumption (clamped to 0).
    #[test]
    fn surplus_is_budget_minus_consumption() {
        let m = civ_engine::metrics::compute(1000.0, 400.0);
        assert_eq!(m.surplus_joules, 600.0);
    }

    /// FR-MET-001: Over-budget consumption yields zero surplus.
    #[test]
    fn over_budget_yields_zero_surplus() {
        let m = civ_engine::metrics::compute(100.0, 150.0);
        assert_eq!(m.surplus_joules, 0.0);
        assert_eq!(m.tyranny_index, 1.0);
    }

    /// FR-MET-001: Fixed-point metrics match float metrics.
    #[test]
    fn fixed_point_matches_float() {
        use civ_engine::Fixed;
        let fm = civ_engine::metrics::compute(1000.0, 500.0);
        let fpm = civ_engine::metrics::compute_fixed(Fixed::from_num(1000), Fixed::from_num(500));
        assert!((fm.waste_joules - fpm.waste_joules.to_num::<f64>()).abs() < 0.01);
        assert!((fm.surplus_joules - fpm.surplus_joules.to_num::<f64>()).abs() < 0.01);
    }

    /// FR-MET-001: Non-finite inputs are sanitized.
    #[test]
    fn non_finite_sanitized() {
        let m = civ_engine::metrics::compute(f64::INFINITY, f64::NAN);
        assert_eq!(m.waste_joules, 0.0);
        assert_eq!(m.legitimacy_index, 1.0);
    }
}
