//! Real behavioural oracles for the economy FR cluster:
//! `FR-CIV-ECON-003`, `FR-ECON-006`, `FR-ECON-009`, `FR-ECON-010`.
//!
//! Replaces the auto-generated placeholders
//! (`crates/economy/tests/fr_fr_civ_econ_003.rs`, `fr_fr_econ_006.rs`,
//! `fr_fr_econ_009.rs`, `fr_fr_econ_010.rs`) whose whole body was
//! `assert_eq!(SCHEMA_VERSION, 1); let _ = EconomyState::default(); let _ =
//! ResourceType::Food;` — byte-identical across four unrelated IDs, asserting
//! shared-type existence rather than any requirement.
//!
//! Requirement text (source of truth):
//!
//! - `FR-CIV-ECON-003` — "Joule economy allocator" (`docs/reference/FR_TRACKER.md:9`,
//!   implementing code `crates/economy/src/allocation.rs` + `allocator.rs`).
//!   The allocator hands out joules from a finite budget: it must never grant
//!   more than the budget holds, life-supporting (subsistence) tiers are filled
//!   before discretionary ones, allocation is deterministic, and every cleared
//!   auction trade transfers joules between institutions without creating or
//!   destroying any.
//! - `FR-ECON-006` — "GDP SHALL be derived from sum of regional Joule throughput
//!   converted at a fixed exchange rate." (`crates/economy/src/gdp.rs`).
//! - `FR-ECON-009` — "Subsistence mode SHALL activate when a civilization's total
//!   Joule balance drops below threshold." (`crates/economy/src/subsistence.rs`).
//! - `FR-ECON-010` — "Treasury balance SHALL be tracked in MilliCredits (`i64`)
//!   with no floating-point accumulation." (`crates/economy/src/treasury.rs`).
//!
//! Every assertion below is a conservation law, an exact value, a boundary, or a
//! determinism property: each one fails if the behaviour it covers regresses.

use civ_economy::{
    allocate_by_priority, allocate_with, compute_gdp, drain_energy_budget,
    verify_ledger_conservation, AllocationEngine, AllocationRegime, Allocator, Bid, EconomyState,
    InstitutionLedger, JouleAllocator, LedgerSide, Offer, PriorityTier, RegionGdp, SubsistenceMode,
    Treasury, ACCOUNT_ENERGY_BUDGET, DEFAULT_SUBSISTENCE_THRESHOLD, INSTITUTION_MARKET,
    INSTITUTION_TREASURY,
};

/// Deterministic LCG — keeps the "random" sequences reproducible so a failure is
/// replayable (no `rand` dependency, no wall-clock seeding).
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

/// Fund an institution from the macro joule budget.
fn fund(
    economy: &mut EconomyState,
    ledger: &mut InstitutionLedger,
    institution: civ_economy::InstitutionId,
    joules: i64,
) {
    ledger
        .post(
            economy,
            LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
            LedgerSide::Institution(institution),
            joules,
        )
        .expect("macro budget must cover the funding post");
}

/// Total joules held across all institution accounts.
fn institution_total(ledger: &InstitutionLedger) -> i64 {
    ledger
        .accounts
        .values()
        .map(|account| account.balance_joules)
        .sum()
}

// ===========================================================================
// FR-CIV-ECON-003 — joule economy allocator
// ===========================================================================

/// The joule allocator can never hand out more joules than the budget holds, and
/// it meets demand in full whenever the budget allows it. This is the joule
/// conservation law for a single consumer.
/// Covers FR-CIV-ECON-003.
#[test]
fn joule_allocator_never_grants_more_joules_than_the_budget_holds() {
    for budget in [0i64, 1, 37, 100, 999, 1_000_000] {
        for demand in [0i64, 1, 37, 100, 999, 1_000_000] {
            let granted = JouleAllocator.allocate(budget, demand);
            assert!(
                granted >= 0,
                "allocated joules must be non-negative: budget={budget} demand={demand} granted={granted}"
            );
            assert!(
                granted <= budget.min(demand),
                "allocated joules must not exceed min(budget, demand): budget={budget} demand={demand} granted={granted}"
            );
            if budget >= demand {
                assert_eq!(
                    granted, demand,
                    "a sufficient budget must satisfy the demand in full"
                );
            } else {
                assert_eq!(
                    granted, budget,
                    "a scarce budget binds: the joule ceiling is the grant"
                );
            }
        }
    }
}

/// Joules are a conserved quantity across consumers too: the priority allocator
/// distributes exactly the budget, filling life-supporting tiers before
/// discretionary ones.
/// Covers FR-CIV-ECON-003.
#[test]
fn joule_priority_allocation_conserves_the_budget_and_fills_subsistence_first() {
    let demands = [
        (PriorityTier::Luxury, 300),
        (PriorityTier::Basic, 200),
        (PriorityTier::Subsistence, 500),
    ];
    let budget = 600;

    let granted = allocate_by_priority(&JouleAllocator, budget, &demands);

    assert_eq!(granted.len(), 3, "one grant per demand, in input order");
    assert_eq!(granted[2], 500, "subsistence joules are filled first");
    assert_eq!(granted[1], 100, "the basic tier receives the remainder");
    assert_eq!(granted[0], 0, "luxury is starved while higher tiers are unmet");
    assert_eq!(
        granted.iter().sum::<i64>(),
        budget,
        "every available joule is allocated exactly once (no joule created or lost)"
    );

    // An abundant budget satisfies every tier and still creates nothing.
    let abundant = allocate_by_priority(&JouleAllocator, 5_000, &demands);
    assert_eq!(abundant, vec![300, 200, 500]);
    assert_eq!(abundant.iter().sum::<i64>(), 1_000, "sum equals total demand");
}

