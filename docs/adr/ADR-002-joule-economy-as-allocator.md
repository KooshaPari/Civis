# ADR-002: Joule Economy as Pluggable Resource Allocator

**Date:** 2026-02-21

**Status:** ACCEPTED

**Author:** CIV Economy & Venture Integration Team

---

## Context

CIV's economy module (CIV-0100) implements a **ledger-based market model** with supply/demand, transfers, and conservation invariants. Separately, Venture platform (TRACK_B_TREASURY_COMPLIANCE_SPEC) implements a **spend-quota model** for controlling AI agent resource consumption.

Question: How should CIV's economy allocate resources (energy, goods) in a way that:
1. Maintains conservation invariants within CIV
2. Integrates with Venture's spend-quota enforcement
3. Allows alternative allocation strategies (market, plan-based, hybrid joule-economy)

The **joule-economy** is a novel allocation mechanism proposed for CIV: agents accumulate "joules" (a synthetic currency representing work capacity), allocate them across goals, and the economy respects these allocations while maintaining conservation.

## Decision

Implement the CIV economy as a **pluggable allocator interface** with multiple backend strategies:

```rust
// In crate/economy/src/lib.rs

pub trait Allocator: Send + Sync {
    /// Allocate resources given agent state, constraints, and current supplies
    fn allocate(
        &self,
        agent: &Actor,
        supply: &SupplyState,
        constraints: &AllocationConstraints,
    ) -> Result<AllocationDecision>;

    /// Validate allocation respects conservation invariants
    fn validate(&self, allocation: &AllocationDecision) -> bool;
}

// Three implementations available:

pub struct MarketAllocator {
    /// Traditional supply/demand market clearing
    /// Price discovery, competitive bidding, exchange
}

pub struct PlanAllocator {
    /// Central planner model
    /// Pre-determined allocations, quotas, directives
}

pub struct JouleAllocator {
    /// Joule-economy model (agent-centric)
    /// Agents accumulate joules (work capacity)
    /// Allocate joules across goals
    /// Economy matches joules to supplies
}

// Registry pattern: select allocator at runtime
pub struct EconomyEngine {
    allocator: Box<dyn Allocator>,
}

impl EconomyEngine {
    pub fn new(allocator: Box<dyn Allocator>) -> Self {
        Self { allocator }
    }
}
```

## Rationale

### 1. CIV Design Freedom
CIV's economy should be **experimentable**. Different allocation strategies may produce different emergent behaviors:
- Markets favor efficient producers; inequality grows
- Plans favor equity; innovation may suffer
- Joule-economy balances autonomy and conservation

By making allocators pluggable, we can:
- Run A/B tests (same simulation state, different allocators)
- Hybrid experiments (market for some goods, plan for others)
- Quickly iterate on novel allocators

### 2. Venture Integration
Venture's spend-quota is orthogonal to CIV's allocation:
- **Venture controls:** Total budget, spend velocity, policy guardrails
- **CIV controls:** How agents allocate within approved budget

Making allocators pluggable allows:
- Venture's `civ.policy.evaluate()` tool to work with any allocator
- Cost model (P1-4) to remain allocator-agnostic
- Future integration points (e.g., Venture directs which allocator to use)

### 3. Conservation & Determinism
All allocators must satisfy the same **conservation invariant**:
```
supply_in + reserves_in - losses - consumption - reserves_out = delta_stock
```

By enforcing this via the `validate()` method:
- All allocators are deterministic (same state → same allocation)
- CIV-0104 (Minimal Constraint Set Theorem) applies uniformly
- Audit trail shows allocator decisions, not just outcomes

### 4. Code Reuse
MarketAllocator and PlanAllocator may share:
- Price indexing logic
- Quota tracking
- Conservation validator

Joule-specific logic stays isolated in JouleAllocator.

## Consequences

### Positive
- CIV economy can evolve without breaking Venture integration
- Experimental allocator strategies don't risk production
- Clear separation: Venture controls budget, CIV controls allocation strategy
- Easy to test each allocator independently

### Negative
- More complex codebase (trait + multiple impls)
- Allocator changes must preserve conservation invariant
- Benchmark overhead: trait dispatch slower than monolithic function
- Testing must cover all allocators (3x test burden)

## Alternatives Considered

### A1: Single Market Allocator Only
**Pros:** Simplest implementation
**Cons:** No way to experiment with alternative strategies; may not match CIV design intent
**Rejected:** Reduces CIV's expressiveness

### A2: Joule Economy Only (No Pluggability)
**Pros:** Focused design, less complexity
**Cons:** Can't compare with market/plan baselines; no flexibility for Venture to choose
**Rejected:** Loses experimental advantage

### A3: Allocator Config in Spec Bundle
**Pros:** Venture can select allocator per simulation run
**Cons:** Requires spec bundle versioning; more infrastructure
**Deferred:** Could add in future (P2 polish phase)

## Implementation

### Phase 1 (P0)

Implement `trait Allocator` and `MarketAllocator`:
- Supply/demand price discovery
- Competitive bidding model
- Full conservation invariant validation

```rust
pub struct MarketAllocator {
    price_index: HashMap<GoodId, Price>,
    demand_queue: VecDeque<(ActorId, Good, Quantity)>,
}

impl Allocator for MarketAllocator {
    fn allocate(
        &self,
        agent: &Actor,
        supply: &SupplyState,
        constraints: &AllocationConstraints,
    ) -> Result<AllocationDecision> {
        // Price discovery based on supply/demand
        // Allocate to agent via clearing price
        // Check conservation
    }

    fn validate(&self, allocation: &AllocationDecision) -> bool {
        // Sum of allocations ≤ supply
        // Conservation equation holds
    }
}
```

### Phase 2 (P1)

Add `PlanAllocator` and `JouleAllocator`:
- Plan: quota assignments, central directives
- Joule: joule accumulation, goal-based allocation

### Phase 3 (P2)

Add **allocator selection** via spec bundle:
```yaml
economy:
  allocator: "market"  # or "plan" or "joule"
  allocator_params:
    market:
      price_discovery_method: "clearinghouse"
    joule:
      joule_accumulation_rate: 10
```

## Validation Commands

```bash
# Test all allocators
cargo test --package economy allocator::tests

# Benchmark allocators (market vs plan vs joule)
cargo bench --package economy

# Run determinism tests with each allocator
cargo test --package economy determinism -- --nocapture
```

## Traceability

- **Spec:** CIV-0100 (Economy Spec v1) — allocator interface
- **Cross-Track:** TRACK_B_TREASURY_COMPLIANCE_SPEC (Venture spend control) — orthogonal
- **Theory:** CIV-0104 (Minimal Constraint Set Theorem) — conservation invariant
- **Integration:** P1-2 (civ.policy.evaluate tool) — allocator-agnostic

## References

- **CIV-0100:** Economy Spec v1 (ledger, market model)
- **CIV-0104:** Minimal Constraint Set Theorem (conservation)
- **TRACK_B:** Treasury & Compliance Spec (Venture spend control)
- **NEXT_STEPS.md:** P1-4 Cost Model task

---

**Decision Delta:**
- Economy module uses pluggable `Allocator` trait
- Three implementations: Market, Plan, Joule
- All must satisfy conservation invariant
- Selection via spec bundle (Phase 3)

**Allocator Interface Stability:** PUBLIC (external integrations may depend on this)

**Review Date:** 2026-05-21 (after P0 + Joule impl. in P1)
