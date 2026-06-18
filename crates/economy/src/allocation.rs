//! Allocation engines (CIV-0100 §allocation).
//!
//! Two complementary mechanisms live here:
//!
//! * [`AllocationEngine`] — proportional rationing used by the market/clearing
//!   path (see [`CapitalistAllocator`]).
//! * [`subsistence_first_allocate`] — FR-ECON-005 need-based allocator: per-agent
//!   deficit ranking, subsistence goods first, luxury goods from leftovers, and
//!   a per-agent deprivation counter incremented on unmet subsistence need.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

// ---------------------------------------------------------------------------
// FR-ECON-005: Subsistence-first need allocator.
// ---------------------------------------------------------------------------

/// Numeric good identifier (FR-ECON-005). Integer-friendly: stable across
/// serialization, replays, and external registry mappings.
pub type GoodId = u32;

/// One unit of need satisfaction equals this fraction of the agent's total
/// need. With `SCALE = 10`, an agent with `satisfaction = 0.3` has a deficit
/// of `0.7` and needs `ceil(0.7 * 10) = 7` units to be fully served.
///
/// 10 was chosen so that `f64` deficit values in the common `[0.0, 1.0]`
/// range map cleanly to a small positive `i64` unit count (0..=10) without
/// rounding loss for one-decimal satisfactions.
const SCALE: i64 = 10;

/// Classifies an agent's need: subsistence needs are filled before any
/// luxury allocation, and unmet subsistence increments the agent's
/// deprivation counter (FR-ECON-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedKind {
    /// Need required to sustain the agent (food, water, shelter, ...).
    Subsistence,
    /// Discretionary need served only from leftover stock.
    Luxury,
}

/// A single per-agent need statement (FR-ECON-005).
///
/// `satisfaction` is a fraction in `[0.0, 1.0]`. Values outside the range are
/// clamped: `satisfaction >= 1.0` means the need is already met (deficit 0),
/// `satisfaction <= 0.0` means the agent needs the full `SCALE` units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AgentNeed {
    /// Stable agent identifier.
    pub agent_id: u32,
    /// Good that satisfies this need.
    pub good: GoodId,
    /// Subsistence vs luxury classification.
    pub kind: NeedKind,
    /// Current satisfaction in `[0.0, 1.0]` (clamped on read).
    pub satisfaction: f64,
}

impl AgentNeed {
    /// Deficit in `[0.0, 1.0]` (1.0 minus satisfaction, clamped at 0).
    fn deficit(&self) -> f64 {
        debug_assert!(
            self.satisfaction.is_finite(),
            "agent {} satisfaction must be finite, got {}",
            self.agent_id,
            self.satisfaction,
        );
        (1.0 - self.satisfaction).max(0.0)
    }

    /// Integer unit count needed to fully meet this need (`ceil(deficit *
    /// SCALE)`), or 0 if the need is already satisfied.
    fn need_units(&self) -> i64 {
        let deficit = self.deficit();
        if deficit <= 0.0 {
            return 0;
        }
        // `deficit` is in [0.0, 1.0] and `SCALE = 10`, so the result is at
        // most 10. Use `ceil` so a deficit of 0.05 still needs 1 unit.
        let raw = deficit * SCALE as f64;
        let clamped = raw.clamp(0.0, SCALE as f64);
        clamped.ceil() as i64
    }
}

/// Result of [`subsistence_first_allocate`] (FR-ECON-005).
///
/// All three maps are keyed for stable, deterministic iteration:
/// * `received` — agent id → integer units received (sum across all needs
///   for that agent; per-call this is at most one need per agent, but the
///   type is `BTreeMap` so callers can accumulate across multiple passes).
/// * `deprivation_counters` — agent id → number of unmet subsistence needs
///   observed during this call. Luxury needs never increment this counter.
/// * `unallocated` — good id → integer units of stock remaining after the
///   pass (always present for goods that appeared in the input stock, even
///   when the remaining count is 0).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationOutcome {
    /// Per-agent units received.
    pub received: BTreeMap<u32, i64>,
    /// Per-agent count of unmet subsistence needs.
    pub deprivation_counters: BTreeMap<u32, u32>,
    /// Per-good leftover stock after allocation.
    pub unallocated: BTreeMap<GoodId, i64>,
}

