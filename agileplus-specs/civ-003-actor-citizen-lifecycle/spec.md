---
spec_id: civ-003
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Actor and Citizen Lifecycle

**Slug**: civ-003-actor-citizen-lifecycle | **Epic**: E2 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Citizens (actors) must have a full lifecycle — birth, employment, aging, retirement, death — so that population dynamics, social satisfaction, and rebellion risk can emerge from simulation state rather than be scripted. Social ideology and institutional membership must be modeled so that policy feedback loops (legitimacy → satisfaction → unrest → policy change) close correctly.

## Target Users

- Actors crate developers implementing `civ-agents` and `civ-social`
- Policy designers tuning ideology drift parameters
- Research agents studying emergent social dynamics

## Functional Requirements

- [ ] **FR-CIV-ACTOR-001**: Citizen lifecycle state machine — Born → Employed → Retired → Dead; each tick: age++, check employment, check mortality; serializable with stable entity ID
- [ ] **FR-CIV-ACTOR-002**: Citizen needs — food, housing, health; unmet needs increment deprivation counters; deprivation above threshold triggers satisfaction penalty
- [ ] **FR-CIV-SOCIAL-001**: Institution system — `Institution { policies, members, budget, approval_rating }`; methods: `add_member()`, `remove_member()`, `update_policy()`; policies stored as deterministic key-value map
- [ ] **FR-CIV-SOCIAL-002**: Ideology field on Citizen — `ideology: Fixed` in range [-1, +1] (libertarian to authoritarian); `ideology_shift()` driven by institution policy drift; bounded each tick

## Non-Functional Requirements

- Crates: `civ-agents` (actors), `civ-social` (institutions + ideology)
- All lifecycle transitions deterministic; no float in state paths
- Ideology stored as fixed-point type; float representation only for export
- Institution member/policy maps use `BTreeMap` for determinism

## Constraints and Dependencies

- Depends on: FR-CORE-002 (ECS entity model) for Citizen entity registration
- Depends on: FR-ECON-005 (allocation algorithm) for deprivation counter coupling
- Depends on: FR-CIV-SOCIAL-001 (institutions) for employment assignment
- Citizen health influenced by climate events (FR-CIV-CLIMATE-001)

## Acceptance Criteria

- [ ] Citizen lifecycle test: Born → Employed → Retired → Dead transitions deterministically over N ticks
- [ ] Deprivation counters increment when food/housing/health needs unmet
- [ ] `Institution::add_member()` / `remove_member()` maintain correct member counts
- [ ] `ideology_shift()` keeps ideology in [-1, +1] for any policy drift input
- [ ] All state survives serialization round-trip with identical values
- [ ] `cargo test -p civ-agents` and `cargo test -p civ-social` pass

## Implementation Notes

- Citizen ECS currently in `civ-engine`; planned extraction to `civ-agents` crate
- `civ-laws` RON stubs present for policy storage; institution system not yet split
- Spec reference: `docs/specs/CIV-0103-institutions-timeseries-citizen-lifecycle-v1.md`

## Status

| Story | Status |
|-------|--------|
| P2.1 Citizen lifecycle harness | Planned |
| P2.2 Citizen lifecycle implementation | Planned |
| P2.3 Institution system harness | Planned |
| P2.4 Institution implementation | Planned |
| P2.5 Ideology harness | Planned |
| P2.6 Ideology implementation | Planned |
