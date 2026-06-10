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

    /// Clear one good's market by excess-demand price adjustment (FR-ECON-003).
    ///
    /// Moves the price toward equilibrium: excess demand (`demand > supply`)
    /// pushes the price up, excess supply pushes it down, and a balanced market
    /// leaves it unchanged. The adjustment is the normalized excess demand
    /// `(demand - supply) / (demand + supply)` in `[-1, 1]`, scaled by
    /// `ADJUST_BPS`, applied in integer fixed-point. Price is floored at 1 cent
    /// so a good never becomes free or negative. Unknown goods are inserted at
    /// the [`Default`] reference price before clearing.
    ///
    /// Deterministic and integer-only, so it is replay-stable.
    pub fn clear(&mut self, good: &str, supply: i64, demand: i64) {
        const ADJUST_BPS: i64 = 2_000; // max ±20% move per clearing step
        let price = self.prices.entry(good.to_string()).or_insert(1_000);

        let total = supply.saturating_add(demand);
        if total <= 0 {
            return; // no market: nobody supplying or demanding
        }
        let excess = demand.saturating_sub(supply); // >0 shortage, <0 glut
        // delta_bps = ADJUST_BPS * excess / total, in [-ADJUST_BPS, ADJUST_BPS]
        let delta_bps = ADJUST_BPS.saturating_mul(excess) / total;
        let delta = (*price).saturating_mul(delta_bps) / 10_000;
        *price = (*price).saturating_add(delta).max(1);
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

    #[test]
    fn clear_raises_price_on_excess_demand() {
        let mut m = MarketState::default();
        let before = m.prices["food"];
        m.clear("food", 10, 40); // demand >> supply
        assert!(m.prices["food"] > before, "shortage must raise price");
    }

    #[test]
    fn clear_lowers_price_on_excess_supply() {
        let mut m = MarketState::default();
        let before = m.prices["food"];
        m.clear("food", 40, 10); // supply >> demand
        assert!(m.prices["food"] < before, "glut must lower price");
    }

    #[test]
    fn clear_holds_price_at_equilibrium() {
        let mut m = MarketState::default();
        let before = m.prices["food"];
        m.clear("food", 25, 25); // balanced
        assert_eq!(m.prices["food"], before, "balanced market is stable");
    }

    #[test]
    fn clear_inserts_unknown_good_at_reference_then_adjusts() {
        let mut m = MarketState {
            prices: BTreeMap::new(),
        };
        m.clear("ore", 5, 15); // unknown good, shortage
        assert!(m.prices["ore"] > 1_000, "inserted at 1000 ref then raised");
    }

    #[test]
    fn clear_is_noop_for_empty_market() {
        let mut m = MarketState::default();
        let before = m.prices.clone();
        m.clear("food", 0, 0); // nobody trading
        assert_eq!(m.prices, before);
    }

    proptest! {
        /// Clearing never drives a price to zero or negative, for any supply/demand.
        #[test]
        fn clear_keeps_price_positive(
            supply in 0i64..1_000_000,
            demand in 0i64..1_000_000,
            rounds in 1usize..50,
        ) {
            let mut m = MarketState::default();
            for _ in 0..rounds {
                m.clear("food", supply, demand);
            }
            prop_assert!(m.prices["food"] >= 1, "price floored at 1, got {}", m.prices["food"]);
        }

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
