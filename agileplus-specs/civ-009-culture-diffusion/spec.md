---
spec_id: civ-009
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Culture Diffusion and Ideology Spread

**Slug**: civ-009-culture-diffusion | **Epic**: E2 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Culture diffusion models how ideas, practices, and ideological alignment propagate between citizens, regions, and nations via trade, migration, occupation, and media. Without culture diffusion, the simulation has no mechanism for voluntary political change (only coercive), meaning rebellions are undermodeled and diplomacy cannot correctly account for ideological affinity. Cultural resistance to foreign influence also drives the Authoritarian Backfire theorem from CIV-0104.

## Target Users

- Social crate developers implementing culture in `civ-social`
- Research agents studying ideology spread and resistance dynamics
- Policy designers tuning culture diffusion rates and resistance thresholds

## Functional Requirements

- [ ] **FR-CIV-CULT-001**: Culture entity `Culture { id, name, ideology_centroid: Fixed, spread_rate: Fixed, resistance: Fixed }` — each nation has a dominant culture; citizens have `culture_affinity: Fixed` to each known culture
- [ ] **FR-CIV-CULT-002**: Diffusion mechanics — each tick, culture spread from high-affinity to low-affinity citizens via adjacency (trade network, shared border, occupation); spread proportional to `spread_rate × contact_intensity`; resistance reduces net spread
- [ ] **FR-CIV-CULT-003**: Ideology convergence — when a citizen's `culture_affinity` to a foreign culture exceeds `CONVERGENCE_THRESHOLD`, their `ideology` drifts toward that culture's `ideology_centroid`; this feeds back into the institution legitimacy model

## Non-Functional Requirements

- Culture spread computed in deterministic transition phase; no float
- Contact intensity derived from `civ-spatial` adjacency graph
- Diffusion computations use `Fixed` arithmetic; bounded each tick
- Culture events (significant affinity shifts) logged to event stream

## Constraints and Dependencies

- Depends on: FR-CIV-SOCIAL-002 (ideology model) for ideology drift coupling
- Depends on: FR-CIV-DIPLO-001 (diplomatic FSM) for contact intensity (trade/occupation)
- Depends on: `civ-spatial` adjacency graph for border contact
- Target release v2

## Acceptance Criteria

- [ ] Culture spread: high-affinity citizen's `culture_affinity` grows toward `spread_rate × contact_intensity` per tick
- [ ] Resistance reduces net spread proportionally
- [ ] When `culture_affinity > CONVERGENCE_THRESHOLD`, ideology drifts toward foreign centroid
- [ ] Ideology drift fed back into legitimacy model decrements
- [ ] All culture spread events logged and reproducible from replay

## Status

| Story | Status |
|-------|--------|
| Culture entity and schema | Planned |
| Diffusion mechanics | Planned |
| Ideology convergence coupling | Planned |
