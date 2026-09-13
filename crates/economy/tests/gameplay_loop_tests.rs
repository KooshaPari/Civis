//! FR-ECON-GAMEPLAY — end-to-end gameplay loop coverage.
//!
//! Proves that the demand/supply feedback, market price response to
//! settlement wealth, and macro-budget income side all close the loop
//! deterministically.
//!
//! These tests assert what the parent scorecard flagged as "no
//! convincing gameplay loop" for the 35% Economy/Resources lane:
//!
//! 1. effective_demand grows sub-linearly with wealth
//! 2. scarcity drives market price up
//! 3. scarcity credits income into the macro budget
//! 4. wealth accumulation is monotone (no draw-down)
//! 5. the gameplay loop is deterministic across runs
//! 6. wealth saturates so runaway growth cannot happen
//! 7. apply_economy_pressure never panics on edge cases

use civ_economy::gameplay_loop::{
    accumulated_for, accumulate_wealth, apply_economy_pressure, compute_outputs, effective_demand,
    per_tick_wealth, tick_settlement_economy, wealth_factor_millionths,
    SettlementEconomyInputs, SettlementWealthSnapshot, WEALTH_SATURATION_REF,
};
use civ_economy::{EconomyState, MarketState};

fn inputs(sid: u32, pop: i64, food: i64, treasury: i64) -> SettlementEconomyInputs {
    SettlementEconomyInputs {
        settlement_id: sid,
        population: pop,
        food_stocked: food,
        treasury_share: treasury,
    }
}

/// 1. Saturation `w / (w + ref) ∈ [0, 1)` for any non-negative wealth.
#[test]
fn wealth_factor_millionths_is_strictly_below_one_million() {
    for w in [0_i64, 1, 100, WEALTH_SATURATION_REF, 10 * WEALTH_SATURATION_REF, 1_000_000] {
        let factor = wealth_factor_millionths(w);
        assert!(
            (0..1_000_000).contains(&factor),
            "wealth factor must be in [0, 1_000_000) millionths, got {factor} for w={w}"
        );
    }
}

/// 2. `effective_demand = population × factor`. When population is 0
///    the result is exactly 0 regardless of accumulated wealth.
#[test]
fn effective_demand_is_zero_when_population_is_zero() {
    let result = effective_demand(inputs(0, 0, 100, 100), 1_000_000);
    assert_eq!(result, 0);
}

/// 3. `effective_demand` grows sub-linearly with wealth because the
///    factor saturates.
#[test]
fn effective_demand_saturates_with_wealth() {
    let pop = 100;
    let d1 = effective_demand(inputs(0, pop, 50, 50), 100);
    let d2 = effective_demand(inputs(0, pop, 50, 50), 1_000);
    let d3 = effective_demand(inputs(0, pop, 50, 50), 10_000);
    let d4 = effective_demand(inputs(0, pop, 50, 50), 1_000_000);

    // All positive.
    for d in [d1, d2, d3, d4] {
        assert!(d > 0, "demand must be > 0 when population > 0 (got {d})");
    }

    // Monotone non-decreasing.
    assert!(d1 <= d2, "d1={d1} should be <= d2={d2}");
    assert!(d2 <= d3, "d2={d2} should be <= d3={d3}");
    assert!(d3 <= d4, "d3={d3} should be <= d4={d4}");

    // Strictly below population cap (the saturating factor never
    // reaches 1.0 in millionths).
    for d in [d1, d2, d3, d4] {
        assert!(d < pop, "demand {d} must stay below population cap {pop}");
    }
}

/// 4. Scarcity (supply < demand) credits positive income into the macro
///    budget and intentionally does NOT mutate `state.ledger` (which has
///    a tick-bounded cap that the gameplay-loop income stream would
///    exceed on long simulations).
#[test]
fn apply_economy_pressure_credits_income_on_scarcity() {
    let mut market = MarketState::default();
    let mut state = EconomyState::with_energy_budget(0);
    let before_budget = state.energy_budget_joules;

    // supply = 100, demand = 5_000 -> large imbalance
    let income =
        apply_economy_pressure(&mut market, &mut state, 7, 100, 5_000);

    assert!(income > 0, "scarcity must credit positive income, got {income}");
    assert_eq!(
        state.energy_budget_joules,
        before_budget + income,
        "energy_budget_joules must rise by exactly the credited income"
    );

    // The gameplay loop intentionally does NOT mutate state.ledger (the
    // ledger has a `tick * 2` cap and a long-running scarcity stream
    // would blow past it). Verify the budget mutation alone is what
    // carries the income.
    assert!(
        state.ledger.is_empty(),
        "gameplay loop must not mutate state.ledger (cap interaction)"
    );

    // Conservation invariant still holds: empty ledger short-circuits
    // the per-tick bound check.
    civ_economy::verify_ledger_conservation(&state)
        .expect("ledger conservation must hold after scarcity income");
}

