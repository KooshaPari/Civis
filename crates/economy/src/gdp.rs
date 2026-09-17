//! FR-ECON-006 — GDP derived from regional Joule throughput.
//!
//! GDP is the sum of regional Joule throughput converted at a fixed exchange
//! rate. Each region contributes its production minus waste. The aggregate
//! is deterministic and integer-only.
//!
//! All math is integer-saturating. No floats accumulate across calls.

use serde::{Deserialize, Serialize};

/// Fixed exchange rate: 1 Joule = 1 GDP unit (basis points).
/// Callers can scale by a scenario-configurable multiplier.
const GDP_EXCHANGE_BP: i64 = 10_000; // 1.00 in basis points

/// Basis-point denominator.
const BP_DENOM: i64 = 10_000;

/// Per-region GDP contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionGdp {
    /// Region identifier.
    pub region_id: u32,
    /// Gross Joule production this tick.
    pub production_joules: i64,
    /// Waste Joules this tick.
    pub waste_joules: i64,
}

impl RegionGdp {
    /// Net Joule throughput (production - waste).
    #[must_use]
    pub fn net_throughput(&self) -> i64 {
        (self.production_joules - self.waste_joules).max(0)
    }

    /// GDP contribution = net_throughput * exchange_rate / 10_000.
    #[must_use]
    pub fn gdp_contribution(&self) -> i64 {
        self.net_throughput() * GDP_EXCHANGE_BP / BP_DENOM
    }
}

/// Aggregate GDP result for a tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GdpResult {
    /// Total GDP across all regions.
    pub total_gdp: i64,
    /// Per-region contributions.
    pub regions: Vec<RegionGdp>,
    /// Number of regions.
    pub region_count: usize,
}

/// Compute aggregate GDP from regional Joule throughput.
///
/// Sums `net_throughput * exchange_rate / 10_000` across all regions.
/// Returns zero GDP for empty input.
#[must_use]
pub fn compute_gdp(regions: &[RegionGdp]) -> GdpResult {
    let total: i64 = regions.iter().map(|r| r.gdp_contribution()).sum();
    GdpResult {
        total_gdp: total,
        regions: regions.to_vec(),
        region_count: regions.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_region_gdp() {
        let r = RegionGdp {
            region_id: 0,
            production_joules: 1000,
            waste_joules: 100,
        };
        assert_eq!(r.net_throughput(), 900);
        assert_eq!(r.gdp_contribution(), 900); // 900 * 1.0
    }

    #[test]
    fn aggregate_gdp_sums_regions() {
        let regions = vec![
            RegionGdp { region_id: 0, production_joules: 1000, waste_joules: 100 },
            RegionGdp { region_id: 1, production_joules: 500, waste_joules: 50 },
        ];
        let result = compute_gdp(&regions);
        assert_eq!(result.total_gdp, 900 + 450);
        assert_eq!(result.region_count, 2);
    }

    #[test]
    fn empty_regions_zero_gdp() {
        let result = compute_gdp(&[]);
        assert_eq!(result.total_gdp, 0);
        assert_eq!(result.region_count, 0);
    }

    #[test]
    fn waste_exceeds_production_clamps_net_to_zero() {
        let r = RegionGdp {
            region_id: 0,
            production_joules: 100,
            waste_joules: 200,
        };
        assert_eq!(r.net_throughput(), 0);
        assert_eq!(r.gdp_contribution(), 0);
    }

    #[test]
    fn zero_waste_full_gdp() {
        let r = RegionGdp {
            region_id: 0,
            production_joules: 500,
            waste_joules: 0,
        };
        assert_eq!(r.gdp_contribution(), 500);
    }
}
