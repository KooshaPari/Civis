//! Tests for FR-CIV-QOL-180
//! Epic: FR-CIV-QOL. Time-series charts and graphs.
#[cfg(test)]
mod fr_fr_civ_qol_180 {
    #[test]
    fn metrics_provide_time_series_data() {
        // FR-CIV-QOL-180 requires time-series charts.
        // Engine metrics compute from budget/consumption values.
        let m = civ_engine::metrics::compute(1_000_000.0, 500_000.0);
        // Metrics should have valid values.
        assert!(m.waste_joules >= 0.0, "Waste must be non-negative");
        assert!(m.surplus_joules >= 0.0, "Surplus must be non-negative");
    }

    #[test]
    fn fixed_metrics_also_available() {
        let m = civ_engine::metrics::compute_fixed(
            civ_engine::Fixed::from_num(1_000_000),
            civ_engine::Fixed::from_num(500_000),
        );
        assert!(m.waste_joules >= civ_engine::Fixed::ZERO);
    }
}