/// The `Joule` regime is the single dispatch point callers use; it must route to
/// the joule allocator's curve for every input.
/// Covers FR-CIV-ECON-003.
#[test]
fn joule_regime_dispatch_matches_the_joule_allocator() {
    for budget in [0i64, 5, 60, 400] {
        for demand in [0i64, 5, 60, 400] {
            assert_eq!(
                allocate_with(AllocationRegime::Joule, budget, demand),
                JouleAllocator.allocate(budget, demand),
                "regime dispatch drifted from JouleAllocator for budget={budget} demand={demand}"
            );
        }
    }
    assert_eq!(allocate_with(AllocationRegime::Joule, 250, 1_000), 250);
    assert_eq!(allocate_with(AllocationRegime::Joule, 1_000, 250), 250);
}

/// Identical inputs must produce identical joule splits, tick after tick: a
/// replay that observes a different allocation is a determinism bug.
/// Covers FR-CIV-ECON-003.
#[test]
fn joule_allocation_is_deterministic_across_repeated_runs() {
    let demands = [
        (PriorityTier::Comfort, 7),
        (PriorityTier::Subsistence, 11),
        (PriorityTier::Luxury, 13),
        (PriorityTier::Basic, 17),
    ];
    let first = allocate_by_priority(&JouleAllocator, 23, &demands);

    // Subsistence 11 filled fully, Basic takes 12 of 17, nothing left below.
    assert_eq!(first, vec![0, 11, 0, 12]);
    assert_eq!(first.iter().sum::<i64>(), 23, "the budget is fully consumed");

    for run in 0..64 {
        assert_eq!(
            allocate_by_priority(&JouleAllocator, 23, &demands),
            first,
            "run {run} produced a different joule split"
        );
    }
}

/// No budget and no demand mean no allocation; negative inputs are clamped to
/// zero rather than inverted into a grant.
/// Covers FR-CIV-ECON-003.
#[test]
fn joule_allocator_grants_nothing_without_budget_or_demand() {
    assert_eq!(JouleAllocator.allocate(0, 500), 0);
    assert_eq!(JouleAllocator.allocate(-1, 500), 0);
    assert_eq!(JouleAllocator.allocate(500, 0), 0);
    assert_eq!(JouleAllocator.allocate(500, -1), 0);
    assert_eq!(
        allocate_by_priority(&JouleAllocator, 0, &[(PriorityTier::Subsistence, 500)]),
        vec![0]
    );
    assert_eq!(
        allocate_by_priority(&JouleAllocator, -100, &[(PriorityTier::Subsistence, 500)]),
        vec![0]
    );
}

/// The macro joule budget is a floor at zero: consumption larger than the budget
/// drains it to exactly zero and leaves the ledger conserved.
/// Covers FR-CIV-ECON-003.
#[test]
fn macro_joule_budget_is_never_driven_negative_by_consumption() {
    let mut economy = EconomyState::with_energy_budget(100);
    drain_energy_budget(&mut economy, 250);
    assert_eq!(
        economy.energy_budget_joules, 0,
        "consumption exceeding the budget must clamp at zero, not go negative"
    );
    assert_eq!(economy.ledger.len(), 1, "the applied drain is recorded once");
    assert_eq!(economy.ledger[0].debit, 100, "only the available joules are booked");

    let mut economy = EconomyState::with_energy_budget(100);
    drain_energy_budget(&mut economy, 40);
    drain_energy_budget(&mut economy, -5);
    assert_eq!(
        economy.energy_budget_joules, 60,
        "a non-positive consumption request is a no-op"
    );
    assert_eq!(economy.ledger.len(), 1);

    // Tick advance closes the ledger for this tick: the per-tick posting bound
    // then holds, and conservation verifies.
    civ_economy::step(&mut economy);
    verify_ledger_conservation(&economy).expect("conservation after drain + step");
}

