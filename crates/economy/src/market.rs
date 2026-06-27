//! Market price tracking stub (CIV-0100 §market).
//!
//! Two complementary types live in this module:
//!
//! * [`MarketState`] — per-good clearing-price stub retained for backwards
//!   compatibility. Each `step` tick deterministically nudges exactly one
//!   good's price; the round-trip is replay-stable.
//! * [`MultiGoodMarket`] — the FR-ECON-003 order-book market. One
//!   [`OrderBook`] per [`GoodId`]. Each tick the caller invokes
//!   [`MultiGoodMarket::clear_all`] which matches bids against asks per good
//!   at the midpoint of the crossed prices, dropping orders older than
//!   `uncleared_ttl_ticks`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::stocks::{deficit, surplus, Good};
use crate::trade_routes::{Settlement, SettlementId};

/// Default clearing price (cents) for goods inserted on first sighting.
pub const DEFAULT_PRICE_CENTS: i64 = 1_000;
/// Maximum absolute price change per `apply_pressure` call, in cents.
pub const MAX_PRESSURE_DELTA_CENTS: i64 = 100;
/// Minimum a price can ever be after `apply_pressure` (cents).
pub const MIN_PRICE_CENTS: i64 = 1;
/// Default smoothing factor for FR-CIV-MARKET price updates.
///
/// Higher values make price changes smaller per tick while keeping the
/// signal deterministic and integer-only.
pub const DEFAULT_SMOOTHING_FACTOR: i64 = 8;

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
    /// Computes a deterministic price delta from the demand/supply imbalance,
    /// then smooths the move by [`DEFAULT_SMOOTHING_FACTOR`]. Goods missing
    /// from the price book are seeded at [`DEFAULT_PRICE_CENTS`] first
    /// (self-healing — engine code can pass new good ids without silent
    /// failure).
    ///
    /// Returns the new price in cents.
    pub fn apply_pressure(&mut self, good: &str, supply: i64, demand: i64) -> i64 {
        self.apply_pressure_with_smoothing(good, supply, demand, DEFAULT_SMOOTHING_FACTOR)
    }

    /// Apply supply/demand pressure with an explicit smoothing factor.
    ///
    /// FR-CIV-MARKET: the price rises when demand exceeds supply and falls on
    /// surplus. The smoothing factor dampens the per-tick adjustment so the
    /// update is emergent instead of binary.
    pub fn apply_pressure_with_smoothing(
        &mut self,
        good: &str,
        supply: i64,
        demand: i64,
        smoothing_factor: i64,
    ) -> i64 {
        let supply = supply.max(0);
        let demand = demand.max(0);
        let smoothing_factor = smoothing_factor.max(1);
        let imbalance = demand - supply;
        let baseline = supply.saturating_add(demand).max(1);
        // Integer-only smoothing: move toward the scarcity signal in small
        // deterministic steps. Positive imbalance raises price, negative
        // imbalance lowers it.
        let delta = imbalance
            .saturating_mul(MAX_PRESSURE_DELTA_CENTS)
            / baseline.saturating_mul(smoothing_factor);
        let delta = delta.clamp(-MAX_PRESSURE_DELTA_CENTS, MAX_PRESSURE_DELTA_CENTS);
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

}

/// Integer-only price delta from tick and good id (replay-stable).
fn deterministic_price_delta(tick: u64, good: &str) -> i64 {
    let mut mix = tick;
    for byte in good.as_bytes() {
        mix = mix.wrapping_mul(31).wrapping_add(u64::from(*byte));
    }
    (mix % 13) as i64 + 1
}

/// FR-CIV-MARKET: deterministic low-price -> high-price settlement flow.
///
/// When one settlement's price is lower than another's for the same good, the
/// lower-price settlement is the supplier and the higher-price settlement is
/// the buyer. The flow quantity is bounded by:
///
/// * the supplier's post-tick surplus,
/// * the buyer's post-tick deficit, and
/// * the price gap damped by the smoothing factor.
///
/// The result is deterministic and uses only integer math.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementTradeFlow {
    /// Source settlement id.
    pub from_settlement: SettlementId,
    /// Destination settlement id.
    pub to_settlement: SettlementId,
    /// Good being moved.
    pub good: Good,
    /// Quantity transferred.
    pub qty: i64,
    /// Source settlement price in cents.
    pub low_price_cents: i64,
    /// Destination settlement price in cents.
    pub high_price_cents: i64,
    /// Settled midpoint price in cents.
    pub settled_price_cents: i64,
}

