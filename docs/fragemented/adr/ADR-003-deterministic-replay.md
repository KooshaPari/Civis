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
