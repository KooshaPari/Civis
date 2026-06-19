---
spec_id: civ-001
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-05-29
---

# Specification: Core Simulation Engine

**Slug**: civ-001-core-simulation-engine | **Epic**: E1 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

The simulation requires a deterministic, fixed-timestep tick loop that advances world state reproducibly, supports multi-client command input, and enforces performance budgets. Without this foundation, no other subsystem (economy, actors, climate, diplomacy) can be correctly built or tested.

## Target Users

- Engine agent developers implementing crates under `civ-engine`
- QA agents running determinism CI gates
- Client protocol agents connecting to the tick stream

## Functional Requirements

- [ ] **FR-CORE-001**: Fixed-timestep tick loop at 100 ms/tick; 10,000 ticks < 2 min headless; jitter < 1 ms P99
- [ ] **FR-CORE-002**: ECS entity model with cells, buildings, agents, institutions; all addressable by stable entity ID; O(n) component queries
- [ ] **FR-CORE-003**: Deterministic transition phase; bit-identical output for identical inputs; no f32/f64 in state-mutating paths; CI gate blocks on failure
- [ ] **FR-CORE-004**: Stochastic event phase using ChaCha20Rng seeded per run; every RNG draw logged in event stream; property test: same seed → same sequence
- [ ] **FR-CORE-005**: Policy evaluation phase runs before production/allocation; pure function of current state; overridable via Scenario API without engine modification
- [ ] **FR-CORE-006**: Multi-client command queue; admin > player > research priority; FIFO within tier; post-cutoff commands deferred
- [ ] **FR-CORE-007**: Tick budget 14 ms P99 on 4-core/16 GB; ticks > 16 ms logged with phase breakdown; CI gate blocks on P99 regression

## Non-Functional Requirements

- Crate: `civ-engine` (path `crates/engine/`)
- All state mutations in fixed-point types enforced by type system
- Replay harness: `determinism_proptest.rs` — runs on every CI commit
- Tick phases ordered: Policy → Deterministic → Stochastic → Allocation → Economy → Replay-record

## Constraints and Dependencies

- Depends on: phenotype-voxel SVO kernel (sibling path-dep) for voxel substrate
- No std::time or system clock in deterministic phases
- `BTreeMap` (not `HashMap`) for all ordered state collections

## Acceptance Criteria

- [ ] `cargo test -p civ-engine` passes all determinism and replay tests
- [ ] Replay of any recorded run from event log produces identical state hashes at every tick
- [ ] 10,000 ticks run in < 2 minutes on commodity hardware (CI verified)
- [ ] Tick jitter < 1 ms P99 under no-client load
- [ ] Entity IDs survive serialization/deserialization round-trips unchanged
- [ ] Multi-client command queue correctly prioritizes and defers commands

## Implementation Notes

- `Simulation::tick()` lives in `crates/engine/src/engine.rs`
- `SimRng = ChaCha8Rng` — seeded from run header
- `ReplayLog` / `.civreplay` format defined in `replay.rs` and `replay_format.rs`
- Scenario loader landed early: `crates/engine/src/scenario.rs`

## Status

| Story | Status |
|-------|--------|
| E1.1 Fixed-timestep tick loop | Partial (tick loop present; jitter CI gate not yet gated) |
| E1.2 ECS entity model | Partial (citizen ECS in engine; no separate actors/social crates) |
| E1.3 Policy evaluation phase | Planned |
| E1.4 Deterministic transition | Partial (fixed-point types; no float assertion CI) |
| E1.5 Stochastic event phase | Planned |
| E1.6 State serialization | Partial |
| E1.7 Multi-client command queue | Planned |
| E1.8 .civreplay export | Partial |
| E1.9 Determinism CI gate | Partial |
| E1.10 Performance profiling | Planned |