/// Compute a deterministic low-to-high price trade flow for a single good.
///
/// The caller provides the current price estimates for both settlements. The
/// returned quantity is the emergent trade volume that should move from the
/// cheaper settlement to the more expensive one.
pub fn settlement_trade_flow(
    low: &Settlement,
    high: &Settlement,
    good: Good,
    low_price_cents: i64,
    high_price_cents: i64,
    smoothing_factor: i64,
) -> Option<SettlementTradeFlow> {
    if low.id == high.id {
        return None;
    }
    if low_price_cents >= high_price_cents {
        return None;
    }

    let supply = surplus(&low.stocks, &low.profile, good).max(0);
    let demand = deficit(&high.stocks, &high.profile, good).max(0);
    if supply <= 0 || demand <= 0 {
        return None;
    }

    let smoothing_factor = smoothing_factor.max(1);
    let price_gap = high_price_cents - low_price_cents;
    let gap_limited_qty = (price_gap / smoothing_factor).max(1);
    let qty = supply.min(demand).min(gap_limited_qty);
    if qty <= 0 {
        return None;
    }

    Some(SettlementTradeFlow {
        from_settlement: low.id,
        to_settlement: high.id,
        good,
        qty,
        low_price_cents,
        high_price_cents,
        settled_price_cents: (low_price_cents + high_price_cents) / 2,
    })
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
    use crate::stocks::{ProductionProfile, Stocks};
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

    fn settlement_with_profile(
        id: u32,
        food_stock: i64,
        production: [i64; 6],
        consumption: [i64; 6],
    ) -> Settlement {
        let mut settlement = Settlement::new(id, glam::IVec3::ZERO);
        let mut stocks = Stocks::default();
        stocks.set(Good::Food, food_stock);
        settlement.stocks = stocks;
        settlement.profile = ProductionProfile::new(production, consumption);
        settlement
    }

    /// FR-CIV-MARKET — scarcity lifts price, surplus lowers it, and the
    /// resulting low-price -> high-price flow narrows the spread
    /// deterministically.
    #[test]
    fn fr_civ_market_scarcity_surplus_and_trade_equalize() {
        let mut market = MarketState::default();

        let scarce_before = market.apply_pressure_with_smoothing("food", 10, 100, 4);
        let surplus_before = market.apply_pressure_with_smoothing("wood", 100, 10, 4);

        assert!(scarce_before > DEFAULT_PRICE_CENTS);
        assert!(surplus_before < DEFAULT_PRICE_CENTS);

        let supplier = settlement_with_profile(
            1,
            12,
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
        );
        let buyer = settlement_with_profile(
            2,
            0,
            [0, 0, 0, 0, 0, 0],
            [8, 0, 0, 0, 0, 0],
        );

        let low_price = 90;
        let high_price = 150;
        let flow = settlement_trade_flow(
            &supplier,
            &buyer,
            Good::Food,
            low_price,
            high_price,
            DEFAULT_SMOOTHING_FACTOR,
        )
        .expect("expected deterministic low->high flow");

        assert_eq!(flow.from_settlement, supplier.id);
        assert_eq!(flow.to_settlement, buyer.id);
        assert!(flow.qty > 0);
        assert!(flow.settled_price_cents > low_price);
        assert!(flow.settled_price_cents < high_price);
        assert_eq!(flow.settled_price_cents, 120);

        let after_gap = (high_price - flow.settled_price_cents)
            .saturating_sub(flow.settled_price_cents - low_price);
        assert!(after_gap < (high_price - low_price));
    }
}

// ---------------------------------------------------------------------------
// Multi-good order-book market (FR-ECON-003 / CIV-0100 §3c).
//
// Sits alongside [`MarketState`] (per-good clearing-price stub). Each tick,
// [`MultiGoodMarket::clear_all`] matches bids against asks per good at the
// midpoint of the crossed prices, dropping orders older than
// `uncleared_ttl_ticks`.
// ---------------------------------------------------------------------------

/// Identifier for a tradeable good (FR-ECON-003).
///
/// Wraps a `u32` so each good has a stable, orderable, hashable identity
/// usable as a `BTreeMap` key. The 32-bit space is enough for both
/// hand-typed good registries and procedurally generated sectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GoodId(pub u32);