/// Clearing an auction transfers joules between institutions. The total joules
/// held across institutions must be unchanged by clearing, and the macro budget
/// must not be touched by trades.
/// Covers FR-CIV-ECON-003.
#[test]
fn auction_clearing_conserves_institution_joules() {
    let mut economy = EconomyState::with_energy_budget(4_000);
    let mut ledger = InstitutionLedger::with_defaults();
    fund(&mut economy, &mut ledger, INSTITUTION_TREASURY, 2_000);
    fund(&mut economy, &mut ledger, INSTITUTION_MARKET, 2_000);
    assert_eq!(economy.energy_budget_joules, 0, "both accounts are funded");
    let total_before = institution_total(&ledger);
    assert_eq!(total_before, 4_000);

    let mut allocator = Allocator::new();
    allocator
        .post_bid(Bid {
            id: 0,
            bidder: INSTITUTION_TREASURY,
            good: "food".to_string(),
            quantity: 5,
            price: 120,
        })
        .expect("valid bid");
    allocator
        .post_offer(Offer {
            id: 0,
            offerer: INSTITUTION_MARKET,
            good: "food".to_string(),
            quantity: 4,
            price: 80,
        })
        .expect("valid offer");

    let trades = allocator.clear(&mut economy, &mut ledger);

    assert_eq!(trades.len(), 1, "the crossing pair clears exactly once");
    let trade = &trades[0];
    assert_eq!(trade.good, "food");
    assert_eq!(trade.quantity, 4, "supply is the binding side");
    assert_eq!(trade.price, 100, "clearing price is the midpoint of 120 and 80");
    assert!(
        !trade.rationed,
        "a crossing book clears by trade, not by rationing"
    );
    assert_eq!(trade.bidder, INSTITUTION_TREASURY);
    assert_eq!(trade.offerer, INSTITUTION_MARKET);

    // 4 units at 100 cents = 400 joules paid by the bidder to the offerer.
    assert_eq!(ledger.institution_balance(INSTITUTION_TREASURY), 1_600);
    assert_eq!(ledger.institution_balance(INSTITUTION_MARKET), 2_400);
    assert_eq!(
        institution_total(&ledger),
        total_before,
        "clearing must not create or destroy institution joules"
    );
    assert_eq!(
        economy.energy_budget_joules, 0,
        "trades settle between institutions and never draw on the macro budget"
    );
    ledger.verify_conservation().expect("ledger stays balanced");

    // 5 bid for, 4 filled: the unfilled unit survives to the next tick.
    assert_eq!(allocator.bid_count(), 1);
    assert_eq!(allocator.offer_count(), 0);
    assert!(
        allocator.clear(&mut economy, &mut ledger).is_empty(),
        "with no supply on the book nothing can clear"
    );
    assert_eq!(allocator.bid_count(), 1, "the live bid is untouched");
    assert_eq!(
        institution_total(&ledger),
        total_before,
        "a no-op tick cannot move joules either"
    );
}

/// When the book does not cross, the allocator falls back to rationing but still
/// cannot clear more than the supplied quantity, and the price signal is the
/// midpoint of the best unmatched bid and ask.
/// Covers FR-CIV-ECON-003.
#[test]
fn joule_rationing_never_exceeds_the_supply_on_the_book() {
    let mut economy = EconomyState::with_energy_budget(4_000);
    let mut ledger = InstitutionLedger::with_defaults();
    fund(&mut economy, &mut ledger, INSTITUTION_TREASURY, 2_000);
    fund(&mut economy, &mut ledger, INSTITUTION_MARKET, 2_000);
    let total_before = institution_total(&ledger);

    let mut allocator = Allocator::new();
    allocator
        .post_bid(Bid {
            id: 0,
            bidder: INSTITUTION_TREASURY,
            good: "food".to_string(),
            quantity: 10,
            price: 50,
        })
        .expect("valid bid");
    allocator
        .post_offer(Offer {
            id: 0,
            offerer: INSTITUTION_MARKET,
            good: "food".to_string(),
            quantity: 4,
            price: 100,
        })
        .expect("valid offer");

    let trades = allocator.clear(&mut economy, &mut ledger);

    assert_eq!(trades.len(), 1);
    let trade = &trades[0];
    assert!(trade.rationed, "50 < 100, so the book does not cross");
    assert_eq!(
        trade.quantity, 4,
        "rationing is capped by the supplied quantity"
    );
    assert_eq!(trade.price, 75, "price signal is (best bid + best ask) / 2");
    assert!(trade.price >= 0, "a price signal is never negative");

    assert_eq!(ledger.institution_balance(INSTITUTION_TREASURY), 2_000 - 300);
    assert_eq!(ledger.institution_balance(INSTITUTION_MARKET), 2_000 + 300);
    assert_eq!(
        institution_total(&ledger),
        total_before,
        "rationed clearing conserves institution joules too"
    );
    ledger.verify_conservation().expect("ledger stays balanced");

    // 10 demanded, 4 rationed: 6 units of demand survive, supply is exhausted.
    assert_eq!(allocator.bid_count(), 1);
    assert_eq!(allocator.offer_count(), 0);
}

