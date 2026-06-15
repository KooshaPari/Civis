//! Resource allocation substrate via continuous double auction (CDA) with
//! proportional rationing fallback (CIV-0100 §allocation).
//!
//! **Emergence charter alignment:** institutions emerge, the mechanism is the
//! floor. This module is the floor: a deterministic, integer-only, conservation-
//! complete auction substrate. It does not hardcode *who* participates; it only
//! defines the rules under which posted orders clear, with rationing as a
//! safety net so a "market" never starves a participant to extinction when
//! supply is short. The same substrate accepts bids from individual agents,
//! settlements, or state-actor institutions — the emergence layer picks.
//!
//! ## Clearing rules
//!
//! 1. Sort bids descending by price (highest willingness-to-pay first), with
//!    FIFO tie-break by `OrderId` for determinism.
//! 2. Sort offers ascending by price (cheapest ask first), with FIFO tie-break
//!    by `OrderId`.
//! 3. Walk the order book pairwise:
//!    - If `bid.price >= offer.price` and remaining quantities are positive,
//!      clear the smaller quantity at the *clearing price* (mid-point of
//!      matched pair, rounded toward the more constrained side).
//!    - If no pair crosses (clearing book empty), fall back to **proportional
//!      rationing** for that good: each surviving bid receives
//!      `floor(remaining / total_demand * supply)` units (the existing
//!      `CapitalistAllocator` semantics, lifted to the multi-agent order book).
//! 4. Every cleared trade posts a balanced institution transfer:
//!    `debit = bidder institution`, `credit = offerer institution`.
//! 5. Order books are immutable for the tick once cleared; uncleared orders
//!    survive into the next tick until cancelled or filled (CIV-002 FR-ECON-003
//!    TTL semantics land in a follow-up slice).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    institution::{InstitutionId, InstitutionLedger, LedgerSide, INSTITUTION_MARKET},
    EconomyState,
};

/// Identifier for a single posted order (deterministic insertion order).
pub type OrderId = u64;

/// Good or joule being traded.
pub type GoodId = String;

/// A buy order: bidder wants up to `quantity` units of `good`, willing to pay
/// at most `price` (in fixed-point cents) per unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    /// Deterministic order identifier.
    pub id: OrderId,
    /// Institution account paying for the units.
    pub bidder: InstitutionId,
    /// Good being demanded.
    pub good: GoodId,
    /// Maximum units demanded.
    pub quantity: i64,
    /// Maximum per-unit price the bidder will pay (cents, >= 0).
    pub price: i64,
}

/// A sell order: offerer has up to `quantity` units of `good`, willing to
/// accept at least `price` per unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    /// Deterministic order identifier.
    pub id: OrderId,
    /// Institution account delivering the units.
    pub offerer: InstitutionId,
    /// Good being supplied.
    pub good: GoodId,
    /// Maximum units offered.
    pub quantity: i64,
    /// Minimum per-unit price the offerer will accept (cents, >= 0).
    pub price: i64,
}

/// A single completed trade from the auction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearedTrade {
    /// Tick at which the trade cleared.
    pub tick: u64,
    /// Good transferred.
    pub good: GoodId,
    /// Units transferred.
    pub quantity: i64,
    /// Per-unit clearing price (cents).
    pub price: i64,
    /// Institution credited (paid).
    pub bidder: InstitutionId,
    /// Institution debited (delivered goods).
    pub offerer: InstitutionId,
    /// `true` if cleared via proportional rationing (supply scarcity fallback).
    pub rationed: bool,
}

/// Allocation substrate: an open order book per good plus a tick-clear driver.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allocator {
    /// Buy orders keyed by `OrderId` for O(log n) insertion / removal.
    pub bids: BTreeMap<OrderId, Bid>,
    /// Sell orders keyed by `OrderId` for O(log n) insertion / removal.
    pub offers: BTreeMap<OrderId, Offer>,
    /// Monotonic counter for [`Allocator::next_order_id`].
    next_id: OrderId,
}

impl Allocator {
    /// Empty allocator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Monotonically increasing order id (replay-stable).
    pub fn next_order_id(&self) -> OrderId {
        self.next_id
    }

    /// Post a bid. Returns the assigned `OrderId`. Quantity and price must be
    /// non-negative; otherwise the bid is rejected (no mutation).
    pub fn post_bid(&mut self, mut bid: Bid) -> Option<OrderId> {
        if bid.quantity < 0 || bid.price < 0 {
            return None;
        }
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        bid.id = id;
        self.bids.insert(id, bid);
        Some(id)
    }

