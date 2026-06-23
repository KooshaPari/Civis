# Test-First Engineering Guide for CivLab

**Purpose:** Establish and enforce test-first development (TDD) for all CivLab implementation
**Scope:** All crates (engine, economy, actors, social, policy, metrics, server)
**Mandate:** Every functional requirement (FR) gets a failing test BEFORE implementation
**Coverage Target:** 100% for engine + economy crates, 80%+ for others

---

## Core TDD Cycle

For every feature requirement (FR):

```
1. Write Failing Test
   └─> File: crates/{crate}/tests/fr_{id}.rs
   └─> Test must FAIL (code doesn't exist yet)
   └─> Commit: 'test({crate}): {FR-ID} failing test'

2. Implement Feature
   └─> File: crates/{crate}/src/{module}.rs
   └─> Implement until test passes
   └─> Commit: 'feat({crate}): {FR-ID} {description}'

3. Refactor (Optional)
   └─> Improve code quality, reduce duplication
   └─> All tests still pass
   └─> Commit: 'refactor({crate}): {FR-ID} {improvement}'
```

**Each FR = 1 failing test → 1 implementation → optionally 1 refactor**

---

## Test File Naming & Organization

### Directory Structure

```
crates/{crate}/
  src/
    lib.rs          # Module exports
    module.rs       # Implementation
  tests/
    fr_{id}.rs      # Test file for FR-ID
    common/mod.rs   # Shared test fixtures (if needed)
```

### Naming Convention

| Test Type | File | Example |
|-----------|------|---------|
| Single FR test | `tests/fr_{fr_id}.rs` | `tests/fr_core_tick_loop.rs` |
| Multi-requirement test | `tests/fr_{feature}.rs` | `tests/fr_market_operations.rs` |
| Property test | `tests/fr_{feature}_properties.rs` | `tests/fr_economy_properties.rs` |
| Integration test | `tests/integration_{crates}.rs` | `tests/integration_engine_economy.rs` |
| Determinism/replay | `tests/fr_{feature}_replay.rs` | `tests/fr_determinism_replay.rs` |

### Module-Level Tests

For small, unit-level tests, keep them in the same file as the implementation:

```rust
// crates/engine/src/simulation.rs

pub fn tick_turn_counter(turn: &mut u64) -> Result<()> {
    *turn += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_increments() {
        let mut turn = 0;
        tick_turn_counter(&mut turn).unwrap();
        assert_eq!(turn, 1);
    }
}
```

### Integration Tests (Preferred for FR tests)

Keep tests in `tests/` directory to test module boundaries:

```rust
// crates/engine/tests/fr_core_tick_loop.rs

#[test]
fn test_simulation_tick_increments_turn() {
    let mut sim = Simulation::new();
    let initial_turn = sim.current_turn();
    sim.tick().expect("tick should succeed");
    assert_eq!(sim.current_turn(), initial_turn + 1);
}
```

---

## Test Types Required

Every FR must have AT LEAST ONE test from the following categories. Most FRs need multiple:

### 1. Unit Tests (Pure Functions)

Test a single function with simple inputs/outputs.

**When to use:** Pure logic, no state mutation, no external dependencies

**Example:**

```rust
// crates/economy/tests/fr_econ_market.rs

#[test]
fn test_market_price_increase() {
    let mut market = Market::new();
    market.record_transaction(GoodID::Grain, 100.0); // high supply
    market.record_transaction(GoodID::Grain, 10.0);  // low demand

    let old_price = market.get_price(GoodID::Grain);
    market.update_prices();
    let new_price = market.get_price(GoodID::Grain);

    assert!(new_price < old_price); // low demand → price down
}
```

### 2. Integration Tests (Crate Boundary)

Test interaction between modules within a crate or across crate boundaries.

**When to use:** Module interactions, public API, setup/teardown

**Example:**

```rust
// crates/engine/tests/integration_engine_economy.rs

#[test]
fn test_simulation_integrates_economy() {
    let mut sim = Simulation::new();

    // Set up economy state
    sim.economy.add_goods(vec![Grain, Wood]);

    // Tick should call economy::tick()
    sim.tick().expect("tick succeeds");

    // Verify economy was ticked
    assert_eq!(sim.turn(), 1);
}
```

### 3. Scenario Tests (Full Sim Run)

Load a scenario YAML, run N ticks, assert metric snapshots.

**When to use:** Full system behavior, emergent properties, reproducibility

**Example:**

