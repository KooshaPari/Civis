//! Tests for FR-ECON-006 — GDP Derived from Regional Joule Throughput
//!
//! Epic: FR-ECON
//! GDP SHALL be the sum of regional Joule throughput converted at a fixed
//! exchange rate. Each region contributes production minus waste. The
//! aggregate is deterministic and integer-only.

use civ_economy::{compute_gdp, RegionGdp};

#[cfg(test)]
mod fr_fr_econ_006 {
    use super::*;

    /// FR-ECON-006: Single region GDP equals net throughput (production - waste).
    #[test]
    fn single_region_gdp_equals_net_throughput() {
        let region = RegionGdp {
            region_id: 0,
            production_joules: 1000,
            waste_joules: 100,
        };
        assert_eq!(region.net_throughput(), 900);
        assert_eq!(region.gdp_contribution(), 900, "1.0 exchange rate means contribution = net");
    }

    /// FR-ECON-006: Aggregate GDP sums contributions from all regions.
    #[test]
    fn aggregate_gdp_sums_all_regions() {
        let regions = vec![
            RegionGdp { region_id: 0, production_joules: 2000, waste_joules: 200 },
            RegionGdp { region_id: 1, production_joules: 1000, waste_joules: 50 },
            RegionGdp { region_id: 2, production_joules: 500, waste_joules: 0 },
        ];
        let result = compute_gdp(&regions);
        // Region 0: 1800, Region 1: 950, Region 2: 500 => total 3250
        assert_eq!(result.total_gdp, 3250);
        assert_eq!(result.region_count, 3);
    }

    /// FR-ECON-006: Empty region list produces zero GDP.
    #[test]
    fn empty_regions_zero_gdp() {
        let result = compute_gdp(&[]);
        assert_eq!(result.total_gdp, 0);
        assert_eq!(result.region_count, 0);
        assert!(result.regions.is_empty());
    }

    /// FR-ECON-006: Net throughput clamps at zero when waste exceeds production.
    #[test]
    fn waste_exceeding_production_clamps_to_zero() {
        let region = RegionGdp {
            region_id: 0,
            production_joules: 100,
            waste_joules: 500,
        };
        assert_eq!(region.net_throughput(), 0, "net throughput clamped at 0");
        assert_eq!(region.gdp_contribution(), 0);
    }

    /// FR-ECON-006: Zero production and zero waste yields zero GDP contribution.
    #[test]
    fn zero_production_zero_waste() {
        let region = RegionGdp {
            region_id: 0,
            production_joules: 0,
            waste_joules: 0,
        };
        assert_eq!(region.net_throughput(), 0);
        assert_eq!(region.gdp_contribution(), 0);
    }

    /// FR-ECON-006: GDP result preserves per-region data.
    #[test]
    fn gdp_result_preserves_region_data() {
        let regions = vec![
            RegionGdp { region_id: 42, production_joules: 500, waste_joules: 50 },
        ];
        let result = compute_gdp(&regions);
        assert_eq!(result.regions.len(), 1);
        assert_eq!(result.regions[0].region_id, 42);
        assert_eq!(result.regions[0].production_joules, 500);
    }
}
