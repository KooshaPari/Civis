# Merged Fragmented Markdown

## Source: adr/ADR-001-rust-crate-structure.md

# ADR-001: Rust Crate Structure — Workspace with Focused Modules

**Date:** 2026-02-21

**Status:** ACCEPTED

**Author:** CIV Architecture Team

---

## Context

The CIV city simulation engine requires modular architecture supporting multiple distinct simulation domains:
- Economy (ledger, market clearing)
- Spatial representation (LOD, districts, actors)
- Climate (energy accounting)
- Institutions & actors (lifecycle, state machines)
- Geopolitics (conflict, diplomacy)
- Social dynamics (ideology, health, insurgency)

A monolithic crate would couple these domains unnecessarily. Separate crates enable:
- Independent testing and iteration
- Clear boundary enforcement (via `tach.toml`)
- Easier integration with Venture platform (each crate has well-defined API)
- Parallel development across teams

## Decision

Organize CIV as a **Rust workspace** with the following focused crates:

```
civ/
  Cargo.toml (workspace root)

  crates/
    engine/          # Core simulation loop (CIV-0001)
                     # - Tick-based state machine
                     # - Event ordering logic
                     # - Determinism/replay contract

    economy/         # Economy module (CIV-0100)
                     # - Ledger, market clearing, transfers
                     # - Depends on: engine

    spatial/         # Spatial & LOD (CIV-0101)
                     # - District/agent representation
                     # - LOD transitions
                     # - Neighbor queries
                     # - Depends on: engine

    climate/         # Climate module (CIV-0102)
                     # - Energy accounting, conservation equation
                     # - Weather events
                     # - Depends on: engine, economy (energy supply)

    actors/          # Institutions & citizen lifecycle (CIV-0103)
                     # - Actor state machines
                     # - Institution formation/dissolution
                     # - Time-series metrics
                     # - Depends on: engine, economy, spatial

    policy/          # Policy evaluation framework (CIV-0100, CIV-0104)
                     # - Policy.evaluate() interface
                     # - Constraint solver (minimal constraint set theorem)
                     # - Determinism validator
                     # - Depends on: engine, economy, actors

    geo/             # Geopolitics (CIV-0105)
                     # - War, diplomacy, shadow networks
                     # - Conflict resolution
                     # - Depends on: engine, actors, spatial

    social/          # Social dynamics (CIV-0106)
                     # - Ideology, health, insurgency
                     # - Cohesion metrics
                     # - Depends on: engine, actors, spatial

    metrics/         # Observability (Venture integration)
                     # - Event recording
                     # - Time-series storage
                     # - Audit trail hooks
                     # - Depends on: engine, economy, actors, climate

    server/          # HTTP/gRPC server (Venture integration)
                     # - EventEnvelopeV1 emission
                     # - Policy.evaluate tool endpoint
                     # - Artifact export pipeline
                     # - Depends on: (all modules above)
```

## Dependency Graph (Enforced via `tach.toml`)

```
server  ─────┬─ metrics
             ├─ economy  ─────┬─ engine
             ├─ climate  ─────┤─ economy
             ├─ actors   ─────┼─ spatial ─── engine
             ├─ geo      ─────┤─ policy
             ├─ social   ──────┴─ engine
             └─ policy

            (All non-root crates depend on engine)
```

## Rationale

1. **Clear Separation of Concerns**
   - Each crate has a single, well-defined responsibility
   - Easier to test and reason about in isolation
   - Matches spec organization (CIV-0100, CIV-0101, etc.)

2. **Boundary Enforcement**
   - Use `tach.toml` to enforce dependency graph
   - Prevents accidental coupling (e.g., economy should not depend on geo)
   - CI rejects violations

3. **Parallel Development**
   - Teams can work on crates independently
   - Reduced merge conflicts on shared files
   - Clear handoff points (crate APIs)

4. **Venture Integration**
   - `server/` crate provides single integration point
   - EventEnvelopeV1, tool endpoints, artifact export all in one place
   - Easier to version and test Venture contracts

5. **Performance & Build Time**
   - Only build changed crates in incremental builds
   - Team members can work on one domain without pulling all code

## Consequences