```rust
// crates/engine/tests/fr_scenario_loader.rs

#[test]
fn test_scenario_runs_100_ticks() {
    let scenario = Scenario::load_yaml("docs/scenarios/test_basic.yaml")
        .expect("scenario loads");
    let mut sim = scenario.into_simulation();

    for _ in 0..100 {
        sim.tick().expect("tick succeeds");
    }

    let snapshot = sim.snapshot();
    assert!(snapshot.population > 0); // population survived
    assert_eq!(snapshot.tick, 100);
}
```

### 4. Replay Tests (Determinism)

Record simulation state, replay with same seed, verify byte-for-byte equality.

**When to use:** Determinism verification, replay validation, save/restore

**Example:**

```rust
// crates/engine/tests/fr_determinism_replay.rs

#[test]
fn test_deterministic_replay_100_ticks() {
    let seed = 42u64;

    // First run
    let mut sim1 = Simulation::with_seed(seed);
    let states1 = record_states(&mut sim1, 100);

    // Second run (same seed)
    let mut sim2 = Simulation::with_seed(seed);
    let states2 = record_states(&mut sim2, 100);

    // Verify state equality
    assert_eq!(states1, states2, "replays must be deterministic");
}

fn record_states(sim: &mut Simulation, ticks: u64) -> Vec<SimulationState> {
    let mut states = Vec::new();
    for _ in 0..ticks {
        states.push(sim.state().clone());
        sim.tick().expect("tick");
    }
    states
}
```

### 5. Property-Based Tests (Invariants)

Use `proptest` to verify invariants hold across random input ranges.

**When to use:** Algorithms, resource allocation, conservation laws

**Example:**

```rust
// crates/economy/tests/fr_econ_properties.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_joule_allocation_never_exceeds_budget(
        available_joules in 1000u64..1_000_000u64,
        actor_count in 1..100usize,
    ) {
        let mut allocator = JouleAllocator::new();
        let allocation = allocator.allocate(available_joules, actor_count);

        let total: u64 = allocation.iter().sum();
        prop_assert!(total <= available_joules,
            "allocation {} exceeds budget {}", total, available_joules);
    }

    #[test]
    fn prop_market_prices_stay_bounded(
        transactions in prop::collection::vec((any::<u32>(), any::<f32>()), 1..1000),
    ) {
        let mut market = Market::new();
        for (good_id, qty) in transactions {
            market.record_transaction(GoodID(good_id as u32), qty.abs());
        }
        market.update_prices();

        for (_, price) in market.prices() {
            prop_assert!(price >= 0.0 && price <= 1000.0,
                "price {} out of bounds", price);
        }
    }
}
```

### 6. Snapshot/Regression Tests (State Capture)

Capture full system state, commit as golden file, verify future runs match.

**When to use:** Complex emergent behavior, historical validation

**Example:**

```rust
// crates/engine/tests/fr_state_snapshot.rs

#[test]
fn test_state_matches_golden_snapshot() {
    let mut sim = Scenario::load_yaml("docs/scenarios/test_small.yaml")
        .unwrap()
        .into_simulation();

    for _ in 0..50 {
        sim.tick().unwrap();
    }

    let snapshot = sim.snapshot();
    insta::assert_debug_snapshot!(snapshot);
}
```

(Uses `insta` crate for golden file management)

---

## Cargo Test Organization

### Run All Tests

```bash
cargo test --all
```

### Run Tests by Crate

```bash
cargo test --package civ-engine
cargo test --package civ-economy
cargo test --package civ-actors
```

### Run Tests by Category

```bash
# Unit tests only (in-module)
cargo test --lib

# Integration tests only
cargo test --test '*'

# Single test file
cargo test --test fr_core_tick_loop

# Single test function
cargo test --test fr_core_tick_loop test_simulation_tick_increments_turn

# Determinism tests (single-threaded for consistency)
cargo test -- --test-threads=1
```

### Run With Logging

```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Test Coverage

```bash
# Using tarpaulin
cargo tarpaulin --out Html --exclude-files tests/ --timeout 300

# Using llvm-cov
cargo llvm-cov --html
```

---

## Writing Your First Failing Test

Step-by-step example for `FR-CIV-ECON-001-MARKET`:

### Step 1: Create Test File

```bash
touch crates/economy/tests/fr_econ_market.rs
```

### Step 2: Write Test That MUST FAIL

```rust
// crates/economy/tests/fr_econ_market.rs

use civ_economy::market::{Market, GoodID, Price};