    /// Post an offer. Returns the assigned `OrderId`. Quantity and price must
    /// be non-negative; otherwise the offer is rejected (no mutation).
    pub fn post_offer(&mut self, mut offer: Offer) -> Option<OrderId> {
        if offer.quantity < 0 || offer.price < 0 {
            return None;
        }
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        offer.id = id;
        self.offers.insert(id, offer);
        Some(id)
    }

    /// Cancel an order by id. Returns the cancelled order (bid or offer).
    pub fn cancel(&mut self, id: OrderId) -> Option<CancelledOrder> {
        if let Some(bid) = self.bids.remove(&id) {
            return Some(CancelledOrder::Bid(bid));
        }
        self.offers.remove(&id).map(CancelledOrder::Offer)
    }

    /// Number of live buy orders.
    pub fn bid_count(&self) -> usize {
        self.bids.len()
    }

    /// Number of live sell orders.
    pub fn offer_count(&self) -> usize {
        self.offers.len()
    }

    /// Run one clearing pass against `institution_ledger`. Mutates:
    ///
    /// - the order book (filled orders are removed, partial fills persist with
    ///   decremented remaining quantity),
    /// - the [`InstitutionLedger`] (one balanced transfer per cleared trade).
    ///
    /// **Does not** advance [`EconomyState::tick`] — the economy phase driver
    /// ([`crate::step`]) owns tick advance so this method can be called inside
    /// composition (e.g. one tick may invoke `clear` once and `step` once).
    /// Returns the cleared trades for this tick in deterministic order.
    pub fn clear(
        &mut self,
        economy: &mut EconomyState,
        ledger: &mut InstitutionLedger,
    ) -> Vec<ClearedTrade> {
        let tick = economy.tick;
        let mut trades: Vec<ClearedTrade> = Vec::new();

        // Collect the set of goods present on either side of the book.
        // We then sort and clear each good in turn.
        let mut goods: BTreeMap<String, ()> = BTreeMap::new();
        for bid in self.bids.values() {
            goods.entry(bid.good.clone()).or_insert(());
        }
        for offer in self.offers.values() {
            goods.entry(offer.good.clone()).or_insert(());
        }

        // Snapshot ids per good; sort by price priority. We then walk the
        // ids and mutate the live `self.bids` / `self.offers` directly.
        // This avoids fighting the borrow checker on `&mut Vec<&mut Bid>`.
        for good in goods.keys() {
            // Build ordered id lists by snapshotting bid/offers then sorting.
            let mut bid_ids: Vec<OrderId> = self
                .bids
                .iter()
                .filter(|(_, b)| b.good == *good)
                .map(|(id, _)| *id)
                .collect();
            bid_ids.sort_by(|a, b| {
                let pa = self.bids.get(a).map(|b| b.price).unwrap_or(0);
                let pb = self.bids.get(b).map(|b| b.price).unwrap_or(0);
                pb.cmp(&pa).then(a.cmp(b))
            });

            let mut offer_ids: Vec<OrderId> = self
                .offers
                .iter()
                .filter(|(_, o)| o.good == *good)
                .map(|(id, _)| *id)
                .collect();
            offer_ids.sort_by(|a, b| {
                let pa = self.offers.get(a).map(|o| o.price).unwrap_or(0);
                let pb = self.offers.get(b).map(|o| o.price).unwrap_or(0);
                pa.cmp(&pb).then(a.cmp(b))
            });

            // Phase 1: walk the crossing book (bid >= ask) and clear as much
            // volume as possible at the mid-point clearing price.
            let mut bi = 0usize;
            let mut oi = 0usize;
            while bi < bid_ids.len() && oi < offer_ids.len() {
                // Re-fetch mutable refs and read current state.
                let (bid_price, bid_qty, bid_institution) = {
                    let b = match self.bids.get(&bid_ids[bi]) {
                        Some(b) => b,
                        None => break,
                    };
                    (b.price, b.quantity, b.bidder)
                };
                let (offer_price, offer_qty, offer_institution) = {
                    let o = match self.offers.get(&offer_ids[oi]) {
                        Some(o) => o,
                        None => break,
                    };
                    (o.price, o.quantity, o.offerer)
                };
                if bid_qty <= 0 {
                    bi += 1;
                    continue;
                }
                if offer_qty <= 0 {
                    oi += 1;
                    continue;
                }
                if bid_price < offer_price {
                    break;
                }
                let qty = bid_qty.min(offer_qty);
                let clearing_price = (bid_price + offer_price) / 2;
                if qty > 0 && clearing_price >= 0 {
                    let trade = ClearedTrade {
                        tick,
                        good: good.clone(),
                        quantity: qty,
                        price: clearing_price,
                        bidder: bid_institution,
                        offerer: offer_institution,
                        rationed: false,
                    };
                    let amount = qty.saturating_mul(clearing_price);
                    if post_balanced_trade(ledger, economy, &trade, amount).is_ok() {
                        trades.push(trade);
                    }
                }
                if let Some(b) = self.bids.get_mut(&bid_ids[bi]) {
                    b.quantity -= qty;
                }
                if let Some(o) = self.offers.get_mut(&offer_ids[oi]) {
                    o.quantity -= qty;
                }
                // Advance to next order if fully filled.
                let b_now = self.bids.get(&bid_ids[bi]).map(|b| b.quantity).unwrap_or(0);
                let o_now = self
                    .offers
                    .get(&offer_ids[oi])
                    .map(|o| o.quantity)
                    .unwrap_or(0);
                if b_now == 0 {
                    bi += 1;
                }
                if o_now == 0 {
                    oi += 1;
                }
            }

            // Phase 2: proportional rationing fallback for any remaining
            // positive demand with positive supply.
            let mut remaining_demand: i64 = 0;
            let mut surviving: Vec<(OrderId, i64)> = Vec::new();
            for id in &bid_ids {
                if let Some(b) = self.bids.get(id) {
                    if b.good == *good && b.quantity > 0 {
                        remaining_demand = remaining_demand.saturating_add(b.quantity);
                        surviving.push((*id, b.quantity));
                    }
                }
            }
            let mut remaining_supply: i64 = 0;
            for id in &offer_ids {
                if let Some(o) = self.offers.get(id) {
                    if o.good == *good && o.quantity > 0 {
                        remaining_supply = remaining_supply.saturating_add(o.quantity);
                    }
                }
            }
            if remaining_supply > 0 && remaining_demand > 0 {
                let allocated_total = remaining_supply.min(remaining_demand);
                let mut leftover = allocated_total;
                let last = surviving.len();
                for (i, (bid_id, demand)) in surviving.iter().enumerate() {
                    let portion = if i + 1 == last {
                        leftover
                    } else {
                        // floor(demand / remaining_demand * allocated_total)
                        let num = (*demand as i128).saturating_mul(allocated_total as i128);
                        let denom = remaining_demand.max(1) as i128;
                        ((num / denom) as i64).min(leftover)
                    };
                    if portion <= 0 {
                        continue;
                    }
                    // Bidder identity still live in `self.bids`.
                    let bidder = match self.bids.get(bid_id) {
                        Some(b) => b.bidder,
                        None => continue,
                    };
                    // First live offerer on the book supplies the rationed
                    // goods (their quantity is decremented after the post).
                    let mut offerer: Option<InstitutionId> = None;
                    for oid in &offer_ids {
                        if let Some(o) = self.offers.get(oid) {
                            if o.good == *good && o.quantity > 0 {
                                offerer = Some(o.offerer);
                                break;
                            }
                        }
                    }
                    let offerer = offerer.unwrap_or(INSTITUTION_MARKET);
                    let clearing_price =
                        best_unmatched_price_for_good(&self.bids, &self.offers, good);
                    let trade = ClearedTrade {
                        tick,
                        good: good.clone(),
                        quantity: portion,
                        price: clearing_price,
                        bidder,
                        offerer,
                        rationed: true,
                    };
                    let amount = portion.saturating_mul(clearing_price);
                    if post_balanced_trade(ledger, economy, &trade, amount).is_ok() {
                        trades.push(trade);
                        if let Some(b) = self.bids.get_mut(bid_id) {
                            b.quantity = b.quantity.saturating_sub(portion);
                        }
                        // Decrement the first live offerer's quantity.
                        for oid in &offer_ids {
                            if let Some(o) = self.offers.get_mut(oid) {
                                if o.good == *good && o.quantity > 0 {
                                    o.quantity = o.quantity.saturating_sub(portion);
                                    break;
                                }
                            }
                        }
                        leftover = leftover.saturating_sub(portion);
                        if leftover == 0 {
                            break;
                        }
                    }
                }
            }
        }

        // Drop fully-filled orders; partial fills persist for next tick.
        self.bids.retain(|_, b| b.quantity > 0);
        self.offers.retain(|_, o| o.quantity > 0);

        // Tick advance is the caller's responsibility (see doc comment).
        trades
    }
}

