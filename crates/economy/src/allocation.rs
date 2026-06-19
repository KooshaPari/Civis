//! Allocation engines (CIV-002 §allocation, FR-ECON-005). Joule/planned regimes
//! plus consumer priority-tier allocation (subsistence filled before luxury).

/// Pluggable allocation mechanism (capitalist, planned, joule, hybrid).
pub trait AllocationEngine {
    /// Allocate up to `demand` from `budget` (joules or good units).
    fn allocate(&self, budget: i64, demand: i64) -> i64;
}

/// Capitalist / market regime: proportional rationing when supply is scarce.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CapitalistAllocator;

impl AllocationEngine for CapitalistAllocator {
    fn allocate(&self, budget: i64, demand: i64) -> i64 {
        if demand <= 0 || budget <= 0 {
            return 0;
        }
        // fill_bps = min(10_000, budget * 10_000 / demand)
        let fill_bps = budget
            .saturating_mul(10_000)
            .checked_div(demand)
            .unwrap_or(i64::MAX)
            .min(10_000);
        demand.saturating_mul(fill_bps) / 10_000
    }
}

/// Planned / command regime: satisfy demand fully up to the budget ceiling, no
/// proportional rationing. Distributional shortfall is borne entirely by demand
/// that exceeds budget (caller orders consumers by priority — see
/// [`allocate_by_priority`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlannedAllocator;

impl AllocationEngine for PlannedAllocator {
    fn allocate(&self, budget: i64, demand: i64) -> i64 {
        if demand <= 0 || budget <= 0 {
            return 0;
        }
        demand.min(budget)
    }
}

/// Joule / thermodynamic regime: identical fill curve to the planned regime at a
/// single good, but kept distinct so engines can weight by joule cost when the
/// hybrid scheduler routes energy-priced goods through it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JouleAllocator;

impl AllocationEngine for JouleAllocator {
    fn allocate(&self, budget: i64, demand: i64) -> i64 {
        if demand <= 0 || budget <= 0 {
            return 0;
        }
        demand.min(budget)
    }
}

/// Consumer priority tiers, highest priority first (declaration order = ranking).
/// Subsistence demand is fully met before any lower tier receives a unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PriorityTier {
    /// Survival floor (food, water, heat) — filled first.
    Subsistence,
    /// Basic goods (clothing, tools).
    Basic,
    /// Comfort goods (housing upgrades, leisure).
    Comfort,
    /// Luxury goods — filled last, first to be cut.
    Luxury,
}

/// Allocate a scarce `budget` across tiered demands, highest priority first.
///
/// Higher tiers are filled completely before lower tiers receive anything. The
/// tier where the budget runs out is rationed proportionally via `engine`; all
/// strictly-lower tiers receive zero. Returns one allocation per input demand,
/// in the input order (not sorted), so callers can zip results back to consumers.
pub fn allocate_by_priority(
    engine: &dyn AllocationEngine,
    budget: i64,
    demands: &[(PriorityTier, i64)],
) -> Vec<i64> {
    let mut out = vec![0i64; demands.len()];
    if budget <= 0 {
        return out;
    }

    // Stable index list sorted by tier priority (Subsistence first).
    let mut order: Vec<usize> = (0..demands.len()).collect();
    order.sort_by_key(|&i| demands[i].0);

    let mut remaining = budget;
    for &i in &order {
        let (_, demand) = demands[i];
        if demand <= 0 || remaining <= 0 {
            continue;
        }
        let granted = engine.allocate(remaining, demand);
        out[i] = granted;
        remaining = remaining.saturating_sub(granted);
    }
    out
}

/// Selectable allocation regime — the economy layer picks one and routes all
/// rationing through [`allocate_with`] (FR-ECON-005). Serializable so a scenario
/// or policy can set the regime deterministically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AllocationRegime {
    /// Proportional market rationing (price-clearing proxy). Default.
    #[default]
    Capitalist,
    /// Command fill: meet demand up to budget, no proportional rationing.
    Planned,
    /// Energy-priced fill; identical curve to planned at a single good, kept
    /// distinct so the hybrid scheduler can weight by joule cost.
    Joule,
}

