//! External coverage tests for civ-economy (FR-CIV-TEST-006).
//!
//! Targets four pub fns/paths that have no external integration test:
//!   1. `verify_ledger_conservation` — `UnbalancedEntry` error variant
//!   2. `step` — no-change path (tick-close entry NOT appended when budget unchanged)
//!   3. `MultiGoodMarket::place_order` — push pre-built Order onto bid/ask side
//!   4. `MultiGoodMarket::with_ttl` — custom TTL respected by `clear_all`

use civ_economy::{
    drain_energy_budget, step, verify_ledger_conservation,
    EconomyState, LedgerEntry, LedgerInvariantError,
    ACCOUNT_CONSUMPTION, ACCOUNT_ENERGY_BUDGET,
    GoodId, MultiGoodMarket, Order, Side,
};

// ---------------------------------------------------------------------------
// 1. verify_ledger_conservation — UnbalancedEntry error variant
// ---------------------------------------------------------------------------

/// FR-CIV-TEST-006 §1 — verify_ledger_conservation must return
/// `LedgerInvariantError::UnbalancedEntry` when any ledger leg has debit != credit.
/// This error variant is not covered by the existing internal tests (which only
/// test the NegativeBudget and LedgerTooLarge variants externally).
#[test]
fn verify_ledger_conservation_rejects_unbalanced_entry() {
    let mut state = EconomyState::with_energy_budget(1_000);
    state.tick = 10; // enough headroom so LedgerTooLarge does not fire first

    // Manually push a leg where debit != credit (simulating a corrupted row).
    state.ledger.push(LedgerEntry {
        tick: 5,
        debit: 100,
        credit: 99, // off by one — unbalanced
        account: ACCOUNT_CONSUMPTION,
    });

    let result = verify_ledger_conservation(&state);
    assert_eq!(
        result,
        Err(LedgerInvariantError::UnbalancedEntry {
            index: 0,
            debit: 100,
            credit: 99,
        }),
        "unbalanced debit/credit must produce UnbalancedEntry error"
    );
}

// ---------------------------------------------------------------------------
// 2. step — no-change path (budget unchanged → no tick-close entry appended)
// ---------------------------------------------------------------------------

/// FR-CIV-TEST-006 §2 — when `drain_energy_budget` is NOT called between
/// two consecutive `step` calls, `step` must NOT append a tick-close entry
/// (the budget did not move, so the close entry would have amount = 0).
/// The existing internal test only exercises the path where budget DID change.
#[test]
fn step_does_not_append_tick_close_when_budget_unchanged() {
    let mut state = EconomyState::with_energy_budget(500);

    // First step — no drain before it, so no tick-close entry.
    step(&mut state);
    assert_eq!(state.tick, 1);
    assert_eq!(
        state.ledger.len(),
        0,
        "step with no budget change must not append any ledger entry"
    );

    // Second step — still no drain between steps.
    step(&mut state);
    assert_eq!(state.tick, 2);
    assert_eq!(
        state.ledger.len(),
        0,
        "consecutive steps without drain must produce zero ledger entries"
    );

    // Conservation holds on empty ledger.
    verify_ledger_conservation(&state).expect("empty ledger must pass conservation");
}

// ---------------------------------------------------------------------------
// 3. MultiGoodMarket::place_order — push a pre-built Order onto bid/ask side
// ---------------------------------------------------------------------------

/// FR-CIV-TEST-006 §3 — `place_order` with `Side::Bid` and `Side::Ask` must
/// produce identical cleared trades to calling `place_bid` / `place_ask` with
/// the same parameters. This verifies the dispatch branch and the `Order`
/// struct passthrough.
#[test]
fn multigood_place_order_matches_place_bid_and_place_ask() {
    let grain = GoodId(1);

    // Market A: built via place_bid / place_ask.
    let mut market_a = MultiGoodMarket::new();
    market_a.place_bid(grain, 1, 5, 100, 0);
    market_a.place_ask(grain, 2, 5, 80, 0);

    // Market B: built via place_order with pre-built Order structs.
    let mut market_b = MultiGoodMarket::new();
    market_b.place_order(
        grain,
        Side::Bid,
        Order {
            agent_id: 1,
            qty: 5,
            price_cents: 100,
            placed_tick: 0,
        },
    );
    market_b.place_order(
        grain,
        Side::Ask,
        Order {
            agent_id: 2,
            qty: 5,
            price_cents: 80,
            placed_tick: 0,
        },
    );

    let trades_a = market_a.clear_all(0);
    let trades_b = market_b.clear_all(0);

    assert_eq!(
        trades_a, trades_b,
        "place_order must produce the same trades as place_bid/place_ask"
    );
    assert_eq!(trades_b.len(), 1);
    assert_eq!(trades_b[0].qty, 5);
    assert_eq!(trades_b[0].price_cents, 90); // (100+80)/2
}

// ---------------------------------------------------------------------------
// 4. MultiGoodMarket::with_ttl — custom TTL > 1 keeps orders alive longer
// ---------------------------------------------------------------------------

/// FR-CIV-TEST-006 §4 — `MultiGoodMarket::with_ttl(3)` must keep an uncleared
/// order alive for 3 ticks after placement (placed_tick + ttl >= current_tick).
/// An order placed at tick 0 with TTL 3 should still be present at tick 3
/// (0+3 >= 3) but gone by tick 4 (0+3 < 4).
#[test]
fn multigood_with_ttl_keeps_orders_alive_for_custom_ttl() {
    let iron = GoodId(7);
    let mut market = MultiGoodMarket::with_ttl(3);

    // A bid that cannot cross (no matching ask) — stays on the book.
    market.place_bid(iron, 1, 10, 50, 0);

    // Tick 1: order still in TTL window (0 + 3 >= 1) — no trades but order present.
    let t1 = market.clear_all(1);
    assert!(t1.is_empty());
    assert_eq!(
        market.books[&iron].bids.len(),
        1,
        "order must survive tick 1 with TTL=3"
    );

    // Tick 3: still in window (0 + 3 >= 3).
    let t3 = market.clear_all(3);
    assert!(t3.is_empty());
    assert_eq!(
        market.books[&iron].bids.len(),
        1,
        "order must survive tick 3 with TTL=3"
    );

    // Tick 4: expired (0 + 3 < 4) — order must be dropped.
    let t4 = market.clear_all(4);
    assert!(t4.is_empty());
    assert_eq!(
        market.books[&iron].bids.len(),
        0,
        "order must be dropped by tick 4 with TTL=3 (0+3 < 4)"
    );
}