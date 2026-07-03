//! Market shocks from disasters and demand events (FR-ECON-EMERGE-004).
//!
//! Shocks translate external events (disasters, demand surges) into price
//! multipliers on top of the emergent supply/demand pricing in [`crate::prices`].
//!
//! The disaster kinds mirror [`civ_engine::disasters::DisasterKind`] without
//! depending on that crate; callers translate at the integration boundary.

use serde::{Deserialize, Serialize};

use crate::prices::{ClusterId, PriceState};
use crate::stocks::Good;

/// Severity bounds for shock multipliers.
const MIN_SEVERITY: f32 = 0.0;
const MAX_SEVERITY: f32 = 1.0;

/// A market shock that perturbs cluster prices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarketShock {
    /// A disaster reduces the supply of a good, spiking its price.
    ///
    /// `severity` in `[0.0, 1.0]` where `1.0` wipes out the entire supply.
    /// Price multiplier: `1.0 + severity * 9.0` (up to ×10 at max severity).
    DisasterSupplyShock {
        /// Target cluster.
        cluster: ClusterId,
        /// Good whose supply was disrupted.
        good: Good,
        /// Fractional severity in `[0.0, 1.0]`.
        severity: f32,
    },
    /// A sudden increase in demand spikes the price of a good.
    ///
    /// `magnitude` in `[0.0, 1.0]` where `1.0` doubles demand.
    /// Price multiplier: `1.0 + magnitude`.
    DemandShock {
        /// Target cluster.
        cluster: ClusterId,
        /// Good with spiked demand.
        good: Good,
        /// Fractional demand magnitude in `[0.0, 1.0]`.
        magnitude: f32,
    },
}

impl MarketShock {
    /// Returns the cluster affected by this shock.
    pub fn cluster(&self) -> ClusterId {
        match self {
            MarketShock::DisasterSupplyShock { cluster, .. } => *cluster,
            MarketShock::DemandShock { cluster, .. } => *cluster,
        }
    }

    /// Returns the good affected by this shock.
    pub fn good(&self) -> Good {
        match self {
            MarketShock::DisasterSupplyShock { good, .. } => *good,
            MarketShock::DemandShock { good, .. } => *good,
        }
    }
}

/// Apply a [`MarketShock`] to the matching [`PriceState`], if found in `prices`.
///
/// Prices are multiplied by the shock's computed multiplier. The result is
/// clamped to `[0.1, 10.0]` to stay within the emergent price bounds.
pub fn apply_shock(prices: &mut [PriceState], shock: &MarketShock) {
    let target = shock.cluster();
    let good = shock.good();

    let multiplier = shock_multiplier(shock);

    if let Some(state) = prices.iter_mut().find(|s| s.cluster_id == target) {
        let entry = state.prices.entry(good).or_insert(1.0);
        *entry = (*entry * multiplier).clamp(0.1, 10.0);
    }
}

/// Compute the price multiplier for a shock.
fn shock_multiplier(shock: &MarketShock) -> f32 {
    match shock {
        MarketShock::DisasterSupplyShock { severity, .. } => {
            let s = severity.clamp(MIN_SEVERITY, MAX_SEVERITY);
            // At severity 0: ×1.0 (no effect). At severity 1: ×10.0 (max price).
            1.0 + s * 9.0
        }
        MarketShock::DemandShock { magnitude, .. } => {
            let m = magnitude.clamp(MIN_SEVERITY, MAX_SEVERITY);
            // At magnitude 0: ×1.0. At magnitude 1: ×2.0.
            1.0 + m
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prices::PriceState;
    use crate::stocks::{Good, GOODS};
    use std::collections::BTreeMap;

    fn baseline_price_state(cluster_id: ClusterId) -> PriceState {
        let prices = GOODS.iter().map(|&g| (g, 1.0_f32)).collect();
        PriceState { cluster_id, prices }
    }

    #[test]
    fn disaster_shock_spikes_price() {
        let mut prices = vec![baseline_price_state(0)];
        let shock = MarketShock::DisasterSupplyShock {
            cluster: 0,
            good: Good::Food,
            severity: 0.5,
        };
        apply_shock(&mut prices, &shock);
        assert!(
            prices[0].price(Good::Food) > 1.0,
            "disaster shock must spike food price above base"
        );
    }

    #[test]
    fn max_severity_disaster_reaches_price_cap() {
        let mut prices = vec![baseline_price_state(0)];
        let shock = MarketShock::DisasterSupplyShock {
            cluster: 0,
            good: Good::Water,
            severity: 1.0,
        };
        apply_shock(&mut prices, &shock);
        assert_eq!(prices[0].price(Good::Water), 10.0);
    }

    #[test]
    fn demand_shock_raises_price() {
        let mut prices = vec![baseline_price_state(1)];
        let shock = MarketShock::DemandShock {
            cluster: 1,
            good: Good::Wood,
            magnitude: 1.0,
        };
        apply_shock(&mut prices, &shock);
        assert!((prices[0].price(Good::Wood) - 2.0).abs() < 0.001);
    }

    #[test]
    fn shock_targets_correct_cluster_only() {
        let mut prices = vec![baseline_price_state(0), baseline_price_state(1)];
        let shock = MarketShock::DisasterSupplyShock {
            cluster: 0,
            good: Good::Metal,
            severity: 1.0,
        };
        apply_shock(&mut prices, &shock);
        assert_eq!(prices[0].price(Good::Metal), 10.0, "cluster 0 should be affected");
        assert_eq!(prices[1].price(Good::Metal), 1.0, "cluster 1 must be unaffected");
    }

    #[test]
    fn zero_severity_shock_is_no_op() {
        let mut prices = vec![baseline_price_state(0)];
        let shock = MarketShock::DisasterSupplyShock {
            cluster: 0,
            good: Good::Tools,
            severity: 0.0,
        };
        apply_shock(&mut prices, &shock);
        assert!((prices[0].price(Good::Tools) - 1.0).abs() < 0.001);
    }
}
