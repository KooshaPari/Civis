//! External regression tests for civ-economy correctness bugs found during the
//! 2026-09-12 scorecard re-audit (Economy/Resources 35% lane).
//!
//! Each test below documents a real defect, asserts the correct behavior, and
//! was red before the matching source fix landed in the same commit.

use civ_economy::{
    apply_trade, propose_trade, ProductionProfile, Stocks, TradeOffer,
};

// ---------------------------------------------------------------------------
// Gap P2: `apply_trade` panics when stock changes between propose and apply
// ---------------------------------------------------------------------------

/// Construct two stock-holders A and B where A offers Food to B in exchange
/// for Wood, then drain A's Food stock to zero *between* `propose_trade` and
/// `apply_trade`. The conservation assertion inside `apply_trade` must not
/// fire — the actual transferred quantity (clamped to A's available stock)
/// must be the same on both sides.
///
/// Before the fix this test panics in `debug_assert_eq!` at line 285:
///   `removed_a.abs() == added_b.abs()` fails because `removed_a = 0` (stock
///   already gone) but `added_b = qty_a_to_b` (the offer's original quantity).
#[test]
fn apply_trade_handles_stock_drained_between_propose_and_apply() {
    // A is a farmer with Food surplus and Wood deficit.
    let mut a = Stocks::default();
    a.add(civ_economy::Good::Food, 100);
    // B is a woodcutter with Wood surplus and Food deficit.
    let mut b = Stocks::default();
    b.add(civ_economy::Good::Wood, 50);

    // Farmer profile: +Food/-Wood.  Woodcutter profile: +Wood/-Food.
    let farmer = ProductionProfile::new(
        [10, 0, 0, 0, 0], // production: Food
        [0, 0, 5, 0, 0], // consumption: Wood
    );
    let woodcutter = ProductionProfile::new(
        [0, 0, 10, 0, 0], // production: Wood
        [5, 0, 0, 0, 0],  // consumption: Food
    );

    let offer: TradeOffer = propose_trade(&a, &farmer, &b, &woodcutter)
        .expect("mutually beneficial trade should be proposed");

    // Simulate the offer going stale: drain A's Food stock between propose
    // and apply (e.g. another actor consumed it).
    a.add(civ_economy::Good::Food, -a.get(civ_economy::Good::Food));

    // Snapshot the totals before apply so we can verify conservation.
    let total_before = a.total().saturating_add(b.total());

    apply_trade(&mut a, &mut b, &offer);

    let total_after = a.total().saturating_add(b.total());
    assert_eq!(
        total_before, total_after,
        "combined stock total must be conserved across apply_trade"
    );

    // The direction of the trade must still be respected even if quantities
    // were clamped to zero: A's Food is unchanged (clamped), B's Food may have
    // moved if B actually had surplus Wood that A wanted.
    assert_eq!(
        a.get(civ_economy::Good::Food),
        0,
        "A's Food must be 0 after the stale trade"
    );
}

/// A simpler variant: if both sides have stock but at lower quantity than the
/// offer claims, `apply_trade` must transfer exactly what each side actually
/// has, and the conservation invariant must hold without panicking.
#[test]
fn apply_trade_clamps_each_side_to_actual_stock() {
    let mut a = Stocks::default();
    a.add(civ_economy::Good::Food, 3); // less than offer claims
    let mut b = Stocks::default();
    b.add(civ_economy::Good::Wood, 50);

    let offer = TradeOffer {
        good_a_to_b: civ_economy::Good::Food,
        qty_a_to_b: 10, // offer claims more than A has
        good_b_to_a: civ_economy::Good::Wood,
        qty_b_to_a: 5,
    };

    let total_before = a.total().saturating_add(b.total());

    apply_trade(&mut a, &mut b, &offer);

    // Conservation invariant: combined stock total is unchanged.
    let total_after = a.total().saturating_add(b.total());
    assert_eq!(
        total_before, total_after,
        "apply_trade must not create or destroy stock"
    );

    // The A→B leg is clamped to A's actual Food (3), not the offer's 10.
    assert_eq!(a.get(civ_economy::Good::Food), 0);
    assert_eq!(
        b.get(civ_economy::Good::Food),
        3,
        "B must receive exactly what A actually gave (3), not the offer's claimed 10"
    );

    // The B→A leg runs at the offer's qty because B has enough Wood (50 >= 5).
    assert_eq!(
        a.get(civ_economy::Good::Wood),
        5,
        "A must receive B's full offered Wood (5)"
    );
    assert_eq!(b.get(civ_economy::Good::Wood), 45);
}
