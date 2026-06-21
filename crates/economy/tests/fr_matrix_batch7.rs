//! FR-matrix batch 7 — integration tests for `civ-economy` IMPL-NO-TEST rows.
//!
//! Each test function name keeps a stable `green_` prefix and contains the
//! FR identifier in a machine-readable form. The comments below preserve the
//! exact matrix ID for scanner and audit mapping.
//!
//! Covered rows:
//! - FR-CIV-LIFE-020
//! - FR-ECON-001
//! - FR-ECON-002
//! - FR-ECON-003

use civ_economy::{
    apply_trade, drain_energy_budget, propose_trade, step, step_stocks, verify_ledger_conservation,
    Allocator, Bid, EconomyState, Good, LedgerEntry, LedgerInvariantError, LedgerSide, Offer,
    ProductionProfile as Profile, Stocks, ACCOUNT_CONSUMPTION, ACCOUNT_ENERGY_BUDGET,
    INSTITUTION_MARKET, INSTITUTION_TREASURY,
};

use civ_economy::step_institutions;

fn funded_economy() -> EconomyState {
    let mut state = EconomyState::with_energy_budget(10_000);
    step_institutions(&mut state);
    let mut institutions = state.institutions.clone();
    institutions
        .post(
            &mut state,
            LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
            LedgerSide::Institution(INSTITUTION_TREASURY),
            3_000,
        )
        .expect("fund treasury");
    institutions
        .post(
            &mut state,
            LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
            LedgerSide::Institution(INSTITUTION_MARKET),
            3_000,
        )
        .expect("fund market");
    state.institutions = institutions;
    state
}

// ---------------------------------------------------------------------------
// FR-CIV-LIFE-020
// ---------------------------------------------------------------------------

/// FR-CIV-LIFE-020 — stocks clamp withdrawals at zero and maintain a bounded
/// aggregate total.
#[test]
fn green_fr_civ_life_020_stocks_add_clamps_withdrawals_and_total() {
    let mut stocks = Stocks::default();
    assert_eq!(stocks.add(Good::Food, 8), 8);
    assert_eq!(stocks.add(Good::Food, -3), -3);
    assert_eq!(stocks.add(Good::Food, -999), -5);
    assert_eq!(stocks.get(Good::Food), 0);
    assert_eq!(stocks.total(), 0);
}

/// FR-CIV-LIFE-020 — per-good production and consumption are applied on a
/// per-tick basis; deficits clamp at zero for that good.
#[test]
fn green_fr_civ_life_020_step_stocks_applies_profile_with_deficit_clamp() {
    let mut stocks = Stocks::default();
    stocks.add(Good::Food, 5);
    stocks.add(Good::Water, 1);

    let profile = Profile::new([10, 0, 0, 0, 0], [4, 2, 0, 0, 0]);
    step_stocks(&mut stocks, &profile);

    assert_eq!(stocks.get(Good::Food), 11);
    assert_eq!(stocks.get(Good::Water), 0);
    assert_eq!(stocks.total(), 11);
}

/// FR-CIV-LIFE-020 — trade proposal should identify only mutually beneficial,
/// positive transfers.
#[test]
fn green_fr_civ_life_020_propose_trade_selects_mutual_exchange() {
    let a_stocks = {
        let mut s = Stocks::default();
        s.add(Good::Food, 8);
        s.add(Good::Water, 0);
        s
    };
    let b_stocks = {
        let mut s = Stocks::default();
        s.add(Good::Food, 0);
        s.add(Good::Water, 6);
        s
    };

    let a_profile = Profile::new([4, 0, 0, 0, 0], [0, 1, 0, 0, 0]);
    let b_profile = Profile::new([0, 3, 0, 0, 0], [1, 0, 0, 0, 0]);

    let offer = propose_trade(&a_stocks, &a_profile, &b_stocks, &b_profile)
        .expect("trade should be mutually beneficial");

    assert_eq!(offer.good_a_to_b, Good::Food);
    assert_eq!(offer.good_b_to_a, Good::Water);
    assert_eq!(offer.qty_a_to_b, 1);
    assert_eq!(offer.qty_b_to_a, 1);
}

