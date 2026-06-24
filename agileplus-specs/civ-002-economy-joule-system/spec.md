---
spec_id: civ-002
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-05-29
---

# Specification: Economy and Joule System

**Slug**: civ-002-economy-joule-system | **Epic**: E2 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

The Joule energy unit is the universal economic numeraire of Civis. Without a correct production, allocation, market, and taxation layer, no civilization simulation can be stable. The economy system must be deterministic (no float in state paths), conserving (total Joules invariant), and fully auditable via event log.

## Target Users

- Economy crate developers implementing `civ-economy`
- Policy designers using scenario YAML to tweak tax rates and production multipliers
- Research agents running economy conservation property tests

## Functional Requirements

- [ ] **FR-ECON-001**: Buildings produce goods each tick from production rate + available inputs; production halts on missing inputs; production events emitted
- [ ] **FR-ECON-002**: Joule energy conservation — total Joules invariant across ticks; property test: sum invariant; fatal error on violation with ledger diff
- [ ] **FR-ECON-003**: Market clearing each tick via price discovery; deterministic clearing algorithm; uncleared orders expire after configurable TTL
- [ ] **FR-ECON-004**: Configurable taxation system; tax revenue credited to institution treasury same tick; legitimacy decrements as function of effective tax rate
- [ ] **FR-ECON-005**: Allocation algorithm distributes goods to consumers by priority (subsistence before luxury); unmet needs increment deprivation; O(n log n) completion
- [ ] **FR-METRICS-001**: `Metrics` struct with `waste_joules`, `surplus_joules`, `tyranny_index`, `legitimacy_index` fields; all f64 derivable from budget + consumption
- [ ] **FR-METRICS-002**: `compute(energy_budget_joules, consumption_joules) -> Metrics` — constant time, no I/O, no allocation; P99 < 1 µs
- [ ] **FR-METRICS-003**: Tyranny/legitimacy computed in fixed-point `Fixed` (i64 × 10^6) for replay; float variants for research export only; agreement within 6 decimal places

## Non-Functional Requirements

- Crate: `civ-economy` (path `crates/economy/` — actual: `crates/civ-economy/`)
- No float arithmetic in state-mutating paths; fixed-point `Fixed` type enforced by type system
- Property tests use `proptest` — conservation law holds every tick
- Deterministic `MarketState::step` runs identically with same inputs

## Constraints and Dependencies

- Depends on: FR-CORE-003 (deterministic transition phase) for state mutation constraints
- Depends on: FR-CORE-001 (tick loop) for `phase_economy` integration
- Building production rates configurable per building type in scenario YAML
- Joule allocation per actor coupled to `effective_consumption` from policy phase

## Acceptance Criteria

- [ ] `cargo test -p civ-economy` passes all ledger + market + proptest invariants
- [ ] Property test: sum of all Joules is invariant across 1,000 consecutive ticks
- [ ] Market clears deterministically given identical order books
- [ ] `compute(1000.0, 500.0)` returns `waste_joules=50.0`, `surplus_joules=500.0`
- [ ] Fixed-point and float metrics agree to within 6 decimal places
- [ ] Allocation completes in O(n log n); deprivation counters increment correctly on unmet need
- [ ] Tax revenue credited to treasury in same tick as collection

## Implementation Notes

- `EconomyState`, `drain_energy_budget`, `step`, `MarketState::step` in `civ-economy/lib.rs` and `market.rs`
- `phase_economy` in `engine.rs` syncs `WorldState::energy_budget_joules` ↔ `EconomyState`
- `civ-economy` spec reference: `docs/specs/CIV-0100-economy-v1.md`, `CIV-0107-joule-economy-system-v1.md`
- Full `JouleAllocator` / actor splits not yet implemented

## Status

| Story | Status |
|-------|--------|
| E2.1 Production system | Partial (`EconomyState` + `step`; no event emission) |
| E2.2 Inventory management | Partial |
| E2.3 Market clearing | Partial (`MarketState::step` deterministic; single-good only) |
| E2.4 Joule accounting | Partial (`drain_energy_budget` exists; conservation proptest planned) |
| E2.5 Allocation algorithm | Planned (full `JouleAllocator` actor splits not started) |
| E2.6 Taxation system | Planned |
| E2.7 Budget system | Planned |
| E2.8 Legitimacy model | Planned |
| E2.9 Property testing | Partial (proptest in `market.rs`; conservation law not yet asserted) |
| E2.10 Stress testing | Planned |