/// Side of a resting order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    /// Buy-side order.
    Bid,
    /// Sell-side order.
    Ask,
}

/// Resting order in an [`OrderBook`].
///
/// An order is placed by exactly one agent on exactly one side (a buyer's bid
/// or a seller's ask). The `agent_id` field is the placer; the
/// `qty`/`price_cents`/`placed_tick` are the order's terms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    /// Agent that placed the order (buyer for a bid, seller for an ask).
    pub agent_id: u32,
    /// Remaining quantity in integer units (positive; zero is dropped on clear).
    pub qty: i64,
    /// Limit price in fixed-point cents.
    pub price_cents: i64,
    /// Simulation tick at which the order was placed.
    pub placed_tick: u64,
}

/// Single-good order book: resting bids and asks for one [`GoodId`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBook {
    /// Resting bids (buy orders).
    pub bids: Vec<Order>,
    /// Resting asks (sell orders).
    pub asks: Vec<Order>,
}

impl OrderBook {
    /// New empty order book.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Filled trade emitted by [`MultiGoodMarket::clear_all`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    /// Agent that bought the good (bid side).
    pub buyer: u32,
    /// Agent that sold the good (ask side).
    pub seller: u32,
    /// Good that was traded.
    pub good: GoodId,
    /// Quantity filled.
    pub qty: i64,
    /// Trade price in fixed-point cents (midpoint of matched bid/ask).
    pub price_cents: i64,
    /// Simulation tick at which the trade was cleared.
    pub tick: u64,
}

/// Multi-good market (FR-ECON-003).
///
/// One [`OrderBook`] per good. Cleared each tick via [`Self::clear_all`].
/// Orders older than `uncleared_ttl_ticks` (relative to `placed_tick`) are
/// dropped at clear time. Default TTL is 1 tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiGoodMarket {
    /// Per-good order books, keyed by [`GoodId`].
    pub books: BTreeMap<GoodId, OrderBook>,
    /// Orders older than this many ticks are dropped on clear.
    pub uncleared_ttl_ticks: u64,
}

impl Default for MultiGoodMarket {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiGoodMarket {
    /// Create a new market with `uncleared_ttl_ticks = 1`.
    pub fn new() -> Self {
        Self::with_ttl(1)
    }

    /// Create a new market with an explicit `uncleared_ttl_ticks`.
    pub fn with_ttl(uncleared_ttl_ticks: u64) -> Self {
        Self {
            books: BTreeMap::new(),
            uncleared_ttl_ticks,
        }
    }

    /// Ensure an order book exists for `good`, returning a mutable handle.
    /// Idempotent: cheap to call before placing an order so callers can
    /// inspect or pre-populate the book.
    pub fn ensure_book(&mut self, good: GoodId) -> &mut OrderBook {
        self.books.entry(good).or_default()
    }

    /// Place a bid (buy) order for `good` at `qty` × `price_cents` placed
    /// at `placed_tick`. Creates the book's entry on first use.
    pub fn place_bid(
        &mut self,
        good: GoodId,
        agent_id: u32,
        qty: i64,
        price_cents: i64,
        placed_tick: u64,
    ) {
        self.books
            .entry(good)
            .or_default()
            .bids
            .push(Order {
                agent_id,
                qty,
                price_cents,
                placed_tick,
            });
    }

    /// Place an ask (sell) order for `good` at `qty` × `price_cents` placed
    /// at `placed_tick`. Creates the book's entry on first use.
    pub fn place_ask(
        &mut self,
        good: GoodId,
        agent_id: u32,
        qty: i64,
        price_cents: i64,
        placed_tick: u64,
    ) {
        self.books
            .entry(good)
            .or_default()
            .asks
            .push(Order {
                agent_id,
                qty,
                price_cents,
                placed_tick,
            });
    }

    /// Push a pre-built [`Order`] onto the appropriate side of the book for `good`.
    /// Convenience for callers that have already constructed an `Order` value.
    pub fn place_order(&mut self, good: GoodId, side: Side, order: Order) {
        let book = self.books.entry(good).or_default();
        match side {
            Side::Bid => book.bids.push(order),
            Side::Ask => book.asks.push(order),
        }
    }

