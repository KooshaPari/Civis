//! BDD-style integration tests for energy allocation and tiered consumer demand.
//!
//! FR-ECON-005 — allocation engines (capitalist, planned, joule) and priority-tier
//! rationing: subsistence is filled before luxury, zero budget grants nothing, and
//! partition sums never exceed the available budget.

use civ_economy::{
    allocate_by_priority, allocate_with, AllocationEngine, AllocationRegime, CapitalistAllocator,
    JouleAllocator, PlannedAllocator, PriorityTier,
};

// ---------------------------------------------------------------------------
// Given / When / Then helpers
// ---------------------------------------------------------------------------

fn given_a_planned_allocator() -> PlannedAllocator {
    PlannedAllocator
}

fn given_a_capitalist_allocator() -> CapitalistAllocator {
    CapitalistAllocator
}

fn given_a_joule_allocator() -> JouleAllocator {
    JouleAllocator
}

fn when_budget_is(budget: i64) -> i64 {
    budget
}

// ===========================================================================
// Scenario: Energy allocation under each regime
// ===========================================================================

#[test]
fn scenario_planned_allocator_fills_demand_when_budget_is_sufficient() {
    let engine = given_a_planned_allocator();
    let budget = when_budget_is(100);
    let demand = 80;

    let allocated = engine.allocate(budget, demand);

    // Then the demand is fully met
    assert_eq!(allocated, 80, "planned allocator should fill demand fully when budget >= demand");
}

#[test]
fn scenario_planned_allocator_caps_at_budget_when_demand_exceeds_supply() {
    let engine = given_a_planned_allocator();
    let budget = when_budget_is(40);
    let demand = 100;

    let allocated = engine.allocate(budget, demand);

    // Then allocation is capped at the budget ceiling
    assert_eq!(allocated, 40, "planned allocator should cap at budget, not ration proportionally");
}

#[test]
fn scenario_capitalist_allocator_rations_proportionally_when_budget_is_scarce() {
    let engine = given_a_capitalist_allocator();
    let budget = when_budget_is(50);
    let demand = 100;

    let allocated = engine.allocate(budget, demand);

    // Then the allocation is proportional to the budget/demand ratio
    assert_eq!(allocated, 50, "capitalist allocator should ration at 50% fill when budget is half of demand");
}

#[test]
fn scenario_joule_allocator_matches_planned_behavior_for_single_good() {
    let joule = given_a_joule_allocator();
    let planned = given_a_planned_allocator();
    let budget = when_budget_is(60);
    let demand = 150;

    let allocated_joule = joule.allocate(budget, demand);
    let allocated_planned = planned.allocate(budget, demand);

    // Then joule allocator behaves identically to planned for a single good
    assert_eq!(
        allocated_joule, allocated_planned,
        "joule allocator must match planned allocator at single-good granularity"
    );
}

#[test]
fn scenario_all_regimes_return_zero_when_budget_is_zero() {
    let budget = 0;
    let demand = 100;

    for regime in [
        AllocationRegime::Capitalist,
        AllocationRegime::Planned,
        AllocationRegime::Joule,
    ] {
        let allocated = allocate_with(regime, budget, demand);
        assert_eq!(
            allocated, 0,
            "regime {:?} must grant nothing when budget is zero", regime
        );
    }
}

#[test]
fn scenario_all_regimes_return_zero_when_demand_is_zero() {
    let budget = 100;
    let demand = 0;

    for regime in [
        AllocationRegime::Capitalist,
        AllocationRegime::Planned,
        AllocationRegime::Joule,
    ] {
        let allocated = allocate_with(regime, budget, demand);
        assert_eq!(
            allocated, 0,
            "regime {:?} must grant nothing when demand is zero", regime
        );
    }
}

// ===========================================================================
// Scenario: Tiered consumer demand — priority allocation
// ===========================================================================

#[test]
fn scenario_subsistence_is_filled_before_luxury() {
    // Given a scarce budget that cannot cover both subsistence and luxury
    let budget = 60;
    let demands = [
        (PriorityTier::Luxury, 50),
        (PriorityTier::Subsistence, 50),
    ];

    // When allocating by priority using the planned engine
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    // Then subsistence is fully met and luxury receives only the remainder
    assert_eq!(allocations[1], 50, "subsistence demand (index 1) must be filled first");
    assert_eq!(allocations[0], 10, "luxury demand (index 0) gets only the leftover");
}

