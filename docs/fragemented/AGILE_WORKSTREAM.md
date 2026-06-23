# CivLab Agile+ Workstream & Execution Plan

**Version:** 1.0
**Status:** ACTIVE
**Date:** 2026-02-21
**Authors:** CIV Delivery & Engineering Team

---

## Overview

This document defines the **agile+ phased execution plan** for CivLab from MVP through v1.

**Execution Model:**
- **Milestones:** M0 (core tick) → M1 (economy+Joule) → M2 (client protocol + first Bevy client) → M3 (war/diplomacy shadow) → M4 (research API + test coverage)
- **Sprints:** 2-week iterations, feature-focused with acceptance tests
- **Parallel Tracks:** Git worktree strategy for independent feature development
- **L3 Copilot Agents:** Background task dispatch for implementation work
- **Definition of Done:** Tests pass, clippy clean, FR tags, commit references story ID

---

## Execution Timeline

### Phase 0: Foundation (Weeks 1-3)

**Goal:** Establish infrastructure, CI/CD, core patterns.

| Milestone | Duration | Key Deliverables |
|-----------|----------|---|
| **M0.0: Project Setup** | 1 week | Cargo workspace, CI/CD, linters, test infrastructure |
| **M0.1: Core Tick Loop** | 2 weeks | Fixed-timestep engine, phase schedule, determinism tests |

**Definition of Ready (Before Sprint):**
- [ ] Cargo workspace structured (CIV-0001, ADR-001)
- [ ] CI pipeline configured (GitHub Actions, clippy, tests)
- [ ] Test templates created (determinism, replay, property tests)
- [ ] Development environment documented (Makefile, scripts)

**Definition of Done (Sprint Completion):**
- [ ] Tick loop passes determinism test (same seed → identical state)
- [ ] Phase schedule verified (Command → Policy → Transition → Stochastic → Metrics → Broadcast)
- [ ] Performance profiled (all phases \< 14 ms total)
- [ ] Code review passed
- [ ] Acceptance tests all green

---

### Phase 1: Economy & Joule (Weeks 4-9)

**Goal:** Implement Joule economy, production, markets, taxation.

| Milestone | Duration | Key Deliverables |
|-----------|----------|---|
| **M1.0: Basic Production** | 1.5 weeks | Buildings produce goods; inventory management |
| **M1.1: Market Clearing** | 1.5 weeks | Bid/ask matching, price discovery, allocation |
| **M1.2: Joule System** | 2 weeks | Energy accounting, conservation checks, efficiency losses |
| **M1.3: Taxation & Budget** | 1.5 weeks | Fiscal policy, spending, revenue, deficit handling |
| **M1.4: Legitimacy & Mood** | 1 week | Citizen happiness, rebellion risk, policy feedback |

**Stories per Milestone (Example: M1.0 — Basic Production)**

#### Story: E1.1 — Implement Production System

**ID:** FR-CIV-ECON-001
**Tier:** MVP
**Points:** 5
**Assignee:** (L3 Copilot Agent)

**Description:**
Buildings produce goods per tick. Production rates based on:
- Building type (farm → grain; mill → flour; smithy → iron tools)
- Labor input (citizens assigned to building)
- Resource availability (input goods must be in inventory)

**Acceptance Criteria:**
```gherkin
Given a farm building with 10 citizens and 100 grain in warehouse
When simulation runs 1 tick
Then farm produces 50 grain (5 grain per citizen per tick)
And grain is added to warehouse inventory
And production event is emitted with output amount

Given a farm with 0 citizens
When simulation runs 1 tick
Then farm produces 0 grain

Given a smithy requiring iron ore input
When smithy has iron ore in inventory
Then smithy produces 5 iron tools per tick
When smithy runs out of iron ore
Then smithy produces 0 tools (not negative inventory)
```

**Determinism Note:**
- Production rates are deterministic per tick (no RNG)
- Order of production: iterate buildings in sorted order (BTreeMap key)
- All arithmetic is fixed-point (i64 units)

**Definition of Done:**
- [ ] `crates/economy/src/production.rs` implemented
- [ ] Property test: conservation (sum of goods pre + produced = sum of goods post)
- [ ] Replay test: same state + control → identical production
- [ ] Integration test: 100-tick simulation with mixed building types
- [ ] Clippy: zero warnings
- [ ] Commit message: `feat(economy): FR-CIV-ECON-001 production system`

