---
spec_id: civ-005
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Climate, Disasters, and Seasons

**Slug**: civ-005-climate-disasters-seasons | **Epic**: E2 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Seasons, weather events, and disasters drive emergent food scarcity, disease, migration, and conflict in the simulation. The climate system must be deterministic (season calendar is fixed per seed), and stochastic events (droughts, floods, plagues) must be reproducible via the event log. Without climate coupling, the production chain has no environmental pressure and the simulation lacks the depth needed for meaningful policy research.

## Target Users

- Climate crate developers (`civ-climate`, `civ-planet`)
- Scenario designers tuning climate parameters in YAML
- Research agents studying climate-policy interaction

## Functional Requirements

- [ ] **FR-CIV-CLIMATE-001**: Season calendar — each tick maps to a season (Spring, Summer, Autumn, Winter) based on configurable ticks-per-year; season modifies tile fertility multiplier and Citizen health baseline
- [ ] **FR-CIV-CLIMATE-002**: Stochastic disaster events (drought, flood, plague, volcanic eruption) generated in stochastic phase using `ChaCha20Rng`; each event specifies affected region, severity, duration in ticks; logged to event stream
- [ ] **FR-CIV-CLIMATE-003**: Disaster effects are deterministic given event parameters — fertility drops to `severity × base`, Citizen health decrements by `severity × health_rate` per affected tick; production buildings in affected region output halved

## Non-Functional Requirements

- Crate: `civ-planet` (orbital climate) + `civ-climate` (stochastic events)
- Season calendar computed once per run from seed + ticks-per-year config; no runtime randomness
- All disaster effect computations in fixed-point arithmetic
- Climate events emitted to event log for replay verification

## Constraints and Dependencies

- Depends on: FR-CORE-004 (stochastic event phase) for disaster event generation
- Depends on: FR-CIV-BUILD-002 (production chain) for disaster-induced output halving
- Depends on: FR-CIV-ACTOR-001 (citizen lifecycle) for health decrement coupling
- CO₂ model (CIV-0102) deferred to v2; season/disaster system is v1 scope

## Acceptance Criteria

- [ ] Season calendar produces correct season for any tick given ticks-per-year config
- [ ] Disaster events replay identically from same seed
- [ ] Tile fertility multiplier reflects current season and active disaster
- [ ] Building output in disaster-affected region is halved during disaster duration
- [ ] Citizen health decrements correctly for each affected tick

## Status

| Story | Status |
|-------|--------|
| `civ-planet` orbital climate (season calendar) | Partial (orbital climate present; season enum not finalized) |
| Stochastic disaster event generation | Planned |
| Disaster-to-production coupling | Planned |
| Disaster-to-citizen-health coupling | Planned |
| CO₂ model (CIV-0102) | Planned (v2 scope) |