#[test]
fn scenario_all_tiers_fully_met_when_budget_is_sufficient() {
    // Given a budget that covers all tiered demands
    let budget = 200;
    let demands = [
        (PriorityTier::Subsistence, 50),
        (PriorityTier::Basic, 40),
        (PriorityTier::Comfort, 30),
        (PriorityTier::Luxury, 20),
    ];

    // When allocating by priority
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    // Then every tier receives its full demand
    assert_eq!(allocations[0], 50, "subsistence fully met");
    assert_eq!(allocations[1], 40, "basic fully met");
    assert_eq!(allocations[2], 30, "comfort fully met");
    assert_eq!(allocations[3], 20, "luxury fully met");
}

#[test]
fn scenario_lowest_tier_is_starved_when_budget_is_tight() {
    // Given a tight budget that covers subsistence and part of basic only
    let budget = 50;
    let demands = [
        (PriorityTier::Subsistence, 40),
        (PriorityTier::Basic, 40),
        (PriorityTier::Luxury, 40),
    ];

    // When allocating by priority
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    // Then subsistence is fully met, basic gets the remainder, luxury is starved
    assert_eq!(allocations[0], 40, "subsistence fully met");
    assert_eq!(allocations[1], 10, "basic gets the remainder");
    assert_eq!(allocations[2], 0, "luxury is starved when budget runs out");
}

#[test]
fn scenario_zero_budget_grants_nothing_across_all_tiers() {
    // Given a zero budget with demands at every tier
    let budget = 0;
    let demands = [
        (PriorityTier::Subsistence, 40),
        (PriorityTier::Basic, 30),
        (PriorityTier::Comfort, 20),
        (PriorityTier::Luxury, 10),
    ];

    // When allocating by priority
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    // Then every consumer receives zero
    assert!(allocations.iter().all(|&a| a == 0), "zero budget must grant nothing across all tiers");
}

#[test]
fn scenario_negative_budget_grants_nothing_across_all_tiers() {
    // Given a negative budget
    let budget = -10;
    let demands = [
        (PriorityTier::Subsistence, 40),
        (PriorityTier::Luxury, 20),
    ];

    // When allocating by priority
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    // Then every consumer receives zero
    assert!(allocations.iter().all(|&a| a == 0), "negative budget must grant nothing across all tiers");
}

// ===========================================================================
// Scenario: Partition sums — conservation of budget
// ===========================================================================

#[test]
fn scenario_partition_sum_never_exceeds_budget() {
    // Given a random budget and a mix of tiered demands
    let budget = 123;
    let demands = [
        (PriorityTier::Subsistence, 50),
        (PriorityTier::Basic, 40),
        (PriorityTier::Comfort, 30),
        (PriorityTier::Luxury, 20),
    ];

    // When allocating by priority
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);
    let total: i64 = allocations.iter().sum();

    // Then the total allocated does not exceed the budget
    assert!(
        total <= budget,
        "partition sum {total} must not exceed budget {budget}"
    );
}

#[test]
fn scenario_partition_sum_exactly_equals_budget_when_total_demand_exceeds_budget() {
    // Given a budget that is fully exhausted by demand
    let budget = 55;
    let demands = [
        (PriorityTier::Subsistence, 30),
        (PriorityTier::Basic, 30),
        (PriorityTier::Luxury, 30),
    ];

    // When allocating by priority with planned engine (no proportional rationing)
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);
    let total: i64 = allocations.iter().sum();

    // Then the total allocated exactly equals the budget (because budget < total demand)
    assert_eq!(
        total, budget,
        "with planned engine, partition sum should exactly equal budget when demand exceeds supply"
    );
}

#[test]
fn scenario_partition_sum_with_capitalist_engine_rations_at_boundary() {
    // Given a budget that falls inside the basic tier
    let budget = 75;
    let demands = [
        (PriorityTier::Subsistence, 50),
        (PriorityTier::Basic, 50),
        (PriorityTier::Luxury, 50),
    ];

    // When allocating by priority with capitalist engine
    let allocations = allocate_by_priority(&CapitalistAllocator, budget, &demands);
    let total: i64 = allocations.iter().sum();

    // Then the total allocated does not exceed the budget
    assert!(
        total <= budget,
        "partition sum {total} must not exceed budget {budget}"
    );
    // And subsistence is fully met before the capitalist boundary tier
    assert_eq!(allocations[0], 50, "subsistence must be fully met before capitalist rationing begins");
}