**Test Template:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_farm_produces_grain() {
        let mut world = create_test_world();
        let farm_id = world.add_building(BuildingType::Farm);
        world.assign_citizens(farm_id, 10);
        world.add_inventory(farm_id, Good::Grain, 100);

        let state0 = world.state();
        let state1 = production_phase(&state0, &world);

        assert_eq!(state1.inventory(farm_id, Good::Grain), 150);
        assert_events_contain(state1.events(), "production.completed");
    }

    #[test]
    fn test_production_deterministic_replay() {
        let seed = 12345u64;
        let state0 = create_test_world().state();

        let (state1, events1) = production_phase_seeded(&state0, &world, seed);
        let (state2, events2) = production_phase_seeded(&state0, &world, seed);

        assert_eq!(state1, state2);
        assert_eq!(events1, events2);
    }

    #[test]
    fn test_production_conservation_property() {
        let state0 = create_test_world_with_buildings(100).state();
        let world = state0.world();

        let pre_grain = world.total_inventory(Good::Grain);
        let state1 = production_phase(&state0, &world);
        let post_grain = world.total_inventory(Good::Grain);

        // Grain produced should equal increase in inventory
        let grain_produced: i64 = state1.events()
            .iter()
            .filter_map(|e| {
                if let Event::ProductionCompleted { good: Good::Grain, amount, .. } = e {
                    Some(amount)
                } else {
                    None
                }
            })
            .sum();

        assert_eq!(post_grain, pre_grain + grain_produced);
    }
}
```

**Copilot Dispatch Command:**
```bash
# Spawn L3 copilot agent to implement FR-CIV-ECON-001
copilot -p \
  "Implement FR-CIV-ECON-001: Basic Production System.
   1. Add production.rs to crates/economy/src/
   2. Define ProductionConfig with building type → output rates mapping.
   3. Implement production_phase(state, world) → (state', events).
   4. All output is fixed-point i64 (units, not floats).
   5. Write determinism + property tests (templates in docs/testing/).
   6. Run cargo test, clippy. Zero warnings.
   7. Commit: feat(economy): FR-CIV-ECON-001 production system
   8. Reference: CIV-0001, CIV-0107, CIV-0100." \
  --yolo \
  --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

**Dependencies:**
- [ ] Core tick loop (M0.1) complete
- [ ] World/Entity model defined

**Related Stories:**
- E1.2 (ECS entity model)
- E2.2 (Inventory management)

---

#### Story: E1.2 — ECS Entity Model

**ID:** FR-CIV-CORE-019
**Tier:** MVP
**Points:** 8

**Description:**
Define ECS-style entity model:
- Entity IDs (u64)
- Dense component arrays (Position, Inventory, Mood, Health)
- Sparse components (InstRole, MilCommand)
- Query system (iterate entities with specific components)

**Acceptance Criteria:**
```gherkin
Given a world with 1000 agents and 100 buildings
When query agents with (Position, Inventory)
Then iteration is cache-friendly (contiguous memory)
And iteration order is deterministic (sorted by entity ID)

Given agent entity with components (Position, Inventory, Mood)
When query (Position, Inventory)
Then result includes agent
When query (Position, MilCommand)
Then result excludes agent (no MilCommand)
```

**Rust Pattern:**
```rust
pub struct World {
    entity_gen: Vec<u32>,        // Generation for handle validity
    entity_alive: BitSet,        // Which entities are alive
    position: Vec<Position>,     // Dense
    inventory: Vec<Inventory>,   // Dense
    mood: Vec<Mood>,             // Dense
    inst_role: SparseSet<Role>,  // Sparse
}

pub fn iter_entities<'a>(
    world: &'a World,
    query: Query
) -> QueryIter<'a> {
    // Iterate alive entities; return only matching components
}
```

**Definition of Done:**
- [ ] `crates/engine/src/ecs.rs` implemented
- [ ] Query system passes benchmark (1M entities iterated in \< 1 ms)
- [ ] Iteration order guaranteed sorted by entity ID (BTreeMap-backed)
- [ ] Zero-copy (no allocations per query)
- [ ] Clippy: zero warnings
- [ ] Commit message: `feat(engine): FR-CIV-CORE-019 ECS entity model`

---

### Phase 2: Multi-Client Protocol (Weeks 10-15)

**Goal:** WebSocket server, JSON-RPC, binary frames, client libraries.

| Milestone | Duration | Key Deliverables |
|-----------|----------|---|
| **M2.0: WebSocket Server** | 1.5 weeks | Accept connections, upgrade RFC 6455 |
| **M2.1: JSON-RPC Dispatcher** | 1.5 weeks | Method dispatch, error handling |
| **M2.2: Client Handshake & Snapshot** | 1.5 weeks | Bootstrap data, snapshot streaming |
| **M2.3: Command Protocol** | 1.5 weeks | Command validation, priority queue, response |
| **M2.4: Bevy Client Plugin** | 2 weeks | ECS integration, entity sync, rendering |

**Stories per Milestone (Example: M2.0 — WebSocket Server)**

#### Story: E3.1 — Implement WebSocket Server

**ID:** FR-CIV-PROTO-002
**Tier:** v1
**Points:** 5

**Description:**
Implement WebSocket server (RFC 6455) on port 9876. Accept client connections, perform upgrade handshake, manage session state.

**Acceptance Criteria:**
```gherkin
Given WebSocket server running on localhost:9876
When client connects with HTTP upgrade request
Then server responds with 101 Switching Protocols
And WebSocket stream is established

Given 10 simultaneous client connections
When server runs 100 ticks
Then all clients remain connected
And no dropped frames
And server memory usage is < 100 MB
```

**Rust Libraries:**
- `tokio-tungstenite` (WebSocket server)
- `tokio` (async runtime)

**Definition of Done:**
- [ ] `crates/server/src/websocket.rs` implemented
- [ ] Integration test: connect 5 clients, exchange 10 frames, disconnect
- [ ] Load test: 50 clients, no dropped connections
- [ ] Clippy: zero warnings
- [ ] Commit message: `feat(server): FR-CIV-PROTO-002 WebSocket server`

**Copilot Dispatch:**
```bash
copilot -p \
  "Implement FR-CIV-PROTO-002: WebSocket Server (RFC 6455).
   1. Create crates/server/src/websocket.rs
   2. Use tokio-tungstenite for WebSocket handling.
   3. Listen on 127.0.0.1:9876.
   4. Accept client connections, manage ClientSession state.
   5. Handle graceful disconnect (close frame).
   6. Write integration tests: 5 clients, 10 frame exchange.
   7. Load test: 50 simultaneous clients.
   8. Clippy: zero warnings.
   9. Commit: feat(server): FR-CIV-PROTO-002 WebSocket server
   10. Reference: CIV-0200 (Multi-Client Protocol), RFC 6455." \
  --yolo --model gpt-5-mini --add-dir /path/to/civ &
```

---

### Phase 3: War & Diplomacy (Weeks 16-19)

**Goal:** Combat, alliances, basic diplomacy events. (V1 "Shadow" implementation)

| Milestone | Duration | Key Deliverables |
|-----------|----------|---|
| **M3.0: Military Units** | 1.5 weeks | Unit types, armies, generals |
| **M3.1: Combat Resolution** | 1.5 weeks | Deterministic combat, casualties, fatigue |
| **M3.2: Alliances & Diplomacy** | 1.5 weeks | Treaties, alliances, betrayal mechanics |

---

### Phase 4: Research API & Test Coverage (Weeks 20-24)

**Goal:** Scenario API, query system, comprehensive test coverage.

| Milestone | Duration | Key Deliverables |
|-----------|----------|---|
| **M4.0: Scenario YAML Format** | 1.5 weeks | Scenario definition, loading, configuration |
| **M4.1: Python Scenario API** | 2 weeks | Load scenarios, run sims, query state |
| **M4.2: Test Coverage** | 1 week | Reach 80% coverage, add missing tests |

---

## Git Worktree Strategy

### Branch Naming & Worktree Allocation

**Main branch:** `main` (stable, production-ready)
**Development branch:** `dev` (integration point for features)

**Feature branches:** Per-milestone worktree

```bash
# Create worktree for M0.1 (Core Tick Loop)
git worktree add ../civ-wt-m0-core-tick \
  -b feature/m0-core-tick dev

# Create worktree for M1.0 (Basic Production)
git worktree add ../civ-wt-m1-production \
  -b feature/m1-production dev

# Create worktree for M2.0 (WebSocket Server)
git worktree add ../civ-wt-m2-websocket \
  -b feature/m2-websocket dev
```

### Parallel Development

Each worktree is **independent** and can be developed in parallel:

```bash
# Terminal 1: Work on M0.1 (Core Tick)
cd ../civ-wt-m0-core-tick
cargo test --all
git commit -m "feat(engine): FR-CIV-CORE-001 tick loop"
git push origin feature/m0-core-tick

# Terminal 2: Work on M1.0 (Production)
cd ../civ-wt-m1-production
cargo test --all
git commit -m "feat(economy): FR-CIV-ECON-001 production system"
git push origin feature/m1-production
```

### Integration & Merge

**Before merge to `dev`:**

```bash
# Pull latest dev
git fetch origin dev:dev

# Merge dev into feature branch
git merge dev

# Run full test suite
cargo test --all --release

# Clippy
cargo clippy --all -- -D warnings

# Merge to dev (if all green)
git checkout dev
git merge --no-ff feature/m0-core-tick
git push origin dev
```

**Full CI pipeline (before merge to main):**

```bash
# In GitHub Actions
cargo test --all --release
cargo clippy --all -- -D warnings
cargo audit --deny warnings
# Property-based testing
cargo test --all deterministic_replay -- --nocapture
# Performance benchmarks
cargo bench --all
```

---

## Sprint Structure (2-Week Cycles)

### Sprint Template

**Duration:** 2 weeks (10 business days)
**Cadence:** Monday kick-off, Friday demo + retro

```
Week 1:
├─ Mon: Sprint Planning (stories groomed, assigned, COP set)
├─ Tue-Thu: Development (daily standup 10 min)
└─ Fri: Mid-sprint check-in

Week 2:
├─ Mon-Wed: Development (continue)
├─ Thu: Code review, merge to dev
└─ Fri: Sprint Demo + Retro
```

### Example: Sprint M0.1 (Weeks 1-2)

**Goal:** Core Tick Loop with determinism tests

**Stories:**
1. **FR-CIV-CORE-001** (5 pts) — Tick monotonicity
2. **FR-CIV-CORE-002** (5 pts) — Deterministic transition
3. **FR-CIV-CORE-003** (3 pts) — Seeded RNG in stochastic phase
4. **FR-CIV-CORE-013** (3 pts) — Phase schedule integrity

**Total Points:** 16 (6 points/week target)

**L3 Copilot Dispatch (Monday):**

```bash
# Dispatch story implementations as background tasks
for story in FR-CIV-CORE-001 FR-CIV-CORE-002 FR-CIV-CORE-003 FR-CIV-CORE-013; do
  copilot -p \
    "Implement story: $story
     Reference: CIV-0001 Core Simulation Loop spec
     Definition of Done:
       - Tests pass (determinism + property tests)
       - Clippy: zero warnings
       - Commit message: feat(engine): $story
       - Merge to feature branch" \
    --yolo --model gpt-5-mini --add-dir /path/to/civ &
done
```

**Daily Standup (10 min):**
- [ ] What did I complete?
- [ ] What am I working on?
- [ ] Any blockers?
- [ ] Blockers?

**Friday Demo (30 min):**
- Demonstrate working features
- Run determinism tests live
- Review code metrics (coverage, clippy)
- Accept/reject story completion

**Friday Retro (15 min):**
- What went well?
- What could improve?
- Action items for next sprint

---

## Definition of Done (Per Story)

**Before code:**
- [ ] Story is groomed (acceptance criteria clear)
- [ ] Spec reference linked (CIV-0001, etc.)
- [ ] Determinism implications identified

**During development:**
- [ ] Tests written FIRST (TDD)
- [ ] Code follows project style (clippy -D warnings)
- [ ] Fixed-point arithmetic used (no f64 in sim logic)
- [ ] BTreeMap used for ordered collections
- [ ] ChaCha20Rng used for randomness
- [ ] Determinism test included (replay verification)
- [ ] Property test included (invariant checking)

**Before commit:**
- [ ] `cargo test --all` passes (100%)
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo fmt` applied
- [ ] Code review requested (&gt; 1 approver)
- [ ] FR tag in commit message: `feat(module): FR-ID description`

**Example commit:**
```
feat(engine): FR-CIV-CORE-001 implement tick monotonicity

- Add Tick type wrapping u64
- Increment exactly once per step() call
- Write determinism test: 100 iterations, all tick[n+1] == tick[n] + 1
- Write property test: tick is monotonic across 10000 ticks
- Clippy: zero warnings
- Tests: 100% pass

Fixes #123
Refs CIV-0001 (Core Simulation Loop)
```

**After commit:**
- [ ] Code review approved
- [ ] CI pipeline green (all checks pass)
- [ ] PR merged to feature branch
- [ ] Story marked DONE in backlog

---

## L3 Copilot Agent Dispatch Pattern

### What is L3?

**L3 Agent** = autonomous LLM-driven implementation. Given clear spec + acceptance criteria, implements full feature end-to-end (write code, tests, commit).

### When to Use L3

| Scenario | Use L3 | Use Human |
|----------|--------|-----------|
| Clear spec + acceptance criteria | ✓ | |
| Needs design discussion | | ✓ |
| Novel architecture | | ✓ |
| Straightforward implementation | ✓ | |
| Needs code review (always) | ✓ | ✓ |
| Complex debugging | | ✓ |

### Dispatch Command Template

```bash
copilot -p \
  "Implement FR-{EPIC}-{NNN}: {Story Title}

   Specification:
   - Ref: {Spec File} e.g., CIV-0001, CIV-0200
   - Acceptance Criteria: {Copy from story}

   Implementation Notes:
   - Use fixed-point (i64), not floats
   - Use BTreeMap for collections
   - Use ChaCha20Rng for randomness
   - No SystemTime in simulation crate

   Testing:
   - Determinism test (same seed → identical output)
   - Property test (invariant holds across N ticks)
   - Integration test (multi-component scenario)

   Definition of Done:
   - cargo test --all → all pass
   - cargo clippy --all -- -D warnings → clean
   - commit message: feat(module): FR-{ID} {description}
   - merge to feature/{branch}

   References:
   - {Spec File}
   - {ADR if relevant}
   - {Existing similar code}" \
  --yolo \
  --model gpt-5-mini \
  --add-dir /path/to/repo &
```

### Example Dispatch (FR-CIV-CORE-001)

```bash
copilot -p \
  "Implement FR-CIV-CORE-001: Tick Monotonicity

   Specification:
   - Ref: CIV-0001 (Core Simulation Loop spec, section 'Tick Architecture')
   - Requirement: Tick increments exactly once per step. No frame skipping.
   - Ticks are u64 starting from 0.

   Acceptance Criteria:
   1. Create Tick struct or type wrapping u64
   2. Implement Engine::tick() → new Tick
   3. Test: 100 sequential step() calls produce tick[0..100] without gaps
   4. Property test: 10000 steps, verify all ticks monotonic increasing
   5. Determinism test: same input → same tick sequence (use seed)

   Implementation:
   - File: crates/engine/src/tick.rs
   - Define: pub struct Tick(u64)
   - Define: pub fn step(&mut self) -> Tick { self.tick += 1; Tick(self.tick) }
   - No floats. No SystemTime. No randomness.

   Testing:
   - Unit tests in tick.rs (determinism + property)
   - Integration test: run simulation 100 ticks, verify monotonic

   Definition of Done:
   - cargo test crate::tick → 100% pass
   - cargo clippy --all -- -D warnings → clean
   - All tests deterministic (rerun 5x, all pass)
   - Commit: feat(engine): FR-CIV-CORE-001 tick monotonicity

   References:
   - CIV-0001 (spec, Tick Architecture section)
   - ADR-003 (determinism rule)
   - https://docs.rs/rand_chacha/ (for RNG if needed, which it isn't here)" \
  --yolo \
  --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

### Monitoring L3 Agents

```bash
# Check background copilot processes
ps aux | grep copilot

# Monitor logs
tail -f ~/.local/share/copilot/logs/latest.log

# Check git status (L3 commits to feature branch)
cd ../civ-wt-m0-core-tick
git log --oneline -10

# Merge to dev when L3 is done
git checkout dev
git merge --no-ff feature/m0-core-tick
```

---

## Acceptance Testing Framework

### Test Pyramid

```
           ╱╲
          ╱  ╲         E2E Tests (Scenario: 100k ticks)
         ╱    ╲        - Full simulation run
        ╱──────╲       - Metrics bounds check
       ╱        ╲      - Determinism verification
      ╱──────────╲     ~5 tests
     ╱            ╲
    ╱──────────────╲   Integration Tests (Multi-component)
   ╱                ╲  - ECS query + Production + Markets
  ╱████████████████╲  - Snapshot serialization
 ╱                    ╱ - Determinism replay
╱████████████████████╲ ~30 tests
Unit Tests (Single function)
- Tick increment
- Fixed-point math
- RNG seeding
- Collection ordering
~200 tests
```

### Test Category: Determinism

**Every public function must have a determinism test:**

```rust
#[test]
fn test_function_deterministic_replay() {
    let input = create_test_input();
    let seed = 12345u64;

    // Run 1
    let output1 = function_under_test(&input, seed);

    // Run 2 (replay)
    let output2 = function_under_test(&input, seed);

    // Must be identical
    assert_eq!(output1, output2);
}
```

### Test Category: Property-Based

**For invariants (laws that must hold every tick):**

```rust
#[test]
fn test_energy_conservation_property() {
    // Run simulation 1000 times with random seeds
    for seed in [1, 2, 3, ..., 1000] {
        let mut state = create_test_state();
        let mut rng = ChaCha20Rng::seed_from_u64(seed);

        for _tick in 0..100 {
            let control = generate_random_control(&mut rng);
            let state_prev = state.clone();

            state = tick(&state, &control, seed);

            // Invariant: Energy is conserved
            let energy_before = state_prev.total_energy();
            let energy_after = state.total_energy();
            let energy_dissipated = state.events()
                .iter()
                .filter_map(|e| if let Event::EnergyDissipated(x) = e { Some(x) } else { None })
                .sum::<i64>();

            assert_eq!(
                energy_after,
                energy_before - energy_dissipated,
                "Energy conservation failed at seed {}, tick {}",
                seed,
                _tick
            );
        }
    }
}
```

### Test Category: Scenario

**Full-world simulation tests:**

```rust
#[test]
fn test_scenario_starting_settlement_100_ticks() {
    let scenario = load_scenario("scenarios/starting_settlement.yaml");
    let (final_state, events) = run_simulation(scenario, 100);

    // Metric bounds
    assert!(final_state.population > 10, "Population should grow");
    assert!(final_state.treasury > 0, "Should have income");

    // No hard failures
    assert!(!events.iter().any(|e| {
        if let Event::SimulationError { .. } = e { true } else { false }
    }), "No simulation errors allowed");

    // Determinism (replay)
    let (final_state2, events2) = run_simulation(scenario, 100);
    assert_eq!(final_state, final_state2);
    assert_eq!(events, events2);
}
```

---

## Metrics & Reporting

### Weekly Status Report (Every Friday)

**Template:**

```
# CivLab Weekly Status — Week {N} ({Date})

## Completed
- [x] {Story ID}: {Story Title} (X points)
- [x] {Story ID}: {Story Title} (Y points)
**Total Points:** Z

## In Progress
- [ ] {Story ID}: {Story Title} (blocked: {reason})
- [ ] {Story ID}: {Story Title} (50% complete)

## Metrics
- **Code Coverage:** XX%
- **Determinism Tests Passing:** YY/YY
- **Performance:** Avg tick = Z ms (target: 14 ms)
- **Build Health:** ✓ (all CI checks passing)

## Risks & Blockers
- {Risk description + mitigation}

## Next Week
- {Stories planned for next sprint}
```

### Monthly Burndown Chart

**Track:**
- Planned points vs. completed points
- Velocity trend (rolling 4-week average)
- Forecast to completion

### Quality Gates (Per Merge)

| Gate | Pass/Fail | Required |
|---|---|---|
| **Tests Pass** | ✓ all tests | Yes |
| **Clippy** | ✓ zero warnings | Yes |
| **Coverage** | &gt; 70% | Yes (MVP), 80% (v1) |
| **Determinism** | ✓ replay test | Yes |
| **Performance** | &lt; 16 ms/tick | Yes (v1) |
| **Code Review** | ✓ approved | Yes |

---

## Escalation & Decision Framework

### Who Decides?

| Decision Type | Owner | Input From | Process |
|---|---|---|---|
| **Story Scope** | Product Lead | Engineering | Spec review, estimation |
| **Technical Design** | Tech Lead | Engineers | ADR, spec review |
| **Risk Acceptance** | Product + Tech Lead | Team | Retro, risk log |
| **Release Readiness** | Product Lead | QA, Tech | Checklist, metrics |
| **Priority** | Product Lead | Team | Backlog retro |

### Escalation Path

1. **Blockers (within sprint):** Daily standup → Tech Lead decision (same day)
2. **Scope creep:** Sprint Planning → Product Lead gates addition
3. **Technical debt:** Retro → Allocate 20% sprint capacity
4. **Risks:** Logged in risk register; reviewed every 2 weeks

---

## Appendix A: Story Template

**Copy this for each new story:**

```markdown
## Story: {Title}

**ID:** FR-CIV-{EPIC}-{NNN}
**Tier:** MVP / v1 / v2
**Points:** {3/5/8/13}
**Assignee:** L3 Copilot Agent / {Name}

### Description
{What is being built?}

### Acceptance Criteria
{BDD scenario or checklist}

### Determinism Note (if applicable)
{RNG, collection order, floating-point considerations}

### Implementation Notes
{Rust patterns, libraries, references}

### Definition of Done
- [ ] Tests pass (100%)
- [ ] Clippy clean
- [ ] Determinism test included
- [ ] Code review approved
- [ ] Merge to feature branch
- [ ] Commit message: feat(module): FR-ID description

### Copilot Dispatch
{Command to dispatch L3 agent}

### Dependencies
- [ ] {Other stories that must complete first}

### Related Stories
- {Other stories in same epic or related}
```

---

## Appendix B: Milestone Checklist

**M0.1: Core Tick Loop**
- [ ] Tick type + increment logic
- [ ] Phase schedule (Command → Policy → Transition → Stochastic → Metrics → Broadcast)
- [ ] Policy phase interface
- [ ] Deterministic transition phase
- [ ] Stochastic event phase (seeded RNG)
- [ ] Metrics computation
- [ ] Client broadcast infrastructure
- [ ] Determinism tests (10 stories, all pass)
- [ ] Performance profiling (14 ms target)
- [ ] Code review + merge to dev

**M1.0: Basic Production**
- [ ] Building + production model
- [ ] Inventory management
- [ ] Production phase implementation
- [ ] Production event logging
- [ ] Property test: conservation law
- [ ] Replay test: determinism
- [ ] Integration test: 100-tick scenario
- [ ] Code review + merge to dev

...and so on for each milestone.

---

## Appendix C: Performance Budget

**Per Tick (100 ms = ~14 ms per phase in 100 ms simulation step, ~1.4 ms per actual tick in wall time for 10x speed):**

```
Phase Budget:
├─ Command Intake:     50 µs (max 1000 commands)
├─ Policy:           2000 µs (evaluate all policies)
├─ Deterministic:    8000 µs (production, trade, allocation)
├─ Stochastic:       3000 µs (events, RNG rolls)
├─ Metrics:          1000 µs (aggregation)
└─ Broadcast:          50 µs (frame queueing)

Total: ~14 ms (with margin for variance)
```

**Profiling command:**
```bash
cargo build --release
cargo run --release -p civ-server -- \
  --scenario scenarios/test_10k_agents.yaml \
  --profile-ticks 1000 \
  --profile-output profile.json

# Analyze
cat profile.json | jq '.phases[] | {name, min_ms, max_ms, avg_ms}'
```

---

**Document History:**
- v1.0 (2026-02-21): Initial workstream plan. Milestones M0-M4, 2-week sprints, L3 copilot dispatch pattern, DoD checklist.
