# SW-011: Aerial Combat Complete

**Status**: Proposed
**AgilePlus WP State**: planned
**Sequence**: 11
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**AgilePlus Feature Slug**: epic-027-full-conversion
**Sprint**: 4 — Mechanics
**Story Points**: 13
**Priority**: P2
**File Scope**:
  - `src/Runtime/Aviation/AerialMovementSystem.cs`
  - `src/Runtime/Aviation/AerialTargetingSystem.cs`
  - `src/SDK/Models/UnitDefinition.cs`
  - `packs/warfare-starwars/units/`
  - `packs/warfare-modern/units/`
  - `src/Tests/AerialCombatTests.cs`
**Depends On**: [SW-009]
**Requirements**: EPIC-027-FR-018, EPIC-027-NFR-005, EPIC-027-NFR-006, EPIC-027-NFR-013

---

## User Story

As a **mod player**, I want aerial units (gunships, fighters, bombers, transports) in both
mods to be fully functional — buildable, pathfinding, attacking both air and ground targets —
so that air power is a real tactical dimension, not a cosmetic placeholder.

## Background

Aerial units were partially designed in M4/M5 but the ECS integration was incomplete: units
could be declared in YAML and spawned, but pathfinding over ground tiles and targeting logic
was unreliable. The main gaps:
1. Air-unit movement component not correctly overriding terrain costs for flight.
2. Air-to-ground and air-to-air targeting range checks not implemented.
3. Both mods lack at least one dedicated air-defense counter unit.

## Acceptance Criteria

### Scenario 1 — Aerial units build and spawn

**Given** `warfare-starwars` is active and a landing pad / airfield structure is built,
**When** the player builds "LAAT/i Gunship" (Republic) or "Vulture Droid" (CIS),
**Then** the unit spawns at the production structure and is selectable.

### Scenario 2 — Aerial units pathfind without terrain obstruction

**Given** an aerial unit is spawned,
**When** the player orders it to any tile (ground, water, mountain),
**Then** the unit moves to that tile without being blocked by non-air-traversable terrain.

### Scenario 3 — Air-to-ground attack

**Given** an aerial unit is within attack range of a ground enemy,
**When** the attack cooldown fires,
**Then** the aerial unit attacks the ground target and deals damage per the YAML stat block.

### Scenario 4 — Air-to-air attack

**Given** two aerial units from opposing factions are within range,
**When** combat resolves,
**Then** they deal damage to each other using air-combat stat modifiers (if defined).

### Scenario 5 — Air defense unit counters aerial units

**Given** an anti-air unit (e.g. "ARC-170 patrol" or "Hailfire Droid") is within range
of an enemy aerial unit,
**When** the enemy aerial unit enters range,
**Then** the anti-air unit attacks the aerial target (not a ground target) preferentially.

### Scenario 6 — Both mods have ≥ 2 aerial + 1 anti-air unit

**Given** either mod is active,
**When** the player opens the build panel,
**Then** at least 2 aerial unit types and 1 anti-air unit type are available.

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | `AerialUnitArchetype` sets the ECS movement component to ignore ground terrain costs. |
| F-02 | New `target_priority` YAML key: `air`, `ground`, `all` — used by combat targeting. |
| F-03 | Anti-air units declare `target_priority: air` and get a range multiplier against aerial targets. |
| F-04 | Min 2 aerial + 1 anti-air unit per mod in YAML. |
| F-05 | Aerial units use themed projectile if SW-009 is complete; default projectile otherwise. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | Aerial pathfinding must not break ground unit pathfinding (regression test). |
| N-02 | No changes to DINO's ECS Fight system group internals; use stat modifiers only. |
| N-03 | `dotnet test` includes ≥ 5 tests for AerialUnitArchetype + targeting logic. |

## Engine Quirks / Dependencies

- Air-unit ECS components: confirm the DINO `movement` or `pathfinding` component key via
  `dinoforge component-map` before implementing terrain-ignore logic.
- DINO system group ordering: Fight → ResourceDelivery. Air targeting must hook `Fight`.
- All entity queries must use `EntityQueryOptions.IncludePrefab`.
- SW-009 (projectile support) is a soft dependency.
- SW-010 (naval) can land concurrently in Sprint 4; no ordering dependency.

## Definition of Done

- [ ] ≥ 2 aerial + 1 anti-air unit per mod in YAML and validated.
- [ ] Aerial units spawn, move over all terrain, attack ground and air targets (screenshot proof).
- [ ] Anti-air unit preferentially targets aerial units (log confirmation + screenshot).
- [ ] Aerial pathfinding regression test: ground units unaffected.
- [ ] External judge receipt: in-game screenshot showing aerial unit mid-flight.
- [ ] `dotnet test` green with ≥ 5 new aerial combat tests.

## Evidence Requirements

| Requirement ID | Evidence Type | Artifact Path Pattern | Transition Gate |
|----------------|---------------|-----------------------|-----------------|
| EPIC-027-FR-018 | ManualAttestation | `docs/proof/judge-receipts/SW-011-aerial-combat.md` (aerial unit mid-flight; air+ground targets engaged) | Implementing → Validated |
| EPIC-027-NFR-005 | CiOutput | CI build log (netstandard2.0 TFM check) | Implementing → Validated |
| EPIC-027-NFR-006 | ManualAttestation | Plugin loads under BepInEx 5.4.x; aerial units functional (log confirmation) | Implementing → Validated |
| EPIC-027-NFR-013 | CiOutput | `LogOutput.log` grep: no TypeLoadException after clean launch | Implementing → Validated |
| SW-011 | TestResult | `docs/test-results/SW-011/AerialCombatTests.xml` (≥5 tests incl. ground-pathfinding regression N-01) | Implementing → Validated |
| SW-011 | ManualAttestation | Anti-air unit preferentially targets aerial units (log + screenshot confirmation) | Implementing → Validated |
| SW-011 | ReviewApproval | PR URL (auto-detected from WorkPackage.pr_url) | Validated → Shipped |
| SW-011 | CiOutput | GitHub Actions run URL (dotnet test green) | Implementing → Validated |

## Related

- `src/Domains/Warfare/` (archetype pattern)
- SW-009 (projectile support — soft dependency)
- SW-010 (naval — concurrent Sprint 4)