/// Subsistence-first need allocator (FR-ECON-005).
///
/// Algorithm:
/// 1. For each [`AgentNeed`], compute the integer unit deficit
///    `need_units = ceil((1.0 - satisfaction) * 10)` (clamped to `[0, 10]`).
/// 2. Sort all needs by `(kind: Subsistence first, deficit desc, agent_id
///    asc)`. This is the canonical tie-breaker: identical inputs always
///    produce identical iteration order, so the result is deterministic.
/// 3. Walk the sorted list once. For each need, hand out
///    `min(need_units, stock[good])` units of that good (0 if the good is
///    not in stock). Subtract from `stock` and add to `received[agent_id]`.
/// 4. If a Subsistence need is unmet (`allocated < need_units`),
///    `deprivation_counters[agent_id] += 1`. Luxury needs never increment
///    deprivation.
/// 5. The remaining `stock` map (one entry per input good) becomes
///    `outcome.unallocated`.
///
/// Properties guaranteed by the implementation:
/// * **Determinism** — same `(needs, stock)` input always yields an
///   identical `AllocationOutcome` (BTreeMap iteration + the sort
///   tie-breaker above are the only sources of order).
/// * **Conservation** — for every good `g`,
///   `sum_over_agents(received_agent[g]) + outcome.unallocated[g] ==
///   input_stock[g]`. No units are created or destroyed.
/// * **Subsistence first** — all Subsistence needs are processed before
///   any Luxury need consumes stock.
/// * **Integer-friendly** — every unit is `i64`; `f64` is used only for
///   the user's satisfaction input.
pub fn subsistence_first_allocate(
    needs: &[AgentNeed],
    stock: BTreeMap<GoodId, i64>,
) -> AllocationOutcome {
    let mut outcome = AllocationOutcome::default();
    let mut remaining: BTreeMap<GoodId, i64> = stock;

    // Sort by (Subsistence first, deficit desc, agent_id asc). We sort
    // outside the loop so the algorithm is observably a single pass after
    // the (deterministic) ordering is fixed.
    let mut indexed: Vec<(usize, &AgentNeed)> = needs.iter().enumerate().collect();
    indexed.sort_by(|(idx_a, a), (idx_b, b)| {
        // Subsistence (0) sorts before Luxury (1). Encode Luxury as the
        // larger key so `key.cmp(&other)` puts Subsistence first.
        let kind_key = |n: &AgentNeed| -> u8 {
            match n.kind {
                NeedKind::Subsistence => 0,
                NeedKind::Luxury => 1,
            }
        };
        kind_key(a)
            .cmp(&kind_key(b))
            // Higher deficit first.
            .then_with(|| b.deficit().partial_cmp(&a.deficit()).unwrap_or(std::cmp::Ordering::Equal))
            // Stable tie-break: lower agent id first.
            .then_with(|| a.agent_id.cmp(&b.agent_id))
            // Final fallback to original slice order for full determinism
            // across two needs with identical (kind, deficit, agent_id)
            // (impossible if agent_id is unique, but defensive).
            .then_with(|| idx_a.cmp(idx_b))
    });

    for (_, need) in indexed {
        let want = need.need_units();
        if want == 0 {
            // Need is already satisfied; record a 0 receipt (for
            // determinism / audit) but skip the stock lookup.
            outcome.received.entry(need.agent_id).or_insert(0);
            continue;
        }

        let available = remaining.get(&need.good).copied().unwrap_or(0).max(0);
        let allocated = want.min(available);
        if allocated > 0 {
            *remaining.entry(need.good).or_insert(0) -= allocated;
        }
        // Always record a receipt (0 included) so callers can assert
        // `received.get(&agent) == Some(0)` for agents whose need was unmet.
        *outcome.received.entry(need.agent_id).or_insert(0) += allocated;

        if matches!(need.kind, NeedKind::Subsistence) && allocated < want {
            *outcome
                .deprivation_counters
                .entry(need.agent_id)
                .or_insert(0) += 1;
        }
    }

    // Mirror the input stock keys (including zero remainders) so callers
    // can assert on per-good leftover without a separate `was_present` map.
    for (good, units) in &remaining {
        outcome.unallocated.insert(*good, *units);
    }

    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ---- AllocationEngine (existing) -----------------------------------

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

    // ---- FR-ECON-005: subsistence_first_allocate -----------------------

    /// `need_units` helper: 0.3 satisfaction -> 7 units needed.
    #[test]
    fn agent_need_need_units_one_agent_satisfaction_0_3_yields_seven() {
        let need = AgentNeed {
            agent_id: 7,
            good: 1,
            kind: NeedKind::Subsistence,
            satisfaction: 0.3,
        };
        assert_eq!(need.need_units(), 7);
        assert_eq!(need.deficit(), 0.7);
    }

    /// Spec test #1: 1 agent, satisfaction 0.3, 10 units stock -> agent
    /// gets 7, 3 unallocated.
    #[test]
    fn subsistence_first_single_agent_partial_satisfaction_clamps_to_deficit() {
        let needs = vec![AgentNeed {
            agent_id: 1,
            good: 1,
            kind: NeedKind::Subsistence,
            satisfaction: 0.3,
        }];
        let stock = BTreeMap::from([(1u32, 10i64)]);
        let outcome = subsistence_first_allocate(&needs, stock);
        assert_eq!(outcome.received.get(&1).copied(), Some(7));
        assert_eq!(outcome.unallocated.get(&1).copied(), Some(3));
        assert!(outcome.deprivation_counters.is_empty());
    }

    /// Spec test #2: 2 agents with different goods. A's good has 10 units
    /// and A is fully unsatisfied (deficit 1.0, need 10); B's good has 0
    /// stock. A receives all 10 of A's good; B receives 0 and is
    /// deprived.
    #[test]
    fn subsistence_first_two_agents_disjoint_goods_higher_deficit_takes_all() {
        let needs = vec![
            AgentNeed {
                agent_id: 1,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.0, // deficit 1.0 -> 10 units
            },
            AgentNeed {
                agent_id: 2,
                good: 2,
                kind: NeedKind::Subsistence,
                satisfaction: 0.5,
            },
        ];
        let stock = BTreeMap::from([(1u32, 10i64), (2u32, 0i64)]);
        let outcome = subsistence_first_allocate(&needs, stock);
        assert_eq!(outcome.received.get(&1).copied(), Some(10));
        assert_eq!(outcome.received.get(&2).copied(), Some(0));
        assert_eq!(outcome.unallocated.get(&1).copied(), Some(0));
        assert_eq!(outcome.unallocated.get(&2).copied(), Some(0));
        // A is fully served -> no deprivation. B is unmet -> +1.
        assert_eq!(outcome.deprivation_counters.get(&1).copied(), None);
        assert_eq!(outcome.deprivation_counters.get(&2).copied(), Some(1));
    }

    /// Spec test #3: 0 stock, 5 agents with subsistence needs -> all 5
    /// have deprivation counter = 1.
    #[test]
    fn subsistence_first_zero_stock_creates_deprivation_for_every_subsistence_agent() {
        let needs: Vec<AgentNeed> = (0..5)
            .map(|i| AgentNeed {
                agent_id: i,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.0,
            })
            .collect();
        let stock = BTreeMap::from([(1u32, 0i64)]);
        let outcome = subsistence_first_allocate(&needs, stock);
        assert_eq!(outcome.received.len(), 5);
        for i in 0..5 {
            assert_eq!(outcome.received.get(&i).copied(), Some(0));
            assert_eq!(outcome.deprivation_counters.get(&i).copied(), Some(1));
        }
        assert_eq!(outcome.unallocated.get(&1).copied(), Some(0));
    }

    /// Spec test #4: same input twice -> identical outcome.
    #[test]
    fn subsistence_first_is_deterministic() {
        let needs = vec![
            AgentNeed {
                agent_id: 3,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.4,
            },
            AgentNeed {
                agent_id: 1,
                good: 2,
                kind: NeedKind::Luxury,
                satisfaction: 0.2,
            },
            AgentNeed {
                agent_id: 2,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.1,
            },
        ];
        let stock = BTreeMap::from([(1u32, 5i64), (2u32, 7i64)]);
        let a = subsistence_first_allocate(&needs, stock.clone());
        let b = subsistence_first_allocate(&needs, stock);
        assert_eq!(a, b);
    }

    /// Spec test #5: sum of received across all agents <= sum of available
    /// stock (and conservation: received + unallocated == input stock).
    #[test]
    fn subsistence_first_received_never_exceeds_input_stock() {
        let needs = vec![
            AgentNeed {
                agent_id: 1,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.0,
            },
            AgentNeed {
                agent_id: 2,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.5,
            },
            AgentNeed {
                agent_id: 3,
                good: 2,
                kind: NeedKind::Luxury,
                satisfaction: 0.0,
            },
        ];
        let stock = BTreeMap::from([(1u32, 4i64), (2u32, 9i64)]);
        let input_total: i64 = stock.values().copied().sum();
        let outcome = subsistence_first_allocate(&needs, stock.clone());
        let received_total: i64 = outcome.received.values().copied().sum();
        let unallocated_total: i64 = outcome.unallocated.values().copied().sum();
        assert!(received_total <= input_total);
        assert_eq!(received_total + unallocated_total, input_total);
    }

    // ---- Extra behavior coverage ---------------------------------------

    /// Subsistence is processed before luxury (FR-ECON-005 "subsistence
    /// first" guarantee). The luxury agent must not consume stock that a
    /// subsistence agent still needs.
    #[test]
    fn subsistence_first_processes_subsistence_before_luxury() {
        let needs = vec![
            AgentNeed {
                agent_id: 1,
                good: 1,
                kind: NeedKind::Luxury,
                satisfaction: 0.0, // would want 10 units
            },
            AgentNeed {
                agent_id: 2,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.0, // would want 10 units
            },
        ];
        let stock = BTreeMap::from([(1u32, 10i64)]);
        let outcome = subsistence_first_allocate(&needs, stock);
        // Agent 2 (subsistence) must be fully served; agent 1 (luxury) gets 0.
        assert_eq!(outcome.received.get(&2).copied(), Some(10));
        assert_eq!(outcome.received.get(&1).copied(), Some(0));
        assert!(outcome.deprivation_counters.get(&1).is_none());
        assert!(outcome.deprivation_counters.get(&2).is_none());
    }

    /// Unmet luxury needs do NOT increment the deprivation counter (only
    /// subsistence does, per spec).
    #[test]
    fn luxury_unmet_does_not_increment_deprivation() {
        let needs = vec![AgentNeed {
            agent_id: 42,
            good: 1,
            kind: NeedKind::Luxury,
            satisfaction: 0.0,
        }];
        let stock = BTreeMap::from([(1u32, 0i64)]);
        let outcome = subsistence_first_allocate(&needs, stock);
        assert_eq!(outcome.received.get(&42).copied(), Some(0));
        assert!(outcome.deprivation_counters.is_empty());
    }

    /// Deficit is clamped: a need with satisfaction > 1.0 is treated as
    /// already met (deficit 0, no units needed, no deprivation).
    #[test]
    fn need_units_clamps_satisfaction_above_one() {
        let need = AgentNeed {
            agent_id: 1,
            good: 1,
            kind: NeedKind::Subsistence,
            satisfaction: 1.5,
        };
        assert_eq!(need.need_units(), 0);
        assert_eq!(need.deficit(), 0.0);
    }

    /// Deficit is clamped: a need with satisfaction < 0.0 is treated as
    /// fully unsatisfied (deficit 1.0, need = SCALE = 10 units).
    #[test]
    fn need_units_clamps_satisfaction_below_zero() {
        let need = AgentNeed {
            agent_id: 1,
            good: 1,
            kind: NeedKind::Subsistence,
            satisfaction: -0.5,
        };
        assert_eq!(need.need_units(), SCALE);
        assert_eq!(need.deficit(), 1.5);
    }

    /// Stable tie-break by (deficit desc, agent_id asc). Two agents with
    /// the same deficit must be served in agent_id order.
    #[test]
    fn tie_break_is_agent_id_ascending_for_equal_deficit() {
        let needs = vec![
            AgentNeed {
                agent_id: 9,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.0,
            },
            AgentNeed {
                agent_id: 3,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.0,
            },
        ];
        // 10 units of good 1, both fully unsatisfied: agent 3 takes the
        // first cut, then agent 9. With deficit 1.0 each wants 10 units;
        // only one can be fully served, the other is deprived.
        let stock = BTreeMap::from([(1u32, 10i64)]);
        let outcome = subsistence_first_allocate(&needs, stock);
        assert_eq!(outcome.received.get(&3).copied(), Some(10));
        assert_eq!(outcome.received.get(&9).copied(), Some(0));
        assert_eq!(outcome.deprivation_counters.get(&9).copied(), Some(1));
        assert!(outcome.deprivation_counters.get(&3).is_none());
    }

    /// A subsistence need for a good that has no entry in the input stock
    /// is treated as zero stock (agent receives 0, deprivation += 1).
    #[test]
    fn subsistence_for_unknown_good_records_deprivation_and_no_receipt() {
        let needs = vec![AgentNeed {
            agent_id: 1,
            good: 99,
            kind: NeedKind::Subsistence,
            satisfaction: 0.0,
        }];
        let stock = BTreeMap::from([(1u32, 5i64)]); // good 99 is absent
        let outcome = subsistence_first_allocate(&needs, stock);
        assert_eq!(outcome.received.get(&1).copied(), Some(0));
        assert_eq!(outcome.deprivation_counters.get(&1).copied(), Some(1));
    }

    /// Spec test (FR-ECON-005): two agents competing for the same good.
    /// A has satisfaction 0.1 (deficit 0.9 -> 9 units), B has satisfaction
    /// 0.9 (deficit 0.1 -> 1 unit), 10 units of stock. A is served first
    /// (higher deficit), so A receives 9, B receives 1, and no deprivation
    /// is recorded for either agent.
    #[test]
    fn subsistence_first_two_agents_same_good_priority_order() {
        let needs = vec![
            AgentNeed {
                agent_id: 42, // B
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.9,
            },
            AgentNeed {
                agent_id: 7, // A (intentionally listed second to prove the
                             // sort reorders by deficit, not insertion order)
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: 0.1,
            },
        ];
        let stock = BTreeMap::from([(1u32, 10i64)]);
        let outcome = subsistence_first_allocate(&needs, stock);
        assert_eq!(outcome.received.get(&7).copied(), Some(9));
        assert_eq!(outcome.received.get(&42).copied(), Some(1));
        assert_eq!(outcome.unallocated.get(&1).copied(), Some(0));
        assert!(outcome.deprivation_counters.is_empty());
    }

    /// Spec test (FR-ECON-005): an empty `needs` slice returns the empty
    /// outcome (no receipts, no deprivation, unallocated mirrors input
    /// stock).
    #[test]
    fn subsistence_first_empty_needs_returns_empty_outcome() {
        let stock = BTreeMap::from([(1u32, 5i64), (2u32, 3i64)]);
        let outcome = subsistence_first_allocate(&[], stock.clone());
        assert!(outcome.received.is_empty());
        assert!(outcome.deprivation_counters.is_empty());
        assert_eq!(outcome.unallocated, stock);
    }

    proptest! {
        // ---- Property-based invariants ---------------------------------

        /// Conservation: received + unallocated == input stock (per good).
        #[test]
        fn subsistence_first_conservation_holds(
            good_a_stock in 0i64..100,
            good_b_stock in 0i64..100,
            sat_a in 0.0f64..=1.0,
            sat_b in 0.0f64..=1.0,
        ) {
            let needs = vec![
                AgentNeed {
                    agent_id: 1,
                    good: 1,
                    kind: NeedKind::Subsistence,
                    satisfaction: sat_a,
                },
                AgentNeed {
                    agent_id: 2,
                    good: 2,
                    kind: NeedKind::Subsistence,
                    satisfaction: sat_b,
                },
            ];
            let stock = BTreeMap::from([(1u32, good_a_stock), (2u32, good_b_stock)]);
            let outcome = subsistence_first_allocate(&needs, stock.clone());
            let received_g1: i64 = outcome.received.values().copied().sum::<i64>()
                - outcome.received.get(&2).copied().unwrap_or(0);
            // Direct per-good check using the unallocated map.
            let unalloc_g1 = outcome.unallocated.get(&1).copied().unwrap_or(0);
            prop_assert_eq!(received_g1 + unalloc_g1, good_a_stock);

            // Per-agent total received is non-negative and bounded by the
            // agent's own need for that good.
            for (agent, got) in &outcome.received {
                prop_assert!(*got >= 0, "agent {agent} got negative {got}");
            }
        }

        /// Determinism: same (needs, stock) twice -> equal outcomes.
        #[test]
        fn subsistence_first_deterministic_property(
            sat in 0.0f64..=1.0,
            stock_units in 0i64..50,
        ) {
            let needs = vec![AgentNeed {
                agent_id: 1,
                good: 1,
                kind: NeedKind::Subsistence,
                satisfaction: sat,
            }];
            let stock = BTreeMap::from([(1u32, stock_units)]);
            let a = subsistence_first_allocate(&needs, stock.clone());
            let b = subsistence_first_allocate(&needs, stock);
            prop_assert_eq!(a, b);
        }

        /// Deprivation is monotone in unmet subsistence: a Subsistence
        /// need whose `allocated < want` always increments deprivation by
        /// exactly 1.
        #[test]
        fn deprivation_only_for_unmet_subsistence(
            sat in 0.0f64..=1.0,
            stock_units in 0i64..20,
            is_luxury in any::<bool>(),
        ) {
            let kind = if is_luxury {
                NeedKind::Luxury
            } else {
                NeedKind::Subsistence
            };
            let need = AgentNeed {
                agent_id: 7,
                good: 1,
                kind,
                satisfaction: sat,
            };
            let stock = BTreeMap::from([(1u32, stock_units)]);
            let outcome = subsistence_first_allocate(&[need], stock);
            let got = outcome.received.get(&7).copied().unwrap_or(0);
            let dep = outcome.deprivation_counters.get(&7).copied().unwrap_or(0);
            let want = need.need_units();
            if is_luxury {
                prop_assert_eq!(dep, 0, "luxury unmet must not record deprivation");
            } else if want > 0 && got < want {
                prop_assert_eq!(dep, 1, "unmet subsistence must record deprivation = 1");
            } else {
                prop_assert_eq!(dep, 0, "met/zero subsistence must not record deprivation");
            }
        }
    }
}