    /// Clear all books at `current_tick` and return the trades emitted.
    ///
    /// Algorithm per good (BTreeMap iteration order, ascending `GoodId`):
    /// 1. Drop orders whose `placed_tick + uncleared_ttl_ticks < current_tick`
    ///    (i.e., the order's TTL window has fully elapsed before this tick).
    /// 2. Sort bids descending by `price_cents` (stable; ties by `agent_id` asc).
    /// 3. Sort asks ascending by `price_cents` (stable; ties by `agent_id` asc).
    /// 4. Two-pointer walk: while `bid.price_cents >= ask.price_cents` and
    ///    either side has remaining qty, emit a `Trade` at the midpoint
    ///    `(bid.price + ask.price) / 2` for `min(bid.qty, ask.qty)`. Decrement
    ///    both sides; advance past any order that fully filled.
    /// 5. Remove zero-qty orders; keep partial fills as residual.
    ///
    /// Determinism: the only input-dependent orderings are (a) the BTreeMap
    /// key order (canonical, `Ord` on `GoodId`) and (b) the per-side sort
    /// (canonical, `Ord` on `(price, agent_id)`). Insertion order does not
    /// affect the trade vector.
    pub fn clear_all(&mut self, current_tick: u64) -> Vec<Trade> {
        let ttl = self.uncleared_ttl_ticks;
        let mut trades = Vec::new();

        for (good, book) in self.books.iter_mut() {
            // 1. Drop expired orders. An order expires when
            //    placed_tick + ttl < current_tick, i.e., the order's TTL
            //    window has fully elapsed before this tick boundary.
            book.bids
                .retain(|o| o.placed_tick.saturating_add(ttl) >= current_tick);
            book.asks
                .retain(|o| o.placed_tick.saturating_add(ttl) >= current_tick);

            // 2. Sort bids: price desc, then agent_id asc (stable).
            book.bids.sort_by(|a, b| {
                b.price_cents
                    .cmp(&a.price_cents)
                    .then(a.agent_id.cmp(&b.agent_id))
            });
            // 3. Sort asks: price asc, then agent_id asc (stable).
            book.asks.sort_by(|a, b| {
                a.price_cents
                    .cmp(&b.price_cents)
                    .then(a.agent_id.cmp(&b.agent_id))
            });

            // 4. Two-pointer match walk.
            let mut i = 0usize;
            let mut j = 0usize;
            while i < book.bids.len() && j < book.asks.len() {
                // Defensive: skip any zero-qty order that snuck in.
                if book.bids[i].qty <= 0 {
                    i += 1;
                    continue;
                }
                if book.asks[j].qty <= 0 {
                    j += 1;
                    continue;
                }
                if book.bids[i].price_cents < book.asks[j].price_cents {
                    break;
                }
                let trade_qty = book.bids[i].qty.min(book.asks[j].qty);
                // Midpoint in integer cents. Truncation toward zero on odd sums
                // is acceptable for FR-ECON-003 (no fractional cents).
                let trade_price = (book.bids[i].price_cents + book.asks[j].price_cents) / 2;
                trades.push(Trade {
                    buyer: book.bids[i].agent_id,
                    seller: book.asks[j].agent_id,
                    good: *good,
                    qty: trade_qty,
                    price_cents: trade_price,
                    tick: current_tick,
                });
                book.bids[i].qty -= trade_qty;
                book.asks[j].qty -= trade_qty;
                if book.bids[i].qty == 0 {
                    i += 1;
                }
                if book.asks[j].qty == 0 {
                    j += 1;
                }
            }

            // 5. Remove fully-filled orders; keep partial fills.
            book.bids.retain(|o| o.qty > 0);
            book.asks.retain(|o| o.qty > 0);
        }

        trades
    }
}

#[cfg(test)]
mod multigood_tests {
    use super::*;

