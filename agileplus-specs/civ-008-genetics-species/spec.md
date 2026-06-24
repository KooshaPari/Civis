---
spec_id: civ-008
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Genetics and Species Diversity

**Slug**: civ-008-genetics-species | **Epic**: E2 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Multiple species with distinct biological traits (strength, intelligence, longevity, disease resistance) create emergent diversity in labor markets, military effectiveness, and cultural identity. Genetic inheritance allows trait propagation across generations, and selective pressure (disease, famine, war) causes population-level trait drift over time. This grounds demographic simulation in meaningful biological differentiation.

## Target Users

- Actors crate developers adding species/genetics to `civ-agents`
- Simulation researchers studying demographic evolution
- Game designers balancing species trait parameters

## Functional Requirements

- [ ] **FR-CIV-BIO-001**: Species type enum `(Human, Dwarf, Elf, Goblin, ...)` configurable via scenario YAML; each species has `BaseTraits { strength, intelligence, longevity, disease_resistance }` as `Fixed` values in `[0, 2]`
- [ ] **FR-CIV-BIO-002**: Genetic trait inheritance — on Citizen birth, traits sampled from parental distribution with configurable mutation rate; mutation uses `ChaCha20Rng` draw (logged); traits bounded to `[0, 2]`
- [ ] **FR-CIV-BIO-003**: Species traits influence simulation mechanics — `strength` multiplies military unit effectiveness; `intelligence` multiplies research speed; `longevity` extends retirement age threshold; `disease_resistance` reduces health decrement from plague events

## Non-Functional Requirements

- Species traits stored as `Fixed` type; no float in trait computations
- Genetic inheritance RNG draws logged to event stream for replay
- Species registry in `BTreeMap<SpeciesId, SpeciesSpec>` for determinism
- Trait influence applies in relevant tick phases (combat, research, health)

## Constraints and Dependencies

- Depends on: FR-CIV-ACTOR-001 (citizen lifecycle) for birth event trigger
- Depends on: FR-CORE-004 (stochastic event phase) for mutation RNG
- Depends on: FR-CIV-CLIMATE-002 (disasters) for disease_resistance coupling
- Depends on: FR-CIV-WAR-001 (military units) for strength coupling
- Target release v2 — not required for MVP or v1

## Acceptance Criteria

- [ ] Species traits loadable from scenario YAML without code change
- [ ] Newborn citizen traits derived from parental distribution within mutation tolerance
- [ ] Genetic mutation RNG draws logged and reproducible from seed
- [ ] `strength` trait modifies military unit output proportionally
- [ ] `disease_resistance` reduces health decrement during plague events

## Status

| Story | Status |
|-------|--------|
| Species type enum and YAML schema | Planned |
| Genetic trait inheritance + mutation | Planned |
| Trait → simulation coupling | Planned |
