//! Market price tracking (CIV-0100 §market).
//!
//! Deterministic clearing prices per good in fixed-point cents. Prices
//! emerge from the supply/demand imbalance (see [`MarketState::apply_pressure`])
//! plus a deterministic per-tick drift (see [`MarketState::step`]).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Default clearing price (cents) used when [`MarketState::apply_pressure`] is
/// invoked on a good that has no entry yet. Mirrors the seeds in
/// [`MarketState::default`].
pub const DEFAULT_PRICE_CENTS: i64 = 1_000;

/// Per-good clearing prices in fixed-point cents.
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

    /// Mutable accessor — required by the engine for the `TECH_STORAGE`
    /// scarcity-dampening branch in `phase_economy`.
    pub fn prices_mut(&mut self) -> &mut BTreeMap<String, i64> {
        &mut self.prices
    }

    /// Insert `good` with `initial_price` cents if it has no entry yet.
    /// Returns the current price (existing or freshly inserted). Idempotent
    /// — safe to call from scenarios that add new goods each run.
    pub fn ensure_good(&mut self, good: &str, initial_price: i64) -> i64 {
        *self
            .prices
            .entry(good.to_string())
            .or_insert(initial_price.max(1))
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
    ///
    /// If `good` is not yet in the price book (e.g. a scenario introduces a new
    /// good without seeding its price), it is lazily inserted at
    /// [`DEFAULT_PRICE_CENTS`] before the pressure is applied. This makes the
    /// function self-healing — engine code can pass new good ids without
    /// silently no-opping on a missing key.
    pub fn apply_pressure(&mut self, good: &str, demand: i64, supply: i64) {
        /// Maximum price move per application (cents).
        const MAX_DELTA: i64 = 8;
        let price = self
            .prices
            .entry(good.to_string())
            .or_insert(DEFAULT_PRICE_CENTS);
        let imbalance = demand.saturating_sub(supply);
        let delta = imbalance.clamp(-MAX_DELTA, MAX_DELTA);
        *price = (*price + delta).max(1);
    }

    /// Arithmetic mean of all clearing prices in cents. Returns `None` when
    /// the price book is empty (no goods traded). Engine / HUD can use this
    /// as a "consumer price index" or surface the value on the chronicle.
    pub fn mean_clearing_price(&self) -> Option<i64> {
        if self.prices.is_empty() {
            return None;
        }
        let sum: i64 = self.prices.values().sum();
        let count = self.prices.len() as i64;
        Some(sum / count)
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

    /// `apply_pressure` self-heals when the good is not yet in the price book
    /// — it lazily inserts at `DEFAULT_PRICE_CENTS` and applies the move. The
    /// previous version silently no-op'd, which caused scenarios that introduce
    /// new goods to render flat prices forever.
    #[test]
    fn apply_pressure_self_heals_for_new_good() {
        let mut market = MarketState::default();
        assert!(!market.prices.contains_key("tools"));
        // Demand > supply: price should rise from the seeded default.
        market.apply_pressure("tools", 1_000, 100);
        let expected = DEFAULT_PRICE_CENTS + 8;
        assert_eq!(market.prices.get("tools"), Some(&expected));
    }

    /// `apply_pressure` on an empty market lazily inserts the good and applies
    /// the pressure delta. This is the cold-start scenario where `phase_economy`
    /// is called before any other phase seeds the price book.
    #[test]
    fn apply_pressure_works_on_empty_market() {
        let mut market = MarketState {
            prices: BTreeMap::new(),
        };
        market.apply_pressure("food", 100, 50);
        let expected = DEFAULT_PRICE_CENTS + 8;
        assert_eq!(market.prices.get("food"), Some(&expected));
    }

    /// `ensure_good` is idempotent: a second call with a different
    /// `initial_price` does not overwrite the current price (which may have
    /// already drifted from previous ticks).
    #[test]
    fn ensure_good_is_idempotent() {
        let mut market = MarketState::default();
        let first = market.ensure_good("metal", 800);
        assert_eq!(first, 800);
        // Drift the price away from the seed.
        market.apply_pressure("metal", 100, 1_000);
        let drifted = market.ensure_good("metal", 999_999);
        assert!(
            drifted < 999_999,
            "ensure_good must not overwrite an existing entry (got {drifted})"
        );
    }

    /// `ensure_good` clamps the seed to at least 1 cent so the price floor
    /// invariant (`apply_pressure` floors at 1) holds on first contact.
    #[test]
    fn ensure_good_floors_initial_price_at_one() {
        let mut market = MarketState::default();
        assert_eq!(market.ensure_good("luxury", 0), 1);
        assert_eq!(market.ensure_good("luxury", -50), 1);
    }

    /// `mean_clearing_price` is the arithmetic mean of the price book.
    /// Empty book → `None` (no goods traded, no price to surface).
    #[test]
    fn mean_clearing_price_averages_existing_goods() {
        let market = MarketState::default();
        let mean = market.mean_clearing_price().expect("two goods seeded");
        assert_eq!(mean, 1_000);
    }

    #[test]
    fn mean_clearing_price_is_none_when_empty() {
        let market = MarketState {
            prices: BTreeMap::new(),
        };
        assert_eq!(market.mean_clearing_price(), None);
    }

    /// Adding a high-priced good skews the mean — sanity check that the
    /// accessor reflects the full book, not just the original seeds.
    #[test]
    fn mean_clearing_price_reflects_all_goods() {
        let mut market = MarketState::default();
        market.ensure_good("luxury", 4_000);
        // (1000 + 1000 + 4000) / 3 = 2000
        assert_eq!(market.mean_clearing_price(), Some(2_000));
    }
}