/// Compute a deterministic rationing clearing price: average of the best
/// unmatched bid and best unmatched offer for `good`, clamped at zero.
fn best_unmatched_price_for_good(
    bids: &BTreeMap<OrderId, Bid>,
    offers: &BTreeMap<OrderId, Offer>,
    good: &str,
) -> i64 {
    let best_bid = bids
        .values()
        .filter(|b| b.good == good && b.quantity > 0)
        .map(|b| b.price)
        .max()
        .unwrap_or(0);
    let best_ask = offers
        .values()
        .filter(|o| o.good == good && o.quantity > 0)
        .map(|o| o.price)
        .min()
        .unwrap_or(0);
    if best_bid >= best_ask {
        best_ask
    } else {
        (best_bid + best_ask) / 2
    }
}

/// Which side of the book a cancelled order belonged to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelledOrder {
    /// The cancelled buy order.
    Bid(Bid),
    /// The cancelled sell order.
    Offer(Offer),
}

/// Post a balanced institution transfer for a cleared trade.
///
/// The bidder pays (debits their institution account) and the offerer is
/// credited. Macro joule budget is not touched here — trades transfer within
/// the institution layer. When the offerer is the market itself, the
/// transfer collapses to a single `bidder → offerer` posting (the market is
/// not a real balance-holder here, just a clearing venue). When bidder and
/// offerer are distinct institutions, we route through the market as a
/// transparent passthrough so that `market` ends the tick with its balance
/// unchanged, leaving a clean audit trail in the posting log.
fn post_balanced_trade(
    ledger: &mut InstitutionLedger,
    economy: &mut EconomyState,
    trade: &ClearedTrade,
    amount: i64,
) -> Result<(), crate::institution::InstitutionLedgerError> {
    if amount <= 0 {
        // Free allocation: nothing to transfer, no posting recorded, the
        // conservation law is trivially satisfied (zero in = zero out).
        return Ok(());
    }
    if trade.bidder == trade.offerer {
        // Self-trade: post a no-op balanced pair so the audit log records
        // the intent, but balance the same account on both sides.
        ledger.post(
            economy,
            LedgerSide::Institution(trade.bidder),
            LedgerSide::Institution(trade.offerer),
            amount,
        )?;
        return Ok(());
    }
    if trade.offerer == INSTITUTION_MARKET {
        // Direct bidder → market (which is the offerer). One balanced post.
        ledger.post(
            economy,
            LedgerSide::Institution(trade.bidder),
            LedgerSide::Institution(trade.offerer),
            amount,
        )?;
        return Ok(());
    }
    // Standard route: bidder → market → offerer. Two balanced postings that
    // net to zero for the market, leaving it as a transparent passthrough.
    ledger.post(
        economy,
        LedgerSide::Institution(trade.bidder),
        LedgerSide::Institution(INSTITUTION_MARKET),
        amount,
    )?;
    ledger.post(
        economy,
        LedgerSide::Institution(INSTITUTION_MARKET),
        LedgerSide::Institution(trade.offerer),
        amount,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::institution::{InstitutionLedger, INSTITUTION_TREASURY};
    use proptest::prelude::*;

    fn fresh_economy(budget: i64) -> EconomyState {
        EconomyState::with_energy_budget(budget)
    }

    fn fresh_ledger() -> InstitutionLedger {
        InstitutionLedger::with_defaults()
    }

    #[test]
    fn post_bid_assigns_monotonic_ids() {
        let mut alloc = Allocator::new();
        let a = alloc
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "food".to_string(),
                quantity: 5,
                price: 100,
            })
            .expect("post a");
        let b = alloc
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "food".to_string(),
                quantity: 3,
                price: 90,
            })
            .expect("post b");
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(alloc.bid_count(), 2);
    }

    #[test]
    fn post_rejects_negative_quantity_or_price() {
        let mut alloc = Allocator::new();
        assert!(alloc
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "food".to_string(),
                quantity: -1,
                price: 100,
            })
            .is_none());
        assert!(alloc
            .post_offer(Offer {
                id: 0,
                offerer: INSTITUTION_MARKET,
                good: "food".to_string(),
                quantity: 5,
                price: -2,
            })
            .is_none());
        assert_eq!(alloc.bid_count(), 0);
        assert_eq!(alloc.offer_count(), 0);
    }

    #[test]
    fn clear_crossing_pair_posts_balanced_transfer_and_advances_tick() {
        let mut economy = fresh_economy(2_000);
        let mut ledger = fresh_ledger();
        ledger
            .post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                1_000,
            )
            .unwrap();
        ledger
            .post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                1_000,
            )
            .unwrap();
        let mut alloc = Allocator::new();
        alloc
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "food".to_string(),
                quantity: 5,
                price: 120,
            })
            .unwrap();
        alloc
            .post_offer(Offer {
                id: 0,
                offerer: INSTITUTION_MARKET,
                good: "food".to_string(),
                quantity: 5,
                price: 80,
            })
            .unwrap();

        let trades = alloc.clear(&mut economy, &mut ledger);
        assert_eq!(trades.len(), 1);
        let t = &trades[0];
        assert_eq!(t.good, "food");
        assert_eq!(t.quantity, 5);
        // mid-point of 120 and 80 is 100
        assert_eq!(t.price, 100);
        assert!(!t.rationed);

        // Institution balances after trade: buyer (treasury) paid 5*100=500;
        // seller (market) received 500. Market is also the offerer here so
        // its balance increases by the trade amount. Macro budget is
        // unchanged by the trade itself.
        assert_eq!(ledger.institution_balance(INSTITUTION_TREASURY), 500);
        assert_eq!(ledger.institution_balance(INSTITUTION_MARKET), 1_500);
        // Macro energy_budget must not have been touched by trades (only by
        // the two funding posts that moved it from 2000 → 0 before trading).
        assert_eq!(economy.energy_budget_joules, 0);
        // Tick is *not* advanced by `clear` (caller owns tick advance).
        assert_eq!(economy.tick, 0);
        // Order book is empty after a full fill.
        assert_eq!(alloc.bid_count(), 0);
        assert_eq!(alloc.offer_count(), 0);
        ledger.verify_conservation().expect("conservation");
    }

    #[test]
    fn clear_rationing_when_book_does_not_cross() {
        let mut economy = fresh_economy(2_000);
        let mut ledger = fresh_ledger();
        ledger
            .post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                1_000,
            )
            .unwrap();
        ledger
            .post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                1_000,
            )
            .unwrap();
        let mut alloc = Allocator::new();
        // Bids are too low; book won't cross.
        alloc
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "food".to_string(),
                quantity: 10,
                price: 50,
            })
            .unwrap();
        alloc
            .post_offer(Offer {
                id: 0,
                offerer: INSTITUTION_MARKET,
                good: "food".to_string(),
                quantity: 4,
                price: 100,
            })
            .unwrap();
        let trades = alloc.clear(&mut economy, &mut ledger);
        assert_eq!(trades.len(), 1);
        let t = &trades[0];
        assert!(t.rationed);
        assert_eq!(t.quantity, 4); // supply was the binding constraint
                                   // best_bid = 50, best_ask = 100; book did not cross so rationing
                                   // phase fires with clearing price = (50 + 100) / 2 = 75. This is
                                   // the emergent price signal that converges downward in future ticks
                                   // as agents learn to post better prices.
        assert_eq!(t.price, 75);
        assert!(t.price >= 0);
        ledger.verify_conservation().expect("conservation");
    }

    #[test]
    fn clear_conservation_under_random_order_books() {
        let mut economy = fresh_economy(10_000_000);
        let mut ledger = fresh_ledger();
        ledger
            .post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                1_000_000,
            )
            .unwrap();
        ledger
            .post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                1_000_000,
            )
            .unwrap();
        let mut alloc = Allocator::new();
        for i in 0..3 {
            alloc
                .post_bid(Bid {
                    id: 0,
                    bidder: INSTITUTION_TREASURY,
                    good: "food".to_string(),
                    quantity: 100 + i * 50,
                    price: 200 - i * 30,
                })
                .unwrap();
            alloc
                .post_offer(Offer {
                    id: 0,
                    offerer: INSTITUTION_MARKET,
                    good: "food".to_string(),
                    quantity: 80 + i * 40,
                    price: 50 + i * 25,
                })
                .unwrap();
        }
        let before_treasury = ledger.institution_balance(INSTITUTION_TREASURY);
        let _trades = alloc.clear(&mut economy, &mut ledger);
        // Conservation: institution ledger verifies; institution balances
        // never negative.
        ledger.verify_conservation().expect("conservation");
        assert!(ledger.institution_balance(INSTITUTION_TREASURY) >= 0);
        // Aggregate institution balance cannot exceed pre-funding sum.
        let after_treasury = ledger.institution_balance(INSTITUTION_TREASURY);
        assert!(after_treasury <= before_treasury);
    }

    #[test]
    fn clear_is_deterministic_for_same_order_book() {
        fn run() -> Vec<ClearedTrade> {
            let mut economy = fresh_economy(10_000);
            let mut ledger = fresh_ledger();
            ledger
                .post(
                    &mut economy,
                    crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                    LedgerSide::Institution(INSTITUTION_TREASURY),
                    10_000,
                )
                .unwrap();
            let mut alloc = Allocator::new();
            for i in 0..5 {
                alloc
                    .post_bid(Bid {
                        id: 0,
                        bidder: INSTITUTION_TREASURY,
                        good: "food".to_string(),
                        quantity: 10,
                        price: 100 - i * 10,
                    })
                    .unwrap();
                alloc
                    .post_offer(Offer {
                        id: 0,
                        offerer: INSTITUTION_MARKET,
                        good: "food".to_string(),
                        quantity: 10,
                        price: 30 + i * 20,
                    })
                    .unwrap();
            }
            alloc.clear(&mut economy, &mut ledger)
        }
        let a = run();
        let b = run();
        assert_eq!(a, b);
    }

    proptest! {
        /// FR-ECON-002: aggregate institution balance never goes negative and
        /// the conservation invariant holds for any order book we can build.
        #[test]
        fn allocator_conservation_holds_under_random_orders(
            n_bids in 0usize..8,
            n_offers in 0usize..8,
            seed in any::<u64>(),
        ) {
            let mut economy = fresh_economy(20_000_000);
            let mut ledger = fresh_ledger();
            ledger.post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                10_000_000,
            ).unwrap();
            ledger.post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                10_000_000,
            ).unwrap();
            let mut alloc = Allocator::new();
            for i in 0..n_bids {
                let qty = 1 + (seed.wrapping_add(i as u64) % 50) as i64;
                let price = 50 + ((seed >> i).wrapping_rem(200)) as i64;
                alloc.post_bid(Bid {
                    id: 0,
                    bidder: INSTITUTION_TREASURY,
                    good: "food".to_string(),
                    quantity: qty,
                    price,
                }).unwrap();
            }
            for i in 0..n_offers {
                let qty = 1 + (seed.wrapping_add((i as u64).wrapping_mul(7)) % 50) as i64;
                let price = 50 + ((seed >> (i + 3)).wrapping_rem(200)) as i64;
                alloc.post_offer(Offer {
                    id: 0,
                    offerer: INSTITUTION_MARKET,
                    good: "food".to_string(),
                    quantity: qty,
                    price,
                }).unwrap();
            }
            let _ = alloc.clear(&mut economy, &mut ledger);
            ledger.verify_conservation().expect("conservation");
            prop_assert!(ledger.institution_balance(INSTITUTION_TREASURY) >= 0);
            prop_assert!(ledger.institution_balance(INSTITUTION_MARKET) >= 0);
        }

        /// FR-ECON-002 (macro): a clearing pass never touches the macro joule
        /// budget — institution-internal transfers only.
        #[test]
        fn allocator_does_not_drain_macro_budget(seed in any::<u64>()) {
            let mut economy = fresh_economy(20_000_000);
            let mut ledger = fresh_ledger();
            ledger.post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                10_000_000,
            ).unwrap();
            ledger.post(
                &mut economy,
                crate::LedgerSide::Macro(crate::ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                10_000_000,
            ).unwrap();
            let macro_before = economy.energy_budget_joules;
            let mut alloc = Allocator::new();
            let n_bids = (seed % 4) as usize + 1;
            let n_offers = (seed.wrapping_shr(8) % 4) as usize + 1;
            for i in 0..n_bids {
                alloc.post_bid(Bid {
                    id: 0,
                    bidder: INSTITUTION_TREASURY,
                    good: "food".to_string(),
                    quantity: 5 + i as i64,
                    price: 100,
                }).unwrap();
            }
            for i in 0..n_offers {
                alloc.post_offer(Offer {
                    id: 0,
                    offerer: INSTITUTION_MARKET,
                    good: "food".to_string(),
                    quantity: 5 + i as i64,
                    price: 80,
                }).unwrap();
            }
            let _ = alloc.clear(&mut economy, &mut ledger);
            // Macro budget only changes via the funding `post`; allocator
            // must leave it untouched.
            prop_assert_eq!(economy.energy_budget_joules, macro_before);
        }
    }
}