/// Guards the settlement-failure path in `Allocator::clear` (found by this
/// oracle pass — see the report accompanying this commit).
///
/// TODO(FR-CIV-ECON-003): observed vs expected. When the bidder's institution
/// cannot cover the transfer, `clear` records NO trade and posts NO transfer,
/// yet it still decrements both sides of the order book to zero, after which the
/// `retain` sweep drops both orders. The matched goods are destroyed with no
/// joule movement: the offerer loses supply without being paid, and the bidder
/// receives nothing. Expected behaviour, per the module contract ("Every cleared
/// trade posts a balanced institution transfer"), is that a match which cannot
/// settle leaves both orders live for the next tick — or is rejected without
/// consuming quantities. The assertions below pin the CURRENT (defective)
/// behaviour so the defect stays visible in the test suite; flip `bid_count`
/// and `offer_count` to 1 and this test becomes the fix's regression oracle.
/// Covers FR-CIV-ECON-003.
#[test]
fn auction_clearing_consumes_the_book_when_settlement_cannot_be_funded() {
    let mut economy = EconomyState::with_energy_budget(200);
    let mut ledger = InstitutionLedger::with_defaults();
    fund(&mut economy, &mut ledger, INSTITUTION_TREASURY, 100);
    let total_before = institution_total(&ledger);
    assert_eq!(total_before, 100);

    let mut allocator = Allocator::new();
    allocator
        .post_bid(Bid {
            id: 0,
            bidder: INSTITUTION_TREASURY,
            good: "food".to_string(),
            quantity: 5,
            price: 120,
        })
        .expect("valid bid");
    allocator
        .post_offer(Offer {
            id: 0,
            offerer: INSTITUTION_MARKET,
            good: "food".to_string(),
            quantity: 5,
            price: 80,
        })
        .expect("valid offer");

    // The crossing match would settle 5 units at the 100-cent midpoint = 500
    // joules, but the treasury holds only 100.
    let trades = allocator.clear(&mut economy, &mut ledger);

    assert!(
        trades.is_empty(),
        "an un-fundable match must not be recorded as a cleared trade"
    );
    assert_eq!(
        ledger.institution_balance(INSTITUTION_TREASURY),
        100,
        "no payment may be taken from an underfunded bidder"
    );
    assert_eq!(
        ledger.institution_balance(INSTITUTION_MARKET),
        0,
        "the offerer receives nothing when settlement fails"
    );
    assert_eq!(
        institution_total(&ledger),
        total_before,
        "institution joules are unchanged by a failed settlement"
    );
    ledger
        .verify_conservation()
        .expect("no posting was made, so nothing can be unbalanced");

    assert_eq!(
        allocator.bid_count(),
        0,
        "BUG: the bid is dropped although no settlement occurred"
    );
    assert_eq!(
        allocator.offer_count(),
        0,
        "BUG: the offer is dropped although it was never paid for"
    );
    assert!(
        allocator.clear(&mut economy, &mut ledger).is_empty(),
        "the drained book cannot produce a later trade"
    );
}

/// Guards the *absence* of the numeraire read-out that
/// `docs/design/civ-economy-emergent-markets.md` traces FR-CIV-ECON-003 to.
///
/// TODO(FR-CIV-ECON-003): the design doc says numeraire (the good with the
/// highest trade-frequency x acceptability) is a measured label written by
/// `Allocator::clear`. No such read-out exists in the crate today; the
/// allocator implements only the joule-clearing substrate. This guard fails the
/// moment numeraire field/read-out code lands in `allocator.rs`, at which point
/// it must be replaced by a real behavioural oracle for the emergence rule.
/// Covers FR-CIV-ECON-003.
#[test]
fn numeraire_emergence_readout_is_still_absent_from_the_allocator() {
    let allocator_source = include_str!("../src/allocator.rs").to_lowercase();
    assert!(
        !allocator_source.contains("numeraire"),
        "numeraire emergence appears to have landed in allocator.rs; \
         replace this absence guard with an oracle on the measured read-out"
    );
}

// ===========================================================================
// FR-CIV-ECON-003 (continued)
// ===========================================================================

/// A full engine-style tick loop — allocate from the live budget, drain the
/// granted joules, close the tick — must spend the finite joule budget exactly
/// once: it is never overspent, never stranded, and the ledger conservation
/// invariant holds at every tick boundary.
/// Covers FR-CIV-ECON-003.
#[test]
fn joule_budget_loop_spends_every_joule_and_stays_conserved_across_ticks() {
    let demand = 37i64;
    let mut economy = EconomyState::with_energy_budget(500);
    let mut spent = 0i64;

    for expected_tick in 1..=40u64 {
        let before = economy.energy_budget_joules;
        let granted = allocate_with(AllocationRegime::Joule, before, demand);
        assert!(
            granted <= before,
            "tick {expected_tick}: granted {granted} joules from a {before}-joule budget"
        );
        drain_energy_budget(&mut economy, granted);
        civ_economy::step(&mut economy);
        spent += granted;

        assert_eq!(
            economy.energy_budget_joules,
            before - granted,
            "tick {expected_tick}: the budget must fall by exactly the granted joules"
        );
        assert!(
            economy.energy_budget_joules >= 0,
            "tick {expected_tick}: the joule budget went negative"
        );
        assert_eq!(economy.tick, expected_tick, "one economy tick per iteration");
        verify_ledger_conservation(&economy).unwrap_or_else(|error| {
            panic!("tick {expected_tick}: conservation violated: {error:?}")
        });
    }

    assert_eq!(
        spent, 500,
        "the loop must spend the whole budget and invent no joules"
    );
    assert_eq!(
        economy.energy_budget_joules, 0,
        "the budget is exhausted exactly, not overdrawn"
    );
}

