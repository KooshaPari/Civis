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

    /// Nudge a good's clearing price toward supply/demand equilibrium
    /// (FR-CIV-0100 §3d). The price EMERGES from the imbalance between `demand`
    /// and `supply` rather than a scripted curve: it rises when demand exceeds
    /// supply, falls when supply exceeds demand, and never drops below 1.
    /// The per-application move is capped so prices walk toward equilibrium.
    pub fn apply_pressure(&mut self, good: &str, demand: i64, supply: i64) {
        /// Maximum price move per application (cents).
        const MAX_DELTA: i64 = 8;
        if let Some(price) = self.prices.get_mut(good) {
            let imbalance = demand.saturating_sub(supply);
            let delta = imbalance.clamp(-MAX_DELTA, MAX_DELTA);
            *price = (*price + delta).max(1);
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

    /// FR-CIV-0100 §3d — price rises when demand exceeds supply (emergent).
    #[test]
    fn apply_pressure_raises_price_when_demand_exceeds_supply() {
        let mut market = MarketState::default();
        let before = market.prices["food"];
        market.apply_pressure("food", 1_000, 100);
        assert!(market.prices["food"] > before);
    }

    /// FR-CIV-0100 §3d — price falls when supply exceeds demand.
    #[test]
    fn apply_pressure_lowers_price_when_supply_exceeds_demand() {
        let mut market = MarketState::default();
        let before = market.prices["food"];
        market.apply_pressure("food", 100, 1_000);
        assert!(market.prices["food"] < before);
    }

    #[test]
    fn prices_accessor_returns_same_map_reference() {
        let mut market = MarketState::default();
        let ptr_before = market.prices() as *const BTreeMap<String, i64>;
        market.step(3);
        market.apply_pressure("food", 500, 100);
        let ptr_after = market.prices() as *const BTreeMap<String, i64>;
        assert_eq!(ptr_before, ptr_after);
        assert_eq!(market.prices().len(), 2);
        assert_eq!(market.prices().get("food"), market.prices.get("food"));
        assert_eq!(market.prices().get("energy"), market.prices.get("energy"));
    }

    /// FR-CIV-0100 §3d — price never drops below 1 even under huge surplus.
    #[test]
    fn apply_pressure_floors_price_at_one() {
        let mut market = MarketState {
            prices: BTreeMap::from([("food".to_string(), 1)]),
        };
        market.apply_pressure("food", 0, 1_000_000);
        assert_eq!(market.prices["food"], 1);
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