### Positive
- Reduced cyclomatic complexity per crate
- Clearer ownership and accountability
- Easier to onboard new team members to specific domains
- Boundary violations caught at compile time

### Negative
- More complex workspace management (multiple Cargo.tomls)
- Cross-crate API changes require coordination
- Workspace build is slower than monolithic initially (improves with incremental builds)
- More CI/CD configuration

## Alternatives Considered

### A1: Monolithic Crate
**Pros:** Simpler build, shared internal state
**Cons:** Tight coupling, hard to parallelize, cyclomatic complexity grows unbounded
**Rejected:** Does not scale with 8 spec modules

### A2: Separate Repos
**Pros:** Maximum autonomy
**Cons:** Version coordination nightmare, harder to ensure determinism across modules
**Rejected:** Cross-module determinism requires synchronized ticks

### A3: Flat Workspace (No Sub-Modules)
**Pros:** Simple structure
**Cons:** No boundary enforcement, all crates at same level
**Rejected:** Violates dependency ordering (e.g., climate should not depend on geo)

## Implementation

### Crate Structure Template

Each crate should follow:
```
crates/{name}/
  Cargo.toml
  src/
    lib.rs         # Public API
    module1.rs     # Implementation
    ...
  tests/
    integration_tests.rs
  README.md        # Crate-specific docs
```

### Workspace Configuration

**Root Cargo.toml:**
```toml
[workspace]
members = [
  "crates/engine",
  "crates/economy",
  "crates/spatial",
  # ... etc
]
```

### Boundary Enforcement

