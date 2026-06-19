---
spec_id: civ-006
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Deep Combat System

**Slug**: civ-006-deep-combat | **Epic**: E4 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Warfare is a primary driver of resource redistribution, territorial change, and population displacement. The combat system must be deterministic (same battle parameters always produce same casualties), fatigue-based (sustained campaigns degrade unit effectiveness), and fully logged (every combat action auditable for research and replay). The system backs the Diplomatic FSM `ActiveConflict` state from CIV-0105.

## Target Users

- War/diplomacy crate developers implementing `civ-policy` combat subsystem
- Research agents studying war-economy interaction
- Game designers balancing unit types and fatigue parameters

## Functional Requirements

- [ ] **FR-CIV-WAR-001**: Military unit entity `MilitaryUnit { unit_type, strength, fatigue, morale, position }` — unit types: Infantry, Cavalry, Artillery, Naval; all fields fixed-point
- [ ] **FR-CIV-WAR-002**: Combat resolution — deterministic given state: `resolve_combat(attacker, defender, terrain) -> CombatResult`; fatigue penalty applies after each engagement; `CombatResult` logged to event stream
- [ ] **FR-CIV-WAR-003**: Casualty handling — dead units removed from ECS; surviving unit morale decrements by `casualty_ratio × morale_rate`; territory control transfers on army destruction
- [ ] **FR-CIV-WAR-004**: Battle replay — replaying a `.civreplay` from any battle segment produces identical casualties and morale changes; CI property test required

## Non-Functional Requirements

- Crate: `civ-policy` (combat subsystem within policy/diplomacy crate)
- No float in combat resolution; `Fixed` type throughout
- All actor-pair evaluation sorted by stable `ActorId` for determinism
- Combat operates in Phase 3 (Deterministic Transition) of tick cycle

## Constraints and Dependencies

- Depends on: FR-CORE-003 (deterministic transition phase) for combat resolution
- Depends on: FR-CIV-DIPLO-001 (diplomatic FSM) for `ActiveConflict` state gating
- Depends on: FR-CIV-ACTOR-001 (citizen lifecycle) for civilian casualty calculation
- Terrain modifier sourced from `civ-spatial` tile attributes

## Acceptance Criteria

- [ ] `resolve_combat(attacker, defender, terrain)` returns identical `CombatResult` for identical inputs
- [ ] Unit fatigue increments after each engagement; fatigue at max reduces strength by 50%
- [ ] Dead units removed from ECS within same tick as resolution
- [ ] Territory control transfer logged to event stream
- [ ] Battle replay CI test: replay identical battle segment → identical casualties

## Status

| Story | Status |
|-------|--------|
| E4.1 Military units | Planned |
| E4.2 Combat resolution | Planned |
| E4.3 Casualty handling | Planned |
| E4.4 Alliance system | Planned (civ-007 scope) |
| E4.5 War declaration | Planned (civ-007 scope) |
| E4.6 Occupied territories | Planned |
| E4.7 Truce mechanics | Planned (civ-007 scope) |
| E4.8 Battle replay test | Planned |
