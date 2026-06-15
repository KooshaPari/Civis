//! Market price tracking stub (CIV-0100 §market).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Per-good clearing prices in fixed-point cents (stub; full clearing in CIV-0100 §3c).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketState {
    /// Good id → price in cents.
    pub prices: BTreeMap<String, i64>,
}

impl Default for MarketState {
    fn default() -> Self {
        let mut prices = BTreeMap::new();
        prices.insert("food".to_string(), 1_000);
        prices.insert("energy".to_string(), 1_000);
        Self { prices }
    }
}

impl MarketState {
    /// Current clearing prices (good id → cents).
    pub fn prices(&self) -> &BTreeMap<String, i64> {
        &self.prices
    }

    /// Advance one market tick: updates exactly one good's price from `tick` (deterministic).
    pub fn step(&mut self, tick: u64) {
        if self.prices.is_empty() {
            return;
        }
        let len = self.prices.len();
        let idx = tick as usize % len;
        let key = self
            .prices
            .keys()
            .nth(idx)
            .expect("non-empty prices")
            .clone();
        let delta = deterministic_price_delta(tick, &key);
        if let Some(price) = self.prices.get_mut(&key) {
            *price = price.saturating_add(delta);
        }
    }
}

/// Integer-only price delta from tick and good id (replay-stable).
fn deterministic_price_delta(tick: u64, good: &str) -> i64 {
    let mut mix = tick;
    for byte in good.as_bytes() {
        mix = mix.wrapping_mul(31).wrapping_add(u64::from(*byte));
    }
    (mix % 13) as i64 + 1
}

/// Pure emergent clearing price from local scarcity — Phase 1.
///
/// Returns `demand / (supply + 1)` so that price strictly rises as
/// demand-to-supply scarcity increases. No hardcoded price table:
/// the price *emerges* from the ratio itself (charter: price emerges).
///
/// - `supply` — available stock quantity (clamped to ≥ 0 internally).
/// - `demand` — unmet need / willingness-to-pay pressure (≥ 0).
///
/// Deterministic: identical inputs always produce identical output.
pub fn clearing_price(supply: f64, demand: f64) -> f64 {
    if demand <= 0.0 {
        return 0.0;
    }
    let effective_supply = supply.max(0.0) + 1.0;
    demand / effective_supply
}

#[cfg(test)]
fn run_tick_sequence(market: &mut MarketState, ticks: &[u64]) {
    for &tick in ticks {
        market.step(tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn step_updates_exactly_one_price_from_tick() {
        let mut market = MarketState::default();
        let before = market.prices.clone();
        let tick = 0;
        market.step(tick);
        let changed: Vec<_> = market
            .prices
            .iter()
            .filter(|(k, v)| before.get(*k) != Some(v))
            .collect();
        assert_eq!(changed.len(), 1);
        let (good, price) = changed[0];
        let expected = before[good] + deterministic_price_delta(tick, good);
        assert_eq!(*price, expected);
    }

    #[test]
    fn step_is_deterministic_for_same_tick() {
        let mut a = MarketState::default();
        let mut b = MarketState::default();
        a.step(7);
        b.step(7);
        assert_eq!(a.prices, b.prices);
    }

    /// Zero supply: empty price book is a no-op (no panic, no mutation).
    #[test]
    fn step_no_op_when_zero_supply() {
        let mut market = MarketState {
            prices: BTreeMap::new(),
        };
        market.step(0);
        market.step(42);
        assert!(market.prices.is_empty());
    }

    /// Single good: every tick updates that good only; delta matches `deterministic_price_delta`.
    #[test]
    fn step_single_good_updates_only_that_good() {
        let mut market = MarketState {
            prices: BTreeMap::from([("water".to_string(), 500)]),
        };
        let tick = 11;
        let before = market.prices.clone();
        market.step(tick);
        assert_eq!(market.prices.len(), 1);
        assert_eq!(
            market.prices["water"],
            before["water"] + deterministic_price_delta(tick, "water")
        );
    }

    /// Phase 1 — emergent clearing price from scarcity.
    /// (1) Price rises when scarcity rises. (2) Deterministic (same inputs → same output).
    #[test]
    fn clearing_price_rises_with_scarcity_and_is_deterministic() {
        // Scarcity: low supply, high demand → high price.
        let scarce = clearing_price(1.0, 100.0);
        let abundant = clearing_price(100.0, 100.0);
        assert!(
            scarce > abundant,
            "scarce price {scarce} must exceed abundant price {abundant}"
        );

        // More demand at same supply → higher price.
        let low_demand = clearing_price(10.0, 20.0);
        let high_demand = clearing_price(10.0, 80.0);
        assert!(
            high_demand > low_demand,
            "high-demand price {high_demand} must exceed low-demand price {low_demand}"
        );

        // Zero demand → zero price.
        assert_eq!(clearing_price(50.0, 0.0), 0.0);
        assert_eq!(clearing_price(0.0, 0.0), 0.0);

        // Negative supply is clamped to zero.
        assert_eq!(clearing_price(-10.0, 10.0), clearing_price(0.0, 10.0));

        // Determinism: identical inputs → identical output.
        assert_eq!(clearing_price(7.0, 42.0), clearing_price(7.0, 42.0));
    }

    proptest! {
        /// Same tick sequence => identical prices after N steps.
        #[test]
        fn same_tick_sequence_yields_identical_prices(
            ticks in prop::collection::vec(any::<u64>(), 0..100),
        ) {
            let mut a = MarketState::default();
            let mut b = MarketState::default();
            run_tick_sequence(&mut a, &ticks);
            run_tick_sequence(&mut b, &ticks);
            prop_assert_eq!(a.prices, b.prices);
        }

        /// All clearing prices stay strictly positive after any tick sequence.
        #[test]
        fn prices_remain_positive_after_n_steps(
            ticks in prop::collection::vec(any::<u64>(), 0..100),
        ) {
            let mut market = MarketState::default();
            run_tick_sequence(&mut market, &ticks);
            for (good, price) in &market.prices {
                prop_assert!(*price > 0, "price for {good} must be positive, got {price}");
            }
        }
    }
}