/// Replay determinism of the allocation substrate: two independent runs of the
/// same order-book sequence must produce the same trades (goods, quantities,
/// prices, rationing flag), the same institution balances, and the same
/// aggregate joules. A replay that diverges is a determinism bug.
/// Covers FR-CIV-ECON-003.
#[test]
fn auction_clearing_is_replay_deterministic_across_independent_runs() {
    // Returns (per-trade signature, (treasury, market, institution total)).
    fn run() -> (Vec<(String, i64, i64, bool)>, (i64, i64, i64)) {
        let mut economy = EconomyState::with_energy_budget(10_000);
        let mut ledger = InstitutionLedger::with_defaults();
        fund(&mut economy, &mut ledger, INSTITUTION_TREASURY, 3_000);
        fund(&mut economy, &mut ledger, INSTITUTION_MARKET, 3_000);

        let mut allocator = Allocator::new();
        // food: bid 90 < ask 100 -> rationing path.
        allocator
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "food".to_string(),
                quantity: 6,
                price: 90,
            })
            .expect("valid bid");
        allocator
            .post_offer(Offer {
                id: 0,
                offerer: INSTITUTION_MARKET,
                good: "food".to_string(),
                quantity: 2,
                price: 100,
            })
            .expect("valid offer");
        // tools: bid 200 >= ask 150 -> crossing path.
        allocator
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "tools".to_string(),
                quantity: 3,
                price: 200,
            })
            .expect("valid bid");
        allocator
            .post_offer(Offer {
                id: 0,
                offerer: INSTITUTION_MARKET,
                good: "tools".to_string(),
                quantity: 9,
                price: 150,
            })
            .expect("valid offer");

        let trades = allocator.clear(&mut economy, &mut ledger);
        let signature = trades
            .iter()
            .map(|trade| (trade.good.clone(), trade.quantity, trade.price, trade.rationed))
            .collect();
        let balances = (
            ledger.institution_balance(INSTITUTION_TREASURY),
            ledger.institution_balance(INSTITUTION_MARKET),
            institution_total(&ledger),
        );
        (signature, balances)
    }

    let (first_signature, first_balances) = run();
    assert_eq!(
        first_signature,
        vec![
            ("food".to_string(), 2, 95, true),
            ("tools".to_string(), 3, 175, false),
        ],
        "food is rationed at (90 + 100) / 2 and tools clear at (200 + 150) / 2"
    );
    // 2 * 95 + 3 * 175 = 190 + 525 = 715 joules moved from buyer to seller.
    assert_eq!(first_balances, (3_000 - 715, 3_000 + 715, 6_000));

    for run_index in 0..8 {
        let (signature, balances) = run();
        assert_eq!(
            signature, first_signature,
            "run {run_index} cleared to different trades from identical input"
        );
        assert_eq!(
            balances, first_balances,
            "run {run_index} moved different joules from identical input"
        );
    }
}

// ===========================================================================
// FR-ECON-006 — GDP from regional Joule throughput at a fixed exchange rate
// ===========================================================================

/// GDP is the sum of regional net joule throughput converted at the fixed
/// exchange rate (1.00 = 10_000 bp / 10_000 bp, i.e. one net joule = one GDP
/// unit). Input order and region ids are preserved.
/// Covers FR-ECON-006.
#[test]
fn gdp_sums_regional_net_joule_throughput_at_the_fixed_rate() {
    let regions = vec![
        RegionGdp {
            region_id: 0,
            production_joules: 1_000,
            waste_joules: 100,
        },
        RegionGdp {
            region_id: 7,
            production_joules: 500,
            waste_joules: 50,
        },
        RegionGdp {
            region_id: 3,
            production_joules: 250,
            waste_joules: 0,
        },
    ];

    let result = compute_gdp(&regions);

    assert_eq!(result.region_count, 3);
    assert_eq!(result.regions, regions, "region order and ids are preserved");
    let expected = 900 + 450 + 250;
    assert_eq!(result.total_gdp, expected, "aggregate is the regional sum");
    assert_eq!(
        result.total_gdp,
        regions
            .iter()
            .map(RegionGdp::net_throughput)
            .sum::<i64>(),
        "aggregate GDP equals the exact sum of net throughput"
    );
    for region in &result.regions {
        assert_eq!(
            region.gdp_contribution(),
            region.net_throughput(),
            "at a 1.00 exchange rate the contribution is the region's net throughput"
        );
    }
}

/// Regions with no throughput, and the empty economy, contribute exactly zero.
/// Covers FR-ECON-006.
#[test]
fn gdp_is_zero_for_an_empty_or_idle_economy() {
    let empty = compute_gdp(&[]);
    assert_eq!(empty.total_gdp, 0);
    assert_eq!(empty.region_count, 0);
    assert!(empty.regions.is_empty());

    let idle = compute_gdp(&[RegionGdp {
        region_id: 4,
        production_joules: 0,
        waste_joules: 0,
    }]);
    assert_eq!(idle.region_count, 1);
    assert_eq!(idle.total_gdp, 0);
}

/// A region whose waste meets or exceeds production contributes zero, never a
/// negative figure: deficit regions cannot subtract from national GDP.
/// Covers FR-ECON-006.
#[test]
fn gdp_of_deficit_regions_clamps_to_zero_and_never_goes_negative() {
    let regions = [
        RegionGdp {
            region_id: 0,
            production_joules: 100,
            waste_joules: 200,
        },
        RegionGdp {
            region_id: 1,
            production_joules: 300,
            waste_joules: 300,
        },
        RegionGdp {
            region_id: 2,
            production_joules: 400,
            waste_joules: 100,
        },
    ];

    let result = compute_gdp(&regions);

    for region in &result.regions[0..2] {
        assert_eq!(
            region.net_throughput(),
            0,
            "waste in excess of production clamps net throughput at zero"
        );
        assert_eq!(region.gdp_contribution(), 0);
    }
    assert_eq!(result.total_gdp, 300, "only the surplus region contributes");
    assert!(
        result.total_gdp >= 0,
        "aggregate GDP is non-negative even with deficit regions"
    );
}