    /// 1. FR-ECON-003 — three bids (prices 10, 9, 8) and three asks (prices
    /// 1, 2, 3) on the same good, all unit qty, all at `placed_tick = 0`.
    /// After sort: bids desc = [10, 9, 8]; asks asc = [1, 2, 3]. Each bid
    /// crosses the next-cheapest ask, producing exactly 3 trades at
    /// midpoint 5 each. Different `agent_id`s per order verify the per-pair
    /// `(buyer, seller)` routing.
    #[test]
    fn simple_book_three_bids_three_asks_clear_at_midpoints() {
        let mut m = MultiGoodMarket::new();
        let grain = GoodId(1);
        // Bids: buyer 20 @ 10, buyer 30 @ 9, buyer 40 @ 8.
        m.place_bid(grain, 20, 1, 10, 0);
        m.place_bid(grain, 30, 1, 9, 0);
        m.place_bid(grain, 40, 1, 8, 0);
        // Asks: seller 100 @ 1, seller 200 @ 2, seller 300 @ 3.
        m.place_ask(grain, 100, 1, 1, 0);
        m.place_ask(grain, 200, 1, 2, 0);
        m.place_ask(grain, 300, 1, 3, 0);

        let trades = m.clear_all(0);
        assert_eq!(trades.len(), 3);
        assert_eq!(
            trades[0],
            Trade {
                buyer: 20,
                seller: 100,
                good: grain,
                qty: 1,
                price_cents: 5, // (10+1)/2
                tick: 0,
            }
        );
        assert_eq!(
            trades[1],
            Trade {
                buyer: 30,
                seller: 200,
                good: grain,
                qty: 1,
                price_cents: 5, // (9+2)/2
                tick: 0,
            }
        );
        assert_eq!(
            trades[2],
            Trade {
                buyer: 40,
                seller: 300,
                good: grain,
                qty: 1,
                price_cents: 5, // (8+3)/2
                tick: 0,
            }
        );
        // All orders fully filled — books empty after clear.
        let book = &m.books[&grain];
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    /// 2. FR-ECON-003 — an empty book (or a good that was never seen) is a
    /// no-op: no panic, no mutation, no trades emitted, no book entry
    /// created (lazy initialization).
    #[test]
    fn empty_book_for_new_good_is_no_op() {
        let mut m = MultiGoodMarket::new();
        let water = GoodId(42);
        let trades = m.clear_all(0);
        assert!(trades.is_empty());
        assert!(!m.books.contains_key(&water));
    }

    /// 3. FR-ECON-003 — orders older than `uncleared_ttl_ticks` are dropped.
    /// With TTL = 1: an order placed at tick 0 is still in-window at tick 1
    /// (0 + 1 >= 1) but expired by tick 2 (0 + 1 < 2), so it must be gone
    /// from the book after a second clear.
    #[test]
    fn orders_past_uncleared_ttl_dropped_on_second_tick() {
        let mut m = MultiGoodMarket::with_ttl(1);
        let grain = GoodId(1);
        m.place_bid(grain, 10, 5, 100, 0);
        m.place_ask(grain, 20, 5, 90, 0);

        // Tick 0: order in window. One trade at midpoint 95.
        let trades = m.clear_all(0);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].qty, 5);
        assert_eq!(trades[0].price_cents, 95); // (100+90)/2

        // Tick 1: order still in window (0+1 >= 1) but no new resting
        // orders → no new trades.
        let trades = m.clear_all(1);
        assert!(trades.is_empty());

        // Tick 2: TTL elapsed (0+1 < 2) → order dropped.
        let trades = m.clear_all(2);
        assert!(trades.is_empty());
        let book = &m.books[&grain];
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    /// 4. FR-ECON-003 — when the best bid is strictly below the best ask,
    /// no trades are emitted and both sides remain intact on the book.
    #[test]
    fn bid_price_below_ask_price_emits_no_trades() {
        let mut m = MultiGoodMarket::new();
        let ore = GoodId(7);
        m.place_bid(ore, 1, 5, 40, 0);
        m.place_ask(ore, 2, 5, 60, 0);
        let trades = m.clear_all(0);
        assert!(trades.is_empty());
        let book = &m.books[&ore];
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 1);
        assert_eq!(book.bids[0].qty, 5);
        assert_eq!(book.asks[0].qty, 5);
    }

    /// 5. FR-ECON-003 — determinism. Two markets built from the same final
    /// set of orders, populated in different insertion orders, must produce
    /// identical trade vectors. Sorting inside `clear_all` is the source of
    /// truth; insertion order is irrelevant. (The spec phrase "same book,
    /// same insertion order" is satisfied trivially; this is the stronger
    /// statement.)
    #[test]
    fn clear_all_is_deterministic_independent_of_insertion_order() {
        let grain = GoodId(1);
        let ore = GoodId(2);

        let mut a = MultiGoodMarket::new();
        a.place_bid(grain, 10, 2, 50, 0);
        a.place_ask(grain, 20, 2, 40, 0);
        a.place_bid(ore, 30, 1, 30, 0);
        a.place_ask(ore, 40, 1, 25, 0);

        let mut b = MultiGoodMarket::new();
        b.place_ask(ore, 40, 1, 25, 0);
        b.place_bid(ore, 30, 1, 30, 0);
        b.place_ask(grain, 20, 2, 40, 0);
        b.place_bid(grain, 10, 2, 50, 0);

        let ta = a.clear_all(0);
        let tb = b.clear_all(0);
        assert_eq!(ta, tb);
        // And the determinism holds across explicit tick values too.
        let ta2 = a.clear_all(7);
        let tb2 = b.clear_all(7);
        assert_eq!(ta2, tb2);
        assert_eq!(ta.len(), 2);
    }

