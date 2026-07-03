//! Emergent price computation from supply/demand ratios (FR-ECON-EMERGE-001).
//!
//! Prices are derived per-cluster per-good from the ratio of demand to supply.
//! No absolute price oracle exists; all prices are relative and emergent.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::stocks::{Good, Stocks, GOODS};

/// Opaque cluster identifier.
pub type ClusterId = u32;

/// Per-resource price for a single cluster.
///
/// Prices are dimensionless multipliers on a notional base price of `1.0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceState {
    /// The cluster this price state belongs to.
    pub cluster_id: ClusterId,
    /// Per-good price multipliers (demand/supply ratio clamped to `[0.1, 10.0]`).
    pub prices: BTreeMap<Good, f32>,
}

impl PriceState {
    /// Returns the price for `good`, defaulting to `1.0` if not set.
    pub fn price(&self, good: Good) -> f32 {
        self.prices.get(&good).copied().unwrap_or(1.0)
    }
}

/// Compute the emergent price for one good given supply and demand.
///
/// Formula: `base_price * (demand / supply).clamp(0.1, 10.0)`.
/// When supply is zero the multiplier is clamped to its maximum (`10.0`).
#[must_use]
pub fn compute_price(supply: f32, demand: f32, base_price: f32) -> f32 {
    let ratio = if supply <= 0.0 {
        10.0_f32
    } else {
        (demand / supply).clamp(0.1, 10.0)
    };
    base_price * ratio
}

/// Derive a [`PriceState`] for a cluster from its current stocks.
///
/// Each good's demand is modelled as `1.0` (unit demand) while supply is the
/// normalised stock level (`quantity / 100.0`, floored at `0.0`). Callers can
/// pass higher-resolution demand signals by constructing `PriceState` directly.
#[must_use]
pub fn update_cluster_prices(cluster_id: ClusterId, stocks: &Stocks) -> PriceState {
    let mut prices = BTreeMap::new();
    for good in GOODS {
        let supply = (stocks.get(good) as f32 / 100.0).max(0.0);
        let demand = 1.0_f32;
        prices.insert(good, compute_price(supply, demand, 1.0));
    }
    PriceState { cluster_id, prices }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stocks::Stocks;

    #[test]
    fn price_rises_when_supply_less_than_demand() {
        // supply 0.5 (50 units / 100), demand 1.0 → ratio 2.0
        let price = compute_price(0.5, 1.0, 1.0);
        assert!(price > 1.0, "price should rise above base when supply < demand");
    }

    #[test]
    fn price_falls_when_surplus() {
        // supply 2.0 (200 units / 100), demand 1.0 → ratio 0.5
        let price = compute_price(2.0, 1.0, 1.0);
        assert!(price < 1.0, "price should fall below base when supply > demand");
    }

    #[test]
    fn price_clamped_at_maximum_when_supply_zero() {
        let price = compute_price(0.0, 1.0, 1.0);
        assert_eq!(price, 10.0);
    }

    #[test]
    fn price_clamped_at_minimum_when_extreme_surplus() {
        let price = compute_price(1000.0, 1.0, 1.0);
        assert_eq!(price, 0.1);
    }

    #[test]
    fn update_cluster_prices_reflects_stock_level() {
        let mut stocks = Stocks::default();
        // Very high stock → low price
        stocks.add(Good::Food, 500);
        let state = update_cluster_prices(0, &stocks);
        assert!(
            state.price(Good::Food) < 1.0,
            "price should be below base with surplus stock"
        );
        // Zero stock → max price
        let empty = Stocks::default();
        let state2 = update_cluster_prices(1, &empty);
        assert_eq!(state2.price(Good::Food), 10.0);
    }
}