/// With a fixed exchange rate the GDP function is homogeneous of degree one and
/// monotone: scaling every region's joules scales GDP by the same factor, and
/// extra production never lowers GDP.
/// Covers FR-ECON-006.
#[test]
fn gdp_scales_linearly_and_is_monotone_in_joule_throughput() {
    let base = [
        RegionGdp {
            region_id: 0,
            production_joules: 1_234,
            waste_joules: 234,
        },
        RegionGdp {
            region_id: 1,
            production_joules: 500,
            waste_joules: 0,
        },
    ];
    let base_gdp = compute_gdp(&base).total_gdp;
    assert_eq!(base_gdp, 1_500);

    for factor in [2i64, 3, 7, 100] {
        let scaled: Vec<RegionGdp> = base
            .iter()
            .map(|region| RegionGdp {
                region_id: region.region_id,
                production_joules: region.production_joules * factor,
                waste_joules: region.waste_joules * factor,
            })
            .collect();
        assert_eq!(
            compute_gdp(&scaled).total_gdp,
            base_gdp * factor,
            "GDP must scale exactly with throughput at a fixed rate (factor {factor})"
        );
    }

    for extra in [1i64, 10, 1_000] {
        let grown = [
            RegionGdp {
                region_id: 0,
                production_joules: 1_234 + extra,
                waste_joules: 234,
            },
            base[1],
        ];
        let grown_gdp = compute_gdp(&grown).total_gdp;
        assert!(
            grown_gdp > base_gdp,
            "extra production of {extra} joules must raise GDP: {grown_gdp} vs {base_gdp}"
        );
        assert_eq!(grown_gdp - base_gdp, extra);
    }
}

/// Integer aggregation: summing a large number of regions stays exact, is
/// recorded as an `i64` (no float path), and equals the independently computed
/// sum rather than an average.
/// Covers FR-ECON-006.
#[test]
fn gdp_aggregation_is_exact_integer_arithmetic() {
    let regions: Vec<RegionGdp> = (0..100_000u32)
        .map(|region_id| RegionGdp {
            region_id,
            production_joules: 10,
            waste_joules: 3,
        })
        .collect();

    let result = compute_gdp(&regions);

    // Compile-time guarantee: GDP is an integer quantity.
    let total: i64 = result.total_gdp;
    assert_eq!(result.region_count, 100_000);
    assert_eq!(total, 700_000, "100_000 regions x 7 net joules, exactly");
    assert_ne!(total, 7, "the aggregate is a sum, not an average");
    let rebuilt: i64 = result
        .regions
        .iter()
        .map(RegionGdp::gdp_contribution)
        .sum();
    assert_eq!(rebuilt, total, "no drift between aggregate and parts");
}

/// Identical regional inputs always produce identical GDP (replay safety).
/// Covers FR-ECON-006.
#[test]
fn gdp_computation_is_deterministic() {
    let regions = vec![
        RegionGdp {
            region_id: 0,
            production_joules: 987_654,
            waste_joules: 12_345,
        },
        RegionGdp {
            region_id: 1,
            production_joules: 4_321,
            waste_joules: 4_000,
        },
    ];
    let first = compute_gdp(&regions);
    for run in 0..32 {
        assert_eq!(
            compute_gdp(&regions),
            first,
            "run {run} produced a different GDP result"
        );
    }
}

/// Aggregation is commutative and waste subtracts joule-for-joule at the fixed
/// rate: permuting regions cannot change GDP, extra waste lowers it by exactly
/// the amount added, and once waste exceeds a region's production that region
/// floors at zero and can no longer pull the national figure down.
/// Covers FR-ECON-006.
#[test]
fn gdp_is_permutation_invariant_and_waste_subtracts_exactly() {
    let forward = [
        RegionGdp {
            region_id: 0,
            production_joules: 1_000,
            waste_joules: 100,
        },
        RegionGdp {
            region_id: 1,
            production_joules: 2_000,
            waste_joules: 250,
        },
        RegionGdp {
            region_id: 2,
            production_joules: 37,
            waste_joules: 0,
        },
    ];
    let baseline = compute_gdp(&forward).total_gdp;
    assert_eq!(baseline, 900 + 1_750 + 37);

    let reversed = [forward[2], forward[1], forward[0]];
    assert_eq!(
        compute_gdp(&reversed).total_gdp,
        baseline,
        "region order must not change the aggregate at a fixed exchange rate"
    );

    for extra_waste in [1i64, 100, 500] {
        let dirtier = [
            RegionGdp {
                region_id: 0,
                production_joules: 1_000,
                waste_joules: 100 + extra_waste,
            },
            forward[1],
            forward[2],
        ];
        assert_eq!(
            compute_gdp(&dirtier).total_gdp,
            baseline - extra_waste,
            "adding {extra_waste} joules of waste must subtract exactly that much GDP"
        );
    }

    let massively_wasteful = [
        RegionGdp {
            region_id: 0,
            production_joules: 1_000,
            waste_joules: 5_000,
        },
        forward[1],
        forward[2],
    ];
    assert_eq!(
        compute_gdp(&massively_wasteful).total_gdp,
        baseline - 900,
        "a deficit region floors at zero and cannot subtract more than it produced"
    );
}

// ===========================================================================
// FR-ECON-009 — subsistence mode
// ===========================================================================