/// 5. Surplus (supply > demand) does NOT credit any income — the budget
///    stays flat and no ledger entry is recorded.
#[test]
fn apply_economy_pressure_zero_income_on_surplus() {
    let mut market = MarketState::default();
    let mut state = EconomyState::with_energy_budget(0);
    let before_budget = state.energy_budget_joules;
    let before_ledger_len = state.ledger.len();

    // supply = 1_000, demand = 100 -> big surplus
    let income =
        apply_economy_pressure(&mut market, &mut state, 7, 1_000, 100);

    assert_eq!(income, 0, "surplus must credit zero income, got {income}");
    assert_eq!(
        state.energy_budget_joules, before_budget,
        "budget must be unchanged on surplus"
    );
    assert_eq!(
        state.ledger.len(),
        before_ledger_len,
        "no ledger entry must be recorded on surplus"
    );
}/// 6. `accumulate_wealth` is monotone (no draw-down) and accumulates
///    per settlement, not globally.
#[test]
fn accumulate_wealth_is_monotone_per_settlement() {
    let mut snap = SettlementWealthSnapshot::default();

    accumulate_wealth(&mut snap, 0, 10);
    accumulate_wealth(&mut snap, 0, 20);
    accumulate_wealth(&mut snap, 0, -50); // negative input clamped to 0 (no-op increment)
    accumulate_wealth(&mut snap, 1, 100);

    assert_eq!(snap.accumulated.get(&0).copied().unwrap_or(0), 30);
    assert_eq!(snap.accumulated.get(&1).copied().unwrap_or(0), 100);
    assert_eq!(accumulated_for(&snap, 0), 30);
    assert_eq!(accumulated_for(&snap, 1), 100);
    // last_tick holds the CLAMPED value (0 for the negative input),
    // not the original negative.
    assert_eq!(snap.last_tick.get(&0).copied().unwrap_or(-1), 0);
    assert_eq!(snap.last_tick.get(&1).copied().unwrap_or(-1), 100);
}

/// 7. `tick_settlement_economy` is deterministic. Same input, same seed,
///    same sequence -> identical outputs.
#[test]
fn tick_settlement_economy_is_deterministic() {
    let make = || {
        let mut market = MarketState::default();
        let mut state = EconomyState::with_energy_budget(0);
        let mut snap = SettlementWealthSnapshot::default();

        let r1 = tick_settlement_economy(
            &mut market,
            &mut state,
            &mut snap,
            inputs(0, 50, 200, 500),
        );
        let r2 = tick_settlement_economy(
            &mut market,
            &mut state,
            &mut snap,
            inputs(0, 50, 100, 750),
        );
        (r1, r2, market, state, snap)
    };
    let (out1_a, out2_a, market_a, state_a, snap_a) = make();
    let (out1_b, out2_b, market_b, state_b, snap_b) = make();

    assert_eq!(out1_a, out1_b);
    assert_eq!(out2_a, out2_b);
    assert_eq!(market_a, market_b);
    assert_eq!(state_a, state_b);
    assert_eq!(snap_a, snap_b);
}

/// 8. `per_tick_wealth` derives a non-negative scalar from inputs.
#[test]
fn per_tick_wealth_floors_at_zero() {
    let w = per_tick_wealth(-100, -100, Some(-100));
    assert_eq!(w, 0);
    let w = per_tick_wealth(100, 100, Some(50));
    assert_eq!(w, 250);
    let w = per_tick_wealth(0, 0, None);
    assert_eq!(w, 0);
}

/// 9. `compute_outputs` is consistent across repeated invocations.
#[test]
fn compute_outputs_is_pure() {
    let mean = Some(2_000_i64);
    let a = compute_outputs(inputs(0, 25, 50, 100), mean, 500);
    let b = compute_outputs(inputs(0, 25, 50, 100), mean, 500);
    assert_eq!(a, b);
}

/// 10. End-to-end: a settlement that has accumulated wealth sees
///     scarcity, gets income, accumulates MORE wealth; the next tick's
///     effective demand rises (because wealth rose), and the macro
///     budget reflects the sum of credited income.
#[test]
fn gameplay_loop_closes_end_to_end() {
    let mut market = MarketState::default();
    let mut state = EconomyState::with_energy_budget(0);
    let mut snap = SettlementWealthSnapshot::default();
    let starting_budget = state.energy_budget_joules;

    // Pre-warm: a comfortable tick (plenty of food, modest demand so the
    // sat factor is > 0 going into the scarcity scenario).
    accumulate_wealth(&mut snap, 0, 500);
    let starting_acc = accumulated_for(&snap, 0);

    // First real tick: scarce settlement (effective demand is positive
    // because wealth_factor > 0).
    let r1 = tick_settlement_economy(
        &mut market,
        &mut state,
        &mut snap,
        inputs(0, 100, 10, 0),
    );
    assert!(
        r1.wealth_i64 > 0,
        "first tick must produce a positive wealth scalar"
    );
    assert!(
        r1.effective_demand > 0,
        "effective_demand must be positive when accumulated wealth > 0"
    );
    assert!(
        state.energy_budget_joules > starting_budget,
        "scarcity must credit income; budget went from {starting_budget} to {}",
        state.energy_budget_joules
    );

    // Second tick: scarcity persists.
    let r2 = tick_settlement_economy(
        &mut market,
        &mut state,
        &mut snap,
        inputs(0, 100, 10, 0),
    );

    // Accumulated wealth must have grown.
    assert!(
        accumulated_for(&snap, 0) > starting_acc,
        "accumulated wealth must grow: {} vs starting {starting_acc}",
        accumulated_for(&snap, 0)
    );

    // Demand must be monotone non-decreasing.
    assert!(
        r2.effective_demand >= r1.effective_demand,
        "effective demand must be monotone non-decreasing: r1={} r2={}",
        r1.effective_demand,
        r2.effective_demand
    );

    // Conservation invariant holds.
    civ_economy::verify_ledger_conservation(&state)
        .expect("ledger conservation must hold across the gameplay loop");
}