#[test]
fn test_market_tracks_grain_price() {
    let mut market = Market::new();

    // Record transactions
    market.record_transaction(GoodID::Grain, 100.0); // supply
    market.record_transaction(GoodID::Grain, 10.0);  // demand

    // Update prices based on supply/demand
    market.update_prices();

    // Verify price decreased (low demand)
    let price = market.get_price(GoodID::Grain);
    assert!(price < 50.0, "grain price should decrease with low demand");
}
```

### Step 3: Verify Test Fails

```bash
cd crates/economy
cargo test test_market_tracks_grain_price

# Output should be:
# error[E0433]: cannot find `Market` in this scope
#    --> tests/fr_econ_market.rs:3:29
```

### Step 4: Commit Failing Test

```bash
git add tests/fr_econ_market.rs
git commit -m "test(economy): FR-CIV-ECON-001 failing test"
```

### Step 5: Implement Until Test Passes

```rust
// crates/economy/src/market.rs

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoodID {
    Grain,
    Wood,
    // ...
}

pub type Price = f32;

pub struct Market {
    prices: HashMap<GoodID, Price>,
    supply: HashMap<GoodID, f32>,
    demand: HashMap<GoodID, f32>,
}

impl Market {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
            supply: HashMap::new(),
            demand: HashMap::new(),
        }
    }

    pub fn record_transaction(&mut self, good: GoodID, quantity: f32) {
        // Simple heuristic: positive = supply, negative = demand
        if quantity > 0.0 {
            *self.supply.entry(good).or_insert(0.0) += quantity;
        } else {
            *self.demand.entry(good).or_insert(0.0) += quantity.abs();
        }
    }

    pub fn update_prices(&mut self) {
        for (good, _) in self.prices.iter_mut() {
            let supply = self.supply.get(good).copied().unwrap_or(0.0);
            let demand = self.demand.get(good).copied().unwrap_or(1.0); // avoid div by 0

            // Price inversely proportional to supply/demand ratio
            let ratio = supply / (demand + 0.1);
            let new_price = 50.0 / (1.0 + ratio);

            self.prices.insert(*good, new_price);
        }
    }

    pub fn get_price(&self, good: GoodID) -> Price {
        self.prices.get(&good).copied().unwrap_or(50.0)
    }
}
```

### Step 6: Verify Test Passes

```bash
cargo test test_market_tracks_grain_price

# Output should be:
# test test_market_tracks_grain_price ... ok
```

### Step 7: Commit Implementation

```bash
git add crates/economy/src/market.rs
git commit -m "feat(economy): FR-CIV-ECON-001 market implementation"
```

---

## Copilot L3 Agent Test-First Pattern

For delegating to L3 copilot agents, use this prompt template:

### Phase 1: Write Failing Test

```bash
copilot -p "
Implement FR-CIV-ECON-001: Market price tracking.

Step 1 (THIS TASK): Write a failing test.

Requirements:
- Create file: crates/economy/tests/fr_econ_market.rs
- Test name: test_market_tracks_grain_price
- Test must verify:
  * Market::new() creates empty market
  * market.record_transaction(GoodID::Grain, 100.0) records supply
  * market.record_transaction(GoodID::Grain, 10.0) records demand
  * market.update_prices() calculates prices
  * market.get_price(GoodID::Grain) returns price < 50.0 (low demand)