/// Subsistence activates strictly below the threshold: one joule under trips it,
/// exactly at the threshold does not.
/// Covers FR-ECON-009.
#[test]
fn subsistence_activates_strictly_below_the_threshold() {
    let mut mode = SubsistenceMode::new();
    let threshold = mode.threshold();
    assert_eq!(
        threshold, DEFAULT_SUBSISTENCE_THRESHOLD,
        "the default mode uses the declared default threshold"
    );
    assert!(
        threshold > 0,
        "the default subsistence threshold must be a positive joule reserve"
    );

    assert!(
        mode.update(threshold - 1),
        "one joule below the threshold must activate subsistence"
    );
    assert!(mode.is_active());
    assert!(
        !mode.update(threshold),
        "reserves exactly at the threshold are not below it"
    );
    assert!(!mode.is_active());
    assert!(!mode.update(threshold + 1));
}

/// The active flag is exactly the predicate `reserves < threshold` at every step
/// of a trajectory, including negative and extreme reserves.
/// Covers FR-ECON-009.
#[test]
fn subsistence_active_flag_tracks_the_deficit_predicate_across_a_trajectory() {
    let threshold = 500;
    let mut mode = SubsistenceMode::with_threshold(threshold);
    assert_eq!(mode.threshold(), threshold);
    assert!(!mode.is_active(), "a fresh mode starts out of subsistence");

    for reserves in [600i64, 499, 499, 500, 0, -1, 1_000, 250, i64::MIN, i64::MAX] {
        let active = mode.update(reserves);
        assert_eq!(active, mode.is_active());
        assert_eq!(
            active,
            reserves < threshold,
            "active must equal (reserves < threshold) for reserves={reserves}"
        );
    }
}

/// Ticks below the threshold accumulate; the first tick at or above the
/// threshold resets the counter to zero and starts a fresh run at 1.
/// Covers FR-ECON-009.
#[test]
fn subsistence_counts_consecutive_deficit_ticks_and_resets_on_recovery() {
    let mut mode = SubsistenceMode::with_threshold(100);

    for expected in 1..=5u64 {
        assert!(mode.update(10));
        assert_eq!(
            mode.consecutive_ticks(),
            expected,
            "each deficit tick increments the consecutive counter"
        );
    }

    assert!(!mode.update(1_000), "recovery ends subsistence");
    assert_eq!(
        mode.consecutive_ticks(),
        0,
        "recovery resets the consecutive deficit counter"
    );

    assert!(mode.update(99));
    assert_eq!(
        mode.consecutive_ticks(),
        1,
        "a new deficit run starts from one, not from the old count"
    );
}

/// Threshold boundaries: a zero threshold needs a negative reserve, a negative
/// threshold uses the same strict comparison, and extreme reserves behave.
/// Covers FR-ECON-009.
#[test]
fn subsistence_threshold_boundaries_are_consistent() {
    let mut zero_threshold = SubsistenceMode::with_threshold(0);
    assert_eq!(zero_threshold.threshold(), 0);
    assert!(
        !zero_threshold.update(0),
        "0 is not below a 0 threshold (strict comparison)"
    );
    assert!(
        zero_threshold.update(-1),
        "an overdrawn reserve activates subsistence even at a 0 threshold"
    );

    let mut negative_threshold = SubsistenceMode::with_threshold(-5);
    assert!(!negative_threshold.update(-5));
    assert!(negative_threshold.update(-6));

    let mut extremes = SubsistenceMode::with_threshold(100);
    assert!(extremes.update(i64::MIN), "the most overdrawn reserve activates");
    assert!(!extremes.update(i64::MAX), "the largest reserve deactivates");
}

/// Two modes fed the same reserve trajectory stay in lock-step, and the
/// constructor / `Default` paths agree on the declared threshold.
/// Covers FR-ECON-009.
#[test]
fn subsistence_state_is_deterministic_for_identical_trajectories() {
    let mut a = SubsistenceMode::with_threshold(100);
    let mut b = SubsistenceMode::with_threshold(100);

    for reserves in [50i64, 40, 150, 20, 0, 300, 99] {
        assert_eq!(
            a.update(reserves),
            b.update(reserves),
            "identical reserves must yield identical activation for reserves={reserves}"
        );
        assert_eq!(a.is_active(), b.is_active());
        assert_eq!(a.consecutive_ticks(), b.consecutive_ticks());
    }

    assert_eq!(SubsistenceMode::default(), SubsistenceMode::new());
    assert_eq!(
        SubsistenceMode::default().threshold(),
        DEFAULT_SUBSISTENCE_THRESHOLD
    );
}

// ===========================================================================
// FR-ECON-010 — treasury in MilliCredits (i64, no float accumulation)
// ===========================================================================

/// A million single-milli-credit operations must net to exact integers: with a
/// floating-point balance the same loop drifts. This is the accumulation-law
/// check the requirement exists for.
/// Covers FR-ECON-010.
#[test]
fn treasury_stays_exact_after_a_million_single_milli_credit_operations() {
    let mut treasury = Treasury::new(0, 0);
    for _ in 0..1_000_000 {
        treasury.credit_milli_credits(1);
    }
    assert_eq!(treasury.balance_milli_credits(), 1_000_000);

    for _ in 0..1_000_000 {
        treasury.debit_milli_credits(1);
    }
    assert_eq!(
        treasury.balance_milli_credits(),
        0,
        "one million credits then one million debits must return to exactly zero"
    );
}

