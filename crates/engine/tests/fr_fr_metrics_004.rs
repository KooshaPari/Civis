//! Tests for FR-METRICS-004 — Metrics (deterministic metrics)
//!
//! Epic: FR-METRICS
//! Verifies MetricsFixed produces deterministic, cross-platform results.

#[cfg(test)]
mod fr_fr_metrics_004 {
    /// FR-METRICS-004: MetricsFixed derives Default.
    #[test]
    fn metrics_fixed_default() {
        let m = civ_engine::MetricsFixed::default();
        assert_eq!(m.waste_joules, civ_engine::Fixed::ZERO);
        assert_eq!(m.surplus_joules, civ_engine::Fixed::ZERO);
        assert_eq!(m.tyranny_index, civ_engine::Fixed::ZERO);
        assert_eq!(m.legitimacy_index, civ_engine::Fixed::ZERO);
    }

    /// FR-METRICS-004: compute_fixed is deterministic.
    #[test]
    fn compute_fixed_is_deterministic() {
        use civ_engine::Fixed;
        let a = civ_engine::metrics::compute_fixed(Fixed::from_num(500), Fixed::from_num(200));
        let b = civ_engine::metrics::compute_fixed(Fixed::from_num(500), Fixed::from_num(200));
        assert_eq!(a.waste_joules, b.waste_joules);
        assert_eq!(a.surplus_joules, b.surplus_joules);
        assert_eq!(a.tyranny_index, b.tyranny_index);
        assert_eq!(a.legitimacy_index, b.legitimacy_index);
    }

    /// FR-METRICS-004: Negative inputs are clamped to zero.
    #[test]
    fn compute_fixed_clamps_negative() {
        use civ_engine::Fixed;
        let m = civ_engine::metrics::compute_fixed(Fixed::from_num(-10), Fixed::from_num(-5));
        assert_eq!(m.waste_joules, Fixed::ZERO);
        assert_eq!(m.surplus_joules, Fixed::ZERO);
        assert_eq!(m.tyranny_index, Fixed::ZERO);
        assert_eq!(m.legitimacy_index, Fixed::ONE);
    }
}