/// FR-CIV-LIFE-020 — applying a proposed trade conserves total stock volume.
#[test]
fn green_fr_civ_life_020_apply_trade_preserves_total_stock() {
    let mut a_stocks = {
        let mut s = Stocks::default();
        s.add(Good::Food, 8);
        s.add(Good::Water, 0);
        s
    };
    let mut b_stocks = {
        let mut s = Stocks::default();
        s.add(Good::Food, 0);
        s.add(Good::Water, 6);
        s
    };
    let a_profile = Profile::new([4, 0, 0, 0, 0], [0, 1, 0, 0, 0]);
    let b_profile = Profile::new([0, 3, 0, 0, 0], [1, 0, 0, 0, 0]);

    let before_total = a_stocks.total() + b_stocks.total();
    let offer = propose_trade(&a_stocks, &a_profile, &b_stocks, &b_profile)
        .expect("trade should be mutually beneficial");
    apply_trade(&mut a_stocks, &mut b_stocks, &offer);

    assert_eq!(a_stocks.total() + b_stocks.total(), before_total);
    assert_eq!(a_stocks.get(Good::Food), 7);
    assert_eq!(a_stocks.get(Good::Water), 1);
}

// ---------------------------------------------------------------------------
// FR-ECON-001
// ---------------------------------------------------------------------------

/// FR-ECON-001 — positive consumption drains macro budget and emits a matching
/// ledger entry (balanced debit/credit).
#[test]
fn green_fr_econ_001_drain_energy_budget_records_balanced_ledger() {
    let mut state = EconomyState::with_energy_budget(100);
    drain_energy_budget(&mut state, 37);

    assert_eq!(state.energy_budget_joules, 63);
    assert_eq!(state.ledger.len(), 1);
    let entry = &state.ledger[0];
    assert_eq!(entry.account, ACCOUNT_CONSUMPTION);
    assert_eq!(entry.debit, 37);
    assert_eq!(entry.credit, 37);
    assert_eq!(entry.tick, 0);
}

/// FR-ECON-001 — negative or zero consumption is a no-op and does not mutate
/// budget or create ledger noise.
#[test]
fn green_fr_econ_001_drain_energy_budget_non_positive_is_noop() {
    let mut state = EconomyState::with_energy_budget(50);
    state.ledger.push(LedgerEntry {
        tick: 3,
        debit: 7,
        credit: 7,
        account: ACCOUNT_CONSUMPTION,
    });
    let ledger_before = state.ledger.clone();

    drain_energy_budget(&mut state, -1);
    drain_energy_budget(&mut state, 0);

    assert_eq!(state.energy_budget_joules, 50);
    assert_eq!(state.ledger, ledger_before);
}

/// FR-ECON-001 — ledger conservation rejects unbalanced debit/credit legs.
#[test]
fn green_fr_econ_001_verify_ledger_conservation_rejects_unbalanced_entry() {
    let mut state = EconomyState::with_energy_budget(50);
    state.tick = 1;
    state.ledger = vec![LedgerEntry {
        tick: 0,
        debit: 5,
        credit: 4,
        account: ACCOUNT_CONSUMPTION,
    }];

    assert_eq!(
        verify_ledger_conservation(&state),
        Err(LedgerInvariantError::UnbalancedEntry {
            index: 0,
            debit: 5,
            credit: 4,
        })
    );
}

/// FR-ECON-001 — after budget changes in a tick, `step` posts a balancing
/// closing entry and advances the economy clock by one.
#[test]
fn green_fr_econ_001_step_posts_budget_delta_entry() {
    let mut state = EconomyState::with_energy_budget(200);
    drain_energy_budget(&mut state, 80);
    step(&mut state);

    assert_eq!(state.tick, 1);
    assert_eq!(state.ledger.len(), 2);
    assert_eq!(
        state.ledger.last().expect("closing entry").account,
        ACCOUNT_ENERGY_BUDGET
    );
    verify_ledger_conservation(&state).expect("ledger conservation after economy step");
}

// ---------------------------------------------------------------------------
// FR-ECON-002
// ---------------------------------------------------------------------------

