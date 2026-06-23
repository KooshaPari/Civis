//! Allocation engines (CIV-0100 §allocation). Engine wiring lands in a follow-up.

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
    }
}