#[test]
fn scenario_partition_sum_with_joule_engine_matches_planned_budget_exhaustion() {
    let budget = 80;
    let demands = [
        (PriorityTier::Subsistence, 40),
        (PriorityTier::Basic, 40),
        (PriorityTier::Luxury, 40),
    ];

    let joule_allocations = allocate_by_priority(&JouleAllocator, budget, &demands);
    let planned_allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    let joule_total: i64 = joule_allocations.iter().sum();
    let planned_total: i64 = planned_allocations.iter().sum();

    // Then joule engine behaves identically to planned for partition sums
    assert_eq!(
        joule_total, planned_total,
        "joule engine partition sum must match planned engine partition sum"
    );
    assert_eq!(
        joule_allocations, planned_allocations,
        "joule engine per-consumer allocations must match planned engine"
    );
}

#[test]
fn scenario_mixed_tier_ordering_is_resolved_by_priority_not_by_index() {
    // Given demands in shuffled tier order (luxury first, subsistence last)
    let budget = 70;
    let demands = [
        (PriorityTier::Luxury, 30),
        (PriorityTier::Comfort, 30),
        (PriorityTier::Basic, 30),
        (PriorityTier::Subsistence, 30),
    ];

    // When allocating by priority
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    // Then subsistence (index 3) is filled first despite being last in the list
    assert_eq!(allocations[3], 30, "subsistence must be filled first regardless of index order");
    assert_eq!(allocations[2], 30, "basic filled next");
    assert_eq!(allocations[1], 10, "comfort gets the remainder");
    assert_eq!(allocations[0], 0, "luxury is starved");
}

#[test]
fn scenario_zero_or_negative_demands_are_skipped_without_consuming_budget() {
    // Given some demands that are zero or negative
    let budget = 50;
    let demands = [
        (PriorityTier::Subsistence, -10),
        (PriorityTier::Basic, 0),
        (PriorityTier::Comfort, 50),
    ];

    // When allocating by priority
    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    // Then invalid demands are skipped and the valid demand gets the full budget
    assert_eq!(allocations[0], 0, "negative demand receives nothing");
    assert_eq!(allocations[1], 0, "zero demand receives nothing");
    assert_eq!(allocations[2], 50, "valid demand gets the full budget");
}

#[test]
fn scenario_priority_total_with_single_consumer_and_sufficient_budget() {
    let budget = 100;
    let demands = [(PriorityTier::Luxury, 80)];

    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    assert_eq!(allocations[0], 80, "single consumer gets full demand when budget is sufficient");
    assert_eq!(allocations.iter().sum::<i64>(), 80, "partition sum equals the single allocation");
}

#[test]
fn scenario_priority_total_with_single_consumer_and_insufficient_budget() {
    let budget = 30;
    let demands = [(PriorityTier::Subsistence, 80)];

    let allocations = allocate_by_priority(&PlannedAllocator, budget, &demands);

    assert_eq!(allocations[0], 30, "single consumer gets capped at budget");
    assert_eq!(allocations.iter().sum::<i64>(), 30, "partition sum equals the budget cap");
}

#[test]
fn scenario_allocate_with_dispatches_all_three_regimes_correctly() {
    let budget = 40;
    let demand = 100;

    let capitalist = allocate_with(AllocationRegime::Capitalist, budget, demand);
    let planned = allocate_with(AllocationRegime::Planned, budget, demand);
    let joule = allocate_with(AllocationRegime::Joule, budget, demand);

    // Then capitalist rations proportionally, planned and joule cap at budget
    assert_eq!(capitalist, 40, "capitalist should ration at 40% fill");
    assert_eq!(planned, 40, "planned should cap at budget");
    assert_eq!(joule, 40, "joule should cap at budget");
}

#[test]
fn scenario_allocate_with_never_exceeds_budget_for_any_regime() {
    let budget = 25;
    let demand = 1_000;

    for regime in [
        AllocationRegime::Capitalist,
        AllocationRegime::Planned,
        AllocationRegime::Joule,
    ] {
        let allocated = allocate_with(regime, budget, demand);
        assert!(
            allocated <= budget,
            "regime {:?} allocated {allocated} exceeds budget {budget}", regime
        );
    }
}