/// FR-ECON-002 — clearing crossing orders produces balanced institution
/// transfers and does not mutate the macro budget.
#[test]
fn green_fr_econ_002_clear_crossing_orders_posts_balanced_transfer() {
    let mut economy = funded_economy();
    let mut allocator = Allocator::new();
    let mut ledger = economy.institutions.clone();

    allocator
        .post_bid(Bid {
            id: 0,
            bidder: INSTITUTION_TREASURY,
            good: "food".to_string(),
            quantity: 5,
            price: 120,
        })
        .expect("post bid");
    allocator
        .post_offer(Offer {
            id: 0,
            offerer: INSTITUTION_MARKET,
            good: "food".to_string(),
            quantity: 4,
            price: 80,
        })
        .expect("post offer");

    let trades = allocator.clear(&mut economy, &mut ledger);
    assert_eq!(trades.len(), 1);
    assert!(!trades[0].rationed);
    assert_eq!(trades[0].quantity, 4);
    assert_eq!(trades[0].price, 100);
    assert_eq!(trades[0].bidder, INSTITUTION_TREASURY);
    assert_eq!(trades[0].offerer, INSTITUTION_MARKET);
    assert_eq!(ledger.institution_balance(INSTITUTION_TREASURY), 2_600);
    assert_eq!(ledger.institution_balance(INSTITUTION_MARKET), 3_400);
    assert_eq!(economy.energy_budget_joules, 4_000);
    ledger
        .verify_conservation()
        .expect("institution conservation");
}

/// FR-ECON-002 — when no bids cross, remaining demand is rationed over supply and
/// prices set to the unmatched midpoint.
#[test]
fn green_fr_econ_002_clear_rationing_does_not_exceed_supply() {
    let mut economy = funded_economy();
    let mut allocator = Allocator::new();
    let mut ledger = economy.institutions.clone();

    allocator
        .post_bid(Bid {
            id: 0,
            bidder: INSTITUTION_TREASURY,
            good: "water".to_string(),
            quantity: 20,
            price: 50,
        })
        .expect("post ration bid");
    allocator
        .post_offer(Offer {
            id: 0,
            offerer: INSTITUTION_MARKET,
            good: "water".to_string(),
            quantity: 7,
            price: 100,
        })
        .expect("post ration offer");

    let trades = allocator.clear(&mut economy, &mut ledger);
    assert_eq!(trades.len(), 1);
    let trade = &trades[0];
    assert!(trade.rationed);
    assert_eq!(trade.quantity, 7);
    assert_eq!(trade.price, 75);
    assert_eq!(trade.bidder, INSTITUTION_TREASURY);
    assert_eq!(trade.offerer, INSTITUTION_MARKET);
    assert_eq!(allocator.bid_count(), 1);
    assert_eq!(allocator.offer_count(), 0);
    ledger
        .verify_conservation()
        .expect("institution conservation");
}

// ---------------------------------------------------------------------------
// FR-ECON-003
// ---------------------------------------------------------------------------

/// FR-ECON-003 — allocator IDs are monotonic and cancellations remove live orders
/// by id.
#[test]
fn green_fr_econ_003_allocator_cancels_orders() {
    let mut allocator = Allocator::new();

    let bid_id = allocator
        .post_bid(Bid {
            id: 0,
            bidder: INSTITUTION_TREASURY,
            good: "food".to_string(),
            quantity: 10,
            price: 100,
        })
        .expect("post bid");
    let offer_id = allocator
        .post_offer(Offer {
            id: 0,
            offerer: INSTITUTION_MARKET,
            good: "water".to_string(),
            quantity: 4,
            price: 80,
        })
        .expect("post offer");

    assert_eq!(allocator.bid_count(), 1);
    assert_eq!(allocator.offer_count(), 1);
    assert_eq!(bid_id, 0);
    assert_eq!(offer_id, 1);

    let cancelled = allocator.cancel(offer_id);
    assert!(matches!(
        cancelled,
        Some(civ_economy::CancelledOrder::Offer(_))
    ));
    assert_eq!(allocator.offer_count(), 0);

    let remaining = allocator.cancel(999);
    assert!(remaining.is_none());
}