**tach.toml** (via [Tach](https://www.notion.so/Tach-Python-42da5cfc6bda4a098e39b88e2a34b86f)):
```toml
[boundaries]
"engine" = []  # No dependencies
"economy" = ["engine"]
"spatial" = ["engine"]
"climate" = ["engine", "economy"]
# ... full graph
```

Run: `tach check` (pre-commit hook)

## Validation Commands

```bash
# Build all crates
cargo build --workspace

# Test all crates
cargo test --workspace

# Check dependency boundaries
tach check

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all
```

## References

- **CIV-0001:** Core Simulation Loop spec
- **CIV-0100–0106:** Domain-specific specs
- **CLAUDE.md:** Project governance (cross-module determinism)
- **Tach:** Boundary enforcement tool (https://github.com/gauge-sh/tach)

---

**Decision Delta:**
- Workspace-based organization with 9 focused crates
- Dependency graph enforced via `tach.toml`
- Single `server/` crate for Venture integration

**Review Date:** 2026-05-21 (after P0 implementation)


---

## Source: adr/ADR-002-joule-economy-as-allocator.md

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


---

## Source: adr/ADR-003-deterministic-replay.md

# ADR-003: Deterministic Scenario Replay — Mandatory for All Simulation Runs

**Date:** 2026-02-21

**Status:** ACCEPTED

**Author:** CIV Architecture & QA Team

---

## Context

CIV simulations are multi-actor, multi-domain systems with potential for stochasticity:
- Random resource generation (weather, discovery)
- Probabilistic events (disease, rebellion)
- Actor decision randomization (path-finding, trade)

Determinism is critical for:
1. **Auditability:** Reproduce exact simulation run from event log (compliance requirement for Venture)
2. **Debugging:** "Why did city collapse at tick 5000?" → replay with breakpoints
3. **Testing:** Simulation runs must be bit-identical under same seed
4. **Integration:** Venture's deterministic artifact builds depend on CIV being replay-safe

Current risk: Accidental non-determinism from:
- Floating-point arithmetic precision loss
- HashMap/BTreeMap iteration order (Rust std)
- System time leaks
- Uninitialized memory

## Decision

**All CIV simulation logic must be deterministic and replayable.**

Enforce via:

### 1. Mandatory Replay Test for Every Simulation Run

**Code Pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_deterministic_replay() {
        let seed = 12345u64;
        let config = SimulationConfig::default();
        let initial_state = create_test_city();

        // Run 1: Forward simulation
        let (final_state1, events1) = simulate(
            config.clone(),
            initial_state.clone(),
            seed,
            100, // 100 ticks
        ).unwrap();

        // Run 2: Replay from events
        let (final_state2, events2) = replay_from_events(
            config.clone(),
            initial_state.clone(),
            &events1,
        ).unwrap();

        // Assertions
        assert_eq!(final_state1, final_state2, "States differ after replay");
        assert_eq!(events1, events2, "Events differ on replay");
    }
}
```

This test MUST:
- Pass on every commit (CI gate)
- Be part of every integration test suite
- Run with multiple seeds (Monte Carlo verification)

### 2. RNG Seeding & State Logging

**Code Pattern:**
```rust
pub struct SimulationEngine {
    tick: u64,
    rng: ChaCha20Rng,  // Deterministic PRNG (not rand::random!)
    event_log: Vec<Event>,
}

impl SimulationEngine {
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        Self {
            tick: 0,
            rng,
            event_log: vec![],
        }
    }

    /// Every stochastic decision must log state
    pub fn random_choice(&mut self, options: &[T]) -> T {
        let value = self.rng.gen_range(0..options.len());
        self.event_log.push(Event::RngDecision {
            tick: self.tick,
            seed_state_before: self.rng.state(), // Log state
            decision: value,
            seed_state_after: self.rng.state(),
        });
        options[value].clone()
    }
}
```

### 3. No Floating-Point Surprises

**Rules:**
- Use fixed-point arithmetic for money/resources (e.g., `i64` cents instead of `f64` dollars)
- If floating-point required: use `ordered-float` crate for deterministic comparisons
- Never use `f64::NaN` or `-0.0`
- Document all floating-point operations with rationale

**Code Pattern:**
```rust
// Bad: floating-point money
let price: f64 = 12.34;  // Loses precision after many ops

// Good: fixed-point (cents)
let price_cents: i64 = 1234;

// If floats unavoidable:
use ordered_float::OrderedFloat;
let price: OrderedFloat<f64> = OrderedFloat(12.34);
```

### 4. Collection Ordering Guarantees

**Rules:**
- Iterate collections in deterministic order (use `BTreeMap`, not `HashMap`)
- If iteration order matters for events, sort before emitting
- Document collection choice rationale

**Code Pattern:**
```rust
// Bad: HashMap iteration order undefined
let mut goods: HashMap<GoodId, Quantity> = /* ... */;
for (good_id, qty) in &goods {
    // Order is non-deterministic!
    process(good_id, qty);
}

// Good: BTreeMap (sorted by key)
let mut goods: BTreeMap<GoodId, Quantity> = /* ... */;
for (good_id, qty) in &goods {
    // Order guaranteed by sort key
    process(good_id, qty);
}

// Or: collect and sort before iteration
let mut goods: HashMap<GoodId, Quantity> = /* ... */;
let mut items: Vec<_> = goods.into_iter().collect();
items.sort_by_key(|(id, _)| *id);
for (good_id, qty) in items {
    process(&good_id, &qty);
}
```

### 5. System Time Isolation

**Rules:**
- Simulation clock is decoupled from wall-clock time
- No `std::time::SystemTime`, `chrono::Local`, etc. in simulation
- All time is `tick: u64` within simulation engine
- System time only in I/O layer (metrics, logging)

**Code Pattern:**
```rust
// Bad: Leaks wall-clock time
pub fn get_current_time() -> SystemTime {
    SystemTime::now()  // Non-deterministic!
}

// Good: Simulation uses abstract ticks
pub struct SimulationEngine {
    tick: u64,
}

impl SimulationEngine {
    pub fn current_tick(&self) -> u64 {
        self.tick
    }
}

// I/O layer can map ticks to wall-clock for display
fn display_simulation_time(tick: u64) -> String {
    let wall_time = SIMULATION_START + Duration::from_secs(tick * TICK_DURATION_SECS);
    wall_time.to_string()
}
```

### 6. Determinism Validator Tool

**Crate:** `civ/crates/policy` (see ADR-001)

**Interface:**
```rust
pub struct DeterminismValidator {
    expected_events: Vec<Event>,
}

impl DeterminismValidator {
    /// Replay simulation and compare events
    pub fn validate(&self, config: &SimulationConfig, seed: u64) -> Result<()> {
        let (_, actual_events) = simulate(config, seed, 1000)?;
        if self.expected_events != actual_events {
            return Err(DeterminismError::EventMismatch {
                expected_count: self.expected_events.len(),
                actual_count: actual_events.len(),
                first_divergence_tick: /* compute */,
            });
        }
        Ok(())
    }
}
```

**Usage in Tests:**
```rust
#[test]
fn test_determinism_10_runs() {
    let seeds = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    for seed in &seeds {
        let (_, events) = simulate(config, *seed, 1000).unwrap();
        let validator = DeterminismValidator::new(events);
        validator.validate(&config, *seed).unwrap();
    }
}
```

### 7. CI Gate: Determinism Test Required

**In CI/CD pipeline:**
```bash
# Before merge, this must pass:
cargo test --package engine deterministic_replay -- --nocapture --test-threads=1
cargo test --package economy deterministic_replay -- --nocapture --test-threads=1
# ... for all domain crates
```

**Failure blocks merge.**

## Consequences

### Positive
- Simulation is auditable (reproduce any run from event log)
- Debugging is possible (replay with breakpoints, log inspection)
- Venture integration is simplified (deterministic artifacts)
- Testing is reliable (no flaky tests)
- Compliance friendly (full event trail for auditors)

### Negative
- Cannot use `rand::random()` or non-seeded RNGs
- Cannot use `HashMap` or other unordered collections
- Must avoid floating-point arithmetic where possible
- All tests are slower (determinism checks + replays)
- Developers must be careful about hidden non-determinism

## Alternatives Considered

### A1: Optional Determinism (Determinism on Request)
**Pros:** Easier development (no replay overhead)
**Cons:** Non-determinism bugs hide until production; Venture integration broken
**Rejected:** Violates mandatory replay requirement

### A2: External Determinism Wrapper
**Pros:** Simulation code doesn't need to care
**Cons:** Hard to enforce; easy to sneak in non-determinism (system time in sim loop)
**Rejected:** Requires discipline at every layer

## Implementation Phases

### Phase 1 (P0): Foundation
- Implement determinism validator in `policy/` crate
- Add replay tests to `engine/` crate
- Enforce ChaCha20Rng + BTreeMap in domain crates
- CI gate: all determinism tests pass

### Phase 2 (P1): Cross-Crate Verification
- Add determinism tests to all domain crates (economy, actors, geo, social, climate)
- Benchmark replay overhead (target: <5% perf hit)
- Document every RNG usage with rationale

### Phase 3 (P2): Venture Integration
- Determinism guarantees published in spec bundle
- Artifact builds use determinism validator as precondition
- Compliance audits reference determinism proofs

## Validation Commands

```bash
# Run all determinism tests
cargo test --workspace deterministic_replay --nocapture

# Check for non-deterministic patterns (linter)
cargo clippy --workspace -- -W non-determinism

# Benchmark determinism overhead
cargo bench --package engine -- deterministic_replay
```

## Traceability

- **Spec:** CIV-0001 (Core Simulation Loop) — determinism contract
- **Spec:** CIV-0104 (Minimal Constraint Set Theorem) — idempotency
- **Cross-Track:** TRACK_A_ARTIFACT_DETERMINISM_SPEC (Venture artifacts) — determinism requirement
- **Governance:** CLAUDE.md (Determinism-First section)

## References

- **CIV-0001:** Core Simulation Loop spec
- **CIV-0104:** Minimal Constraint Set Theorem
- **TRACK_A:** Artifact Determinism Spec
- **Rust RNG:** https://docs.rs/rand/latest/rand/rngs/struct.ChaCha20Rng.html
- **Ordered-Float:** https://docs.rs/ordered-float/latest/ordered_float/

---

**Decision Delta:**
- All simulation runs must be replay-deterministic
- Mandatory replay tests (CI gate)
- ChaCha20Rng, BTreeMap, no floating-point in simulation logic
- Determinism validator tool in `policy/` crate

**Non-Determinism Policy:** ZERO TOLERANCE. Any non-determinism is treated as a critical bug.

**Review Date:** 2026-05-21 (after P0 determinism tests pass)


---