/// Allocate `demand` from `budget` under the chosen [`AllocationRegime`].
///
/// Single dispatch point so callers (e.g. the engine's economy phase) select a
/// regime without naming concrete allocator types.
#[must_use]
pub fn allocate_with(regime: AllocationRegime, budget: i64, demand: i64) -> i64 {
    match regime {
        AllocationRegime::Capitalist => CapitalistAllocator.allocate(budget, demand),
        AllocationRegime::Planned => PlannedAllocator.allocate(budget, demand),
        AllocationRegime::Joule => JouleAllocator.allocate(budget, demand),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn capitalist_allocator_fills_demand_when_budget_sufficient() {
        let alloc = CapitalistAllocator;
        assert_eq!(alloc.allocate(100, 50), 50);
        assert_eq!(alloc.allocate(100, 100), 100);
    }

    #[test]
    fn capitalist_allocator_rations_proportionally_when_budget_scarce() {
        let alloc = CapitalistAllocator;
        assert_eq!(alloc.allocate(50, 100), 50);
        assert_eq!(alloc.allocate(33, 100), 33);
    }

    #[test]
    fn capitalist_allocator_returns_zero_for_non_positive_inputs() {
        let alloc = CapitalistAllocator;
        assert_eq!(alloc.allocate(0, 100), 0);
        assert_eq!(alloc.allocate(100, 0), 0);
        assert_eq!(alloc.allocate(-10, 50), 0);
        assert_eq!(alloc.allocate(50, -1), 0);
    }

    #[test]
    fn allocate_with_dispatches_per_regime() {
        // Scarce budget (40) vs demand (100): capitalist rations proportionally
        // (40), planned/joule cap at the budget ceiling (also 40 here) — same
        // single-good result, but routed through distinct regimes.
        assert_eq!(allocate_with(AllocationRegime::Capitalist, 40, 100), 40);
        assert_eq!(allocate_with(AllocationRegime::Planned, 40, 100), 40);
        assert_eq!(allocate_with(AllocationRegime::Joule, 40, 100), 40);
        // Sufficient budget: all regimes fill the demand.
        assert_eq!(allocate_with(AllocationRegime::Planned, 100, 60), 60);
        // Default regime is Capitalist.
        assert_eq!(AllocationRegime::default(), AllocationRegime::Capitalist);
    }

    #[test]
    fn allocate_with_never_exceeds_budget() {
        for regime in [
            AllocationRegime::Capitalist,
            AllocationRegime::Planned,
            AllocationRegime::Joule,
        ] {
            assert!(allocate_with(regime, 30, 1_000) <= 30);
            assert_eq!(allocate_with(regime, 0, 100), 0);
        }
    }

    #[test]
    fn planned_allocator_fills_to_ceiling_then_caps() {
        let alloc = PlannedAllocator;
        assert_eq!(alloc.allocate(100, 50), 50); // demand met fully
        assert_eq!(alloc.allocate(40, 100), 40); // capped at budget, not rationed
    }

    #[test]
    fn joule_allocator_matches_planned_at_single_good() {
        assert_eq!(JouleAllocator.allocate(40, 100), PlannedAllocator.allocate(40, 100));
    }

    #[test]
    fn priority_tier_orders_subsistence_first() {
        assert!(PriorityTier::Subsistence < PriorityTier::Luxury);
        assert!(PriorityTier::Basic < PriorityTier::Comfort);
    }

    #[test]
    fn priority_fills_subsistence_before_luxury() {
        // Budget 60 cannot cover both 50 (subsistence) + 50 (luxury).
        let demands = [(PriorityTier::Luxury, 50), (PriorityTier::Subsistence, 50)];
        let got = allocate_by_priority(&PlannedAllocator, 60, &demands);
        // Subsistence (index 1) fully met; luxury (index 0) gets the remainder.
        assert_eq!(got[1], 50, "subsistence must be filled first");
        assert_eq!(got[0], 10, "luxury gets only the leftover");
    }

    #[test]
    fn priority_starves_lowest_tier_when_budget_tight() {
        let demands = [
            (PriorityTier::Subsistence, 40),
            (PriorityTier::Basic, 40),
            (PriorityTier::Luxury, 40),
        ];
        let got = allocate_by_priority(&PlannedAllocator, 50, &demands);
        assert_eq!(got[0], 40, "subsistence fully met");
        assert_eq!(got[1], 10, "basic gets remainder");
        assert_eq!(got[2], 0, "luxury starved");
    }

    #[test]
    fn priority_zero_budget_grants_nothing() {
        let demands = [(PriorityTier::Subsistence, 40)];
        assert_eq!(allocate_by_priority(&PlannedAllocator, 0, &demands), vec![0]);
    }

    proptest! {
        /// When budget and demand are non-negative, allocation never exceeds budget.
        #[test]
        fn capitalist_allocator_never_exceeds_budget(
            budget in 0i64..,
            demand in 0i64..,
        ) {
            let allocated = CapitalistAllocator.allocate(budget, demand);
            prop_assert!(allocated <= budget, "allocated {allocated} > budget {budget}");
        }

        /// Allocation is always non-negative for any inputs.
        #[test]
        fn capitalist_allocator_never_negative(
            budget in any::<i64>(),
            demand in any::<i64>(),
        ) {
            let allocated = CapitalistAllocator.allocate(budget, demand);
            prop_assert!(allocated >= 0, "allocated {allocated} is negative");
        }

        /// Priority allocation never distributes more than the budget in total.
        #[test]
        fn priority_total_within_budget(
            budget in 0i64..1_000_000,
            d0 in 0i64..100_000,
            d1 in 0i64..100_000,
            d2 in 0i64..100_000,
        ) {
            let demands = [
                (PriorityTier::Subsistence, d0),
                (PriorityTier::Basic, d1),
                (PriorityTier::Luxury, d2),
            ];
            let got = allocate_by_priority(&PlannedAllocator, budget, &demands);
            let total: i64 = got.iter().sum();
            prop_assert!(total <= budget, "total {total} > budget {budget}");
            prop_assert!(got.iter().all(|&g| g >= 0), "negative allocation");
        }
    }
}
