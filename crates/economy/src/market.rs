//! Market price tracking stub (CIV-0100 §market).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Default clearing price (cents) for goods inserted on first sighting.
pub const DEFAULT_PRICE_CENTS: i64 = 1_000;
/// Maximum absolute price change per `apply_pressure` call, in cents.
pub const MAX_PRESSURE_DELTA_CENTS: i64 = 100;
/// Minimum a price can ever be after `apply_pressure` (cents).
pub const MIN_PRICE_CENTS: i64 = 1;

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

    /// Ensure a good has a price entry. Idempotent — returns the current price
    /// (or seeds `DEFAULT_PRICE_CENTS` when first seen).
    pub fn ensure_good(&mut self, good: &str) -> i64 {
        *self
            .prices
            .entry(good.to_string())
            .or_insert(DEFAULT_PRICE_CENTS)
    }

    /// Mutable access to the underlying price book. Engine-side scarcity
    /// dampening (e.g. `phase_economy`'s `TECH_STORAGE` branch) calls this to
    /// nudge a good's price directly.
    pub fn prices_mut(&mut self) -> &mut BTreeMap<String, i64> {
        &mut self.prices
    }

    /// Apply supply/demand pressure to a single good's price.
    ///
    /// Computes `pressure = (demand - supply) / max(supply, 1)`, clamped to
    /// `[-1, 1]`, then nudges the price by `pressure * MAX_PRESSURE_DELTA_CENTS`
    /// (saturating). Goods missing from the price book are seeded at
    /// [`DEFAULT_PRICE_CENTS`] first (self-healing — engine code can pass new
    /// good ids without silent failure).
    ///
    /// Returns the new price in cents.
    pub fn apply_pressure(&mut self, good: &str, supply: i64, demand: i64) -> i64 {
        let supply = supply.max(0);
        let demand = demand.max(0);
        let denom = supply.max(1);
        let raw = demand - supply;
        // Clamp pressure to [-9, 9] in fixed-point integer math (0.9 max magnitude).
        let pressure = if raw >= denom {
            9
        } else if raw <= -denom {
            -9
        } else {
            // raw in [-denom+1, denom-1]; scale to [-9, 9] keeping sign.
            let sign = raw.signum();
            let abs_pressure = (raw.abs() * 10) / denom; // 0..=9 (max = 9 since raw < denom)
            sign * abs_pressure.clamp(0, 9)
        };
        // delta = pressure * (MAX_PRESSURE_DELTA_CENTS / 10). Pressure is in [-9, 9]
        // so delta is in [-MAX_PRESSURE_DELTA_CENTS, MAX_PRESSURE_DELTA_CENTS].
        let delta = pressure
            .saturating_mul(MAX_PRESSURE_DELTA_CENTS / 10)
            .clamp(-MAX_PRESSURE_DELTA_CENTS, MAX_PRESSURE_DELTA_CENTS);
        let current = self.ensure_good(good);
        let new_price = current.saturating_add(delta).max(MIN_PRICE_CENTS);
        self.prices.insert(good.to_string(), new_price);
        new_price
    }

    /// Arithmetic mean of all clearing prices in cents. `None` when the
    /// price book is empty. Used as a 'consumer price index' for the
    /// chronicle / HUD.
    pub fn mean_clearing_price(&self) -> Option<i64> {
        if self.prices.is_empty() {
            return None;
        }
        let sum: i64 = self.prices.values().copied().sum();
        let count = self.prices.len() as i64;
        Some(sum / count)
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

        /// `apply_pressure` always keeps prices strictly positive regardless
        /// of arbitrary supply/demand inputs.
        #[test]
        fn apply_pressure_keeps_prices_positive(
            supply in 0i64..1_000,
            demand in 0i64..1_000,
        ) {
            let mut market = MarketState::default();
            market.apply_pressure("widget", supply, demand);
            for (good, price) in &market.prices {
                prop_assert!(*price > 0, "price for {good} must stay > 0, got {price}");
            }
        }
    }

    // ---- Market price-dynamics tests (L5-110) ---------------------------------

    #[test]
    fn apply_pressure_self_heals_for_new_good() {
        let mut market = MarketState::default();
        assert!(!market.prices.contains_key("ore"));
        // supply == demand => no price pressure, just the seed value.
        let price = market.apply_pressure("ore", 10, 10);
        assert_eq!(price, DEFAULT_PRICE_CENTS);
        assert!(market.prices.contains_key("ore"));
    }

    #[test]
    fn apply_pressure_zero_supply_zero_demand_is_neutral() {
        let mut market = MarketState {
            prices: BTreeMap::from([("wood".to_string(), 750)]),
        };
        let new_price = market.apply_pressure("wood", 0, 0);
        // Zero supply + zero demand => raw = 0, pressure = 0, no delta.
        assert_eq!(new_price, 750);
    }

    #[test]
    fn apply_pressure_demand_outstrips_supply_lifts_price() {
        let mut market = MarketState {
            prices: BTreeMap::from([("wood".to_string(), 1_000)]),
        };
        let new_price = market.apply_pressure("wood", 1, 100);
        // raw = 99, denom = 1, raw >= denom => pressure = +9 => delta = +90.
        assert!(new_price > 1_000, "expected price lift, got {new_price}");
        assert!(new_price <= 1_000 + MAX_PRESSURE_DELTA_CENTS);
    }

    #[test]
    fn apply_pressure_supply_outstrips_demand_drops_price() {
        let mut market = MarketState {
            prices: BTreeMap::from([("wood".to_string(), 1_000)]),
        };
        let new_price = market.apply_pressure("wood", 100, 1);
        assert!(new_price < 1_000, "expected price drop, got {new_price}");
        assert!(new_price >= 1_000 - MAX_PRESSURE_DELTA_CENTS);
    }

    #[test]
    fn apply_pressure_floors_at_min_price() {
        let mut market = MarketState {
            prices: BTreeMap::from([("wood".to_string(), MIN_PRICE_CENTS)]),
        };
        let new_price = market.apply_pressure("wood", 1_000, 0);
        assert_eq!(new_price, MIN_PRICE_CENTS);
    }

    #[test]
    fn ensure_good_is_idempotent() {
        let mut market = MarketState::default();
        let p1 = market.ensure_good("new_good");
        let p2 = market.ensure_good("new_good");
        assert_eq!(p1, p2);
        assert_eq!(p1, DEFAULT_PRICE_CENTS);
    }

    #[test]
    fn mean_clearing_price_is_none_when_empty() {
        let market = MarketState {
            prices: BTreeMap::new(),
        };
        assert_eq!(market.mean_clearing_price(), None);
    }

    #[test]
    fn mean_clearing_price_averages_existing_goods() {
        let market = MarketState {
            prices: BTreeMap::from([
                ("a".to_string(), 100),
                ("b".to_string(), 200),
                ("c".to_string(), 300),
            ]),
        };
        assert_eq!(market.mean_clearing_price(), Some(200));
    }

    #[test]
    fn mean_clearing_price_reflects_apply_pressure_changes() {
        let mut market = MarketState {
            prices: BTreeMap::from([
                ("a".to_string(), 1_000),
                ("b".to_string(), 1_000),
            ]),
        };
        let before = market.mean_clearing_price();
        market.apply_pressure("a", 0, 1_000); // strong demand => price up
        let after = market.mean_clearing_price();
        assert!(after.unwrap() > before.unwrap());
    }
}
