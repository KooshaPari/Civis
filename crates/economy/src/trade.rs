//! Trade flows driven by price differentials between clusters (FR-ECON-EMERGE-002).
//!
//! Surplus goods flow from low-price clusters (high supply) to high-price clusters
//! (high scarcity). Flow volume is proportional to the price differential.

use serde::{Deserialize, Serialize};

use crate::prices::{ClusterId, PriceState};
use crate::stocks::{Good, GOODS};

/// Minimum price differential required to trigger a trade flow.
const PRICE_DIFFERENTIAL_THRESHOLD: f32 = 0.2;

/// A directed trade flow of a single good between two clusters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeFlow {
    /// Cluster exporting the good (lower price / higher supply).
    pub from_cluster: ClusterId,
    /// Cluster importing the good (higher price / lower supply).
    pub to_cluster: ClusterId,
    /// The good being traded.
    pub good: Good,
    /// Volume of trade (proportional to price differential).
    pub volume: f32,
}

/// Compute trade flows across all cluster pairs for each good.
///
/// For each pair `(a, b)` and each good, if `|price_a - price_b| >
/// [`PRICE_DIFFERENTIAL_THRESHOLD`]` a flow is generated from the
/// lower-priced (surplus) cluster toward the higher-priced (scarcity) cluster.
/// Volume equals the raw differential scaled by 10.
#[must_use]
pub fn compute_trade_flows(prices: &[PriceState]) -> Vec<TradeFlow> {
    let mut flows = Vec::new();

    for i in 0..prices.len() {
        for j in (i + 1)..prices.len() {
            let a = &prices[i];
            let b = &prices[j];

            for good in GOODS {
                let pa = a.price(good);
                let pb = b.price(good);
                let diff = pa - pb;

                if diff.abs() <= PRICE_DIFFERENTIAL_THRESHOLD {
                    continue;
                }

                let (from, to) = if diff < 0.0 {
                    // a is cheaper → surplus at a, flows to b
                    (a.cluster_id, b.cluster_id)
                } else {
                    // b is cheaper → surplus at b, flows to a
                    (b.cluster_id, a.cluster_id)
                };

                flows.push(TradeFlow {
                    from_cluster: from,
                    to_cluster: to,
                    good,
                    volume: diff.abs() * 10.0,
                });
            }
        }
    }

    flows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prices::PriceState;
    use crate::stocks::Good;
    use std::collections::BTreeMap;

    fn make_price_state(cluster_id: ClusterId, food_price: f32) -> PriceState {
        let mut prices = BTreeMap::new();
        for good in GOODS {
            prices.insert(good, 1.0_f32);
        }
        prices.insert(Good::Food, food_price);
        PriceState { cluster_id, prices }
    }

    #[test]
    fn trade_flows_from_surplus_to_scarcity() {
        // Cluster 0: food price 0.5 (surplus), Cluster 1: food price 2.0 (scarcity)
        let states = vec![make_price_state(0, 0.5), make_price_state(1, 2.0)];
        let flows = compute_trade_flows(&states);
        let food_flow = flows.iter().find(|f| f.good == Good::Food).unwrap();
        assert_eq!(food_flow.from_cluster, 0, "surplus cluster exports");
        assert_eq!(food_flow.to_cluster, 1, "scarcity cluster imports");
        assert!(food_flow.volume > 0.0);
    }

    #[test]
    fn no_flow_when_prices_equal() {
        let states = vec![make_price_state(0, 1.0), make_price_state(1, 1.0)];
        let flows = compute_trade_flows(&states);
        let food_flows: Vec<_> = flows.iter().filter(|f| f.good == Good::Food).collect();
        assert!(food_flows.is_empty(), "no flow when price differential is zero");
    }

    #[test]
    fn no_flow_below_threshold() {
        // Differential of 0.1 is below the 0.2 threshold
        let states = vec![make_price_state(0, 1.0), make_price_state(1, 1.1)];
        let flows = compute_trade_flows(&states);
        let food_flows: Vec<_> = flows.iter().filter(|f| f.good == Good::Food).collect();
        assert!(food_flows.is_empty());
    }

    #[test]
    fn volume_proportional_to_differential() {
        let states = vec![make_price_state(0, 0.5), make_price_state(1, 2.5)];
        let flows = compute_trade_flows(&states);
        let food_flow = flows.iter().find(|f| f.good == Good::Food).unwrap();
        // diff = 2.0, volume = 2.0 * 10 = 20.0
        assert!((food_flow.volume - 20.0).abs() < 0.001);
    }
}