- Test MUST FAIL right now (Market doesn't exist)
- Do NOT implement Market yet
- Commit message: 'test(economy): FR-CIV-ECON-001 failing test'

Only write the test file. Do not implement the feature.
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

### Phase 2: Implement Feature

```bash
copilot -p "
Implement FR-CIV-ECON-001: Market price tracking.

Step 2 (THIS TASK): Implement the feature.

Requirements:
- File: crates/economy/src/market.rs
- Implement Market struct with:
  * new() -> Self
  * record_transaction(good: GoodID, quantity: f32)
  * update_prices()
  * get_price(good: GoodID) -> f32
- Invariant: price is always in range [0.0, 1000.0]
- Test must pass: cargo test --package civ-economy test_market_tracks_grain_price
- Must pass clippy: cargo clippy --package civ-economy -- -D warnings
- Commit message: 'feat(economy): FR-CIV-ECON-001 market'

The test already exists in crates/economy/tests/fr_econ_market.rs
Make it pass.
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

---

## Coverage Requirements & Verification

### Coverage Targets by Crate

| Crate | Target | Measurement |
|-------|--------|-------------|
| engine | 100% | Line + branch coverage |
| economy | 100% | Line + branch coverage |
| actors | 80% | Line coverage |
| social | 80% | Line coverage |
| policy | 80% | Line coverage |
| metrics | 80% | Line coverage |
| server | 70% | Line coverage (external deps harder to test) |
| io | 70% | Line coverage (I/O harder to test) |

### Measure Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin \
  --out Html \
  --output-dir coverage/ \
  --exclude-files tests/ \
  --timeout 300

# View report
open coverage/index.html
```

### Identify Coverage Gaps

```bash
# List uncovered lines
cargo tarpaulin \
  --out Stdout \
  --exclude-files tests/ \
  | grep "MISSED"
```

### Add Tests for Gaps

For each uncovered line:

```rust
// crates/engine/tests/fr_coverage_gaps.rs

#[test]
fn test_error_path_invalid_tick() {
    // Test the error case that wasn't covered
    let sim = Simulation::new();
    let result = sim.invalid_operation();
    assert!(result.is_err());
}
```

---

## CI/CD Integration

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

cargo fmt --all
cargo clippy --all -- -D warnings
cargo test --all

# Must pass traceability
task spec:validate
```

### GitHub Actions Example

```yaml
name: tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --verbose
      - run: cargo clippy --all -- -D warnings
      - run: cargo tarpaulin --out Xml --exclude-files tests/
      - uses: codecov/codecov-action@v3
```

---

## Anti-Patterns

### ❌ DO NOT

1. **Write tests after implementation**
   - Bad: Implement feature, then add tests (false confidence)
   - Good: Write failing test first, then implement

2. **Skip unit tests because you have integration tests**
   - Bad: Only scenario tests
   - Good: Unit + integration + scenario tests

3. **Ignore coverage gaps**
   - Bad: "That error path will never happen"
   - Good: Test all branches, even error paths

4. **Use `#[ignore]` for broken tests**
   - Bad: Mark test as ignored
   - Good: Fix the test or remove it

5. **Comment out assertions to "fix" failures**
   - Bad: `// assert!(condition);` // TODO: fix this
   - Good: Fix the code to make assertion pass

6. **Test implementation details instead of behavior**
   - Bad: `assert_eq!(sim.actors.len(), 10);` // checking internal vec
   - Good: `assert!(sim.snapshot().population > 0);` // checking observable behavior

7. **Write non-deterministic tests**
   - Bad: `assert!(rng.gen::\<f32\>() < 0.5);` // flaky
   - Good: Seed RNG, verify behavior deterministically

---

## Example: Complete FR Implementation with Tests

Full example from test to code:

### Test File: `crates/economy/tests/fr_econ_joule.rs`

```rust
use civ_economy::joule::JouleAllocator;

#[test]
fn test_joule_allocation_respects_budget() {
    let available = 1000.0;
    let actor_count = 10;

    let allocator = JouleAllocator::new();
    let allocation = allocator.allocate(available, actor_count);

    let total: f32 = allocation.iter().sum();
    assert!(total <= available,
        "allocation {} exceeded budget {}", total, available);
}

#[test]
fn test_joule_fair_distribution() {
    let available = 1000.0;
    let actor_count = 10;

    let allocator = JouleAllocator::new();
    let allocation = allocator.allocate(available, actor_count);

    let avg = available / actor_count as f32;
    let tolerance = avg * 0.1; // 10% variance

    for joule in allocation.iter() {
        assert!((joule - avg).abs() < tolerance,
            "joule {} not within 10% of avg {}", joule, avg);
    }
}
```

### Implementation: `crates/economy/src/joule.rs`

```rust
pub struct JouleAllocator;

impl JouleAllocator {
    pub fn new() -> Self {
        Self
    }

    pub fn allocate(&self, available: f32, actor_count: usize) -> Vec<f32> {
        if actor_count == 0 {
            return Vec::new();
        }

        let per_actor = available / actor_count as f32;
        vec![per_actor; actor_count]
    }
}
```

### Export: `crates/economy/src/lib.rs`

```rust
pub mod joule;

pub use joule::JouleAllocator;
```

### Run Tests

```bash
cargo test --package civ-economy test_joule

# test test_joule_allocation_respects_budget ... ok
# test test_joule_fair_distribution ... ok
```

---

## Summary

**The Rule:**
- Every FR gets a failing test BEFORE code
- Every test file in `crates/{crate}/tests/fr_{id}.rs`
- Multiple test types per FR (unit, integration, scenario, property-based)
- 100% coverage on engine, 80%+ everywhere else
- Commit test first, then implement

**Copilot L3 Workflow:**
1. Dispatch agent to write failing test
2. Verify test fails
3. Dispatch agent to implement
4. Verify test passes