    /// 6. FR-ECON-003 — partial fill. A bid of qty 10 crosses an ask of
    /// qty 3 at compatible prices. The resulting trade is exactly qty 3
    /// (the smaller side); the bid is left with qty 7 on the book, the
    /// ask is fully consumed.
    #[test]
    fn partial_fill_emits_one_trade_of_smaller_qty() {
        let mut m = MultiGoodMarket::new();
        let iron = GoodId(3);
        m.place_bid(iron, 1, 10, 100, 0); // bid 10 @ 100
        m.place_ask(iron, 2, 3, 80, 0); //  ask  3 @  80

        let trades = m.clear_all(0);
        assert_eq!(trades.len(), 1);
        assert_eq!(
            trades[0],
            Trade {
                buyer: 1,
                seller: 2,
                good: iron,
                qty: 3,
                price_cents: 90, // (100+80)/2
                tick: 0,
            }
        );
        let book = &m.books[&iron];
        // Bid residual: 10 - 3 = 7.
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.bids[0].agent_id, 1);
        assert_eq!(book.bids[0].qty, 7);
        // Ask fully consumed.
        assert!(book.asks.is_empty());
    }

    /// 7. FR-ECON-003 — leftover order. A bid of qty 3 crosses an ask of
    /// qty 10. Exactly one trade at qty 3 (the smaller side); the bid
    /// fully fills, the ask has qty 7 left on the book as a residual.
    #[test]
    fn leftover_ask_remains_after_larger_ask_partial_fill() {
        let mut m = MultiGoodMarket::new();
        let wood = GoodId(4);
        m.place_bid(wood, 1, 3, 100, 0); // bid  3 @ 100
        m.place_ask(wood, 2, 10, 80, 0); // ask 10 @  80

        let trades = m.clear_all(0);
        assert_eq!(trades.len(), 1);
        assert_eq!(
            trades[0],
            Trade {
                buyer: 1,
                seller: 2,
                good: wood,
                qty: 3,
                price_cents: 90, // (100+80)/2
                tick: 0,
            }
        );
        let book = &m.books[&wood];
        // Bid fully filled (3 == 3), gone.
        assert!(book.bids.is_empty());
        // Ask residual: 10 - 3 = 7.
        assert_eq!(book.asks.len(), 1);
        assert_eq!(book.asks[0].agent_id, 2);
        assert_eq!(book.asks[0].qty, 7);
        assert_eq!(book.asks[0].price_cents, 80);
    }

    /// Extras — `ensure_book` is idempotent and yields a mutable handle.
    #[test]
    fn ensure_book_is_idempotent_and_returns_mutable_handle() {
        let mut m = MultiGoodMarket::new();
        let coal = GoodId(11);

        // First call materialises the book.
        {
            let book = m.ensure_book(coal);
            assert!(book.bids.is_empty());
            assert!(book.asks.is_empty());
            book.bids.push(Order {
                agent_id: 5,
                qty: 1,
                price_cents: 100,
                placed_tick: 0,
            });
        }

        // Second call returns the same book (entry already exists).
        let book = m.ensure_book(coal);
        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.bids[0].agent_id, 5);
    }

    /// Extras — `clear_all` across multiple goods respects BTreeMap ordering
    /// and emits per-good trades in ascending `GoodId` order.
    #[test]
    fn clear_all_emits_trades_in_ascending_good_id_order() {
        let mut m = MultiGoodMarket::new();
        // Insert in non-sorted order; trades must still come out sorted by good.
        m.place_bid(GoodId(20), 1, 1, 50, 0);
        m.place_ask(GoodId(20), 2, 1, 40, 0);
        m.place_bid(GoodId(5), 3, 1, 30, 0);
        m.place_ask(GoodId(5), 4, 1, 25, 0);

        let trades = m.clear_all(0);
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].good, GoodId(5));
        assert_eq!(trades[1].good, GoodId(20));
    }
}