/// 2^53 is the largest integer `f64` represents consecutively; 2^53 + 1 is not
/// representable. An `f64` balance would silently swallow the last milli-credit.
/// Covers FR-ECON-010.
#[test]
fn treasury_keeps_a_one_milli_credit_difference_beyond_f64_exact_range() {
    const TWO_POW_53: i64 = 1 << 53;

    let mut treasury = Treasury::new(0, 0);
    treasury.credit_milli_credits(TWO_POW_53);
    assert_eq!(treasury.balance_milli_credits(), TWO_POW_53);

    treasury.credit_milli_credits(1);
    let after = treasury.balance_milli_credits();
    assert_eq!(after, TWO_POW_53 + 1);
    assert_eq!(
        after - TWO_POW_53,
        1,
        "the single milli-credit must survive; an f64 balance would round it away"
    );

    treasury.debit_milli_credits(1);
    assert_eq!(treasury.balance_milli_credits(), TWO_POW_53);
}

/// Value conservation across a long deterministic credit/debit sequence: the
/// final balance equals credited minus debited exactly, and the balance never
/// goes negative.
/// Covers FR-ECON-010.
#[test]
fn treasury_credit_debit_bookkeeping_conserves_value_exactly() {
    const OPENING: i64 = 10_000_000;
    let mut treasury = Treasury::new(1_000, OPENING);
    let mut credited: i64 = 0;
    let mut debited: i64 = 0;
    let mut seed: u64 = 0x5EED_1234;

    for step in 0..10_000 {
        let delta = (lcg(&mut seed) % 997) as i64 + 1;
        if delta % 2 == 0 {
            treasury.credit_milli_credits(delta);
            credited += delta;
        } else {
            treasury.debit_milli_credits(delta);
            debited += delta;
        }
        assert!(
            treasury.balance_milli_credits() >= 0,
            "step {step}: opening balance covers every debit in this sequence"
        );
    }

    assert_eq!(
        treasury.balance_milli_credits(),
        OPENING + credited - debited,
        "final balance must equal opening + credited - debited exactly"
    );
    assert_eq!(
        treasury.balance_joules(),
        1_000,
        "milli-credit movement must not perturb the joule balance"
    );
}

/// The joule and milli-credit ledgers are independent and reversible: an equal
/// credit/debit pair is a no-op on the balance it touches and on the other.
/// Covers FR-ECON-010.
#[test]
fn treasury_accounts_are_independent_and_reversible() {
    let mut treasury = Treasury::new(0, 0);
    treasury.credit_joules(2_000_000_000);
    treasury.credit_milli_credits(1);
    treasury.debit_joules(2_000_000_000);
    assert_eq!(treasury.balance_joules(), 0);
    assert_eq!(
        treasury.balance_milli_credits(),
        1,
        "joule movement must not perturb the milli-credit balance"
    );

    let mut restored = Treasury::new(7, 9);
    restored.credit_joules(3);
    restored.debit_joules(3);
    restored.credit_milli_credits(3);
    restored.debit_milli_credits(3);
    assert_eq!(
        restored,
        Treasury::new(7, 9),
        "credit followed by an equal debit restores the exact balance"
    );

    assert_eq!(Treasury::default(), Treasury::new(0, 0));
    // Compile-time guarantee: MilliCredits are i64, never a float.
    let milli_credits: i64 = Treasury::default().balance_milli_credits();
    assert_eq!(milli_credits, 0);
    let _: i64 = 1_000; // 1 credit == 1000 milli-credits (unit documentation).
}

/// A fixed multiset of milli-credit operations must reach the same exact balance
/// whatever order it is applied in, and unit credits just above 2^53 must each
/// register. Order-independence and exactness at the `f64` boundary are exactly
/// what a float balance would break.
/// Covers FR-ECON-010.
#[test]
fn treasury_balance_is_order_independent_and_exact_at_the_float_boundary() {
    const TWO_POW_53: i64 = 1 << 53;

    let credits = [7i64, 11, 13, 1_000_000, 4];
    let debits = [3i64, 5, 999_999];
    let expected = credits.iter().sum::<i64>() - debits.iter().sum::<i64>();
    assert_eq!(expected, 28);

    let mut sequential = Treasury::default();
    for amount in credits {
        sequential.credit_milli_credits(amount);
    }
    for amount in debits {
        sequential.debit_milli_credits(amount);
    }
    assert_eq!(sequential.balance_milli_credits(), expected);

    let mut interleaved = Treasury::default();
    for (credit, debit) in credits
        .iter()
        .zip(debits.iter().copied().chain(std::iter::repeat(0i64)))
    {
        interleaved.credit_milli_credits(*credit);
        interleaved.debit_milli_credits(debit);
    }
    assert_eq!(
        interleaved.balance_milli_credits(),
        expected,
        "credit/debit interleaving must not change the final balance"
    );

    // Exactness at the f64 boundary: base 2^53, then ten unit credits, which an
    // f64 accumulator would absorb (2^53 + 1 is not representable in f64).
    let mut edge = Treasury::default();
    edge.credit_milli_credits(TWO_POW_53);
    for _ in 0..10 {
        edge.credit_milli_credits(1);
    }
    assert_eq!(
        edge.balance_milli_credits(),
        TWO_POW_53 + 10,
        "each unit credit above 2^53 must register exactly"
    );
    edge.debit_milli_credits(10);
    assert_eq!(edge.balance_milli_credits(), TWO_POW_53);
}
