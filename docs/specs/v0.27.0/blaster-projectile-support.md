# SW-009: Blaster / Projectile Support

**Status**: Proposed
**AgilePlus WP State**: planned
**Sequence**: 9
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**AgilePlus Feature Slug**: epic-027-full-conversion
**Sprint**: 4 — Mechanics
**Story Points**: 8
**Priority**: P2
**File Scope**:
  - `src/SDK/Models/UnitDefinition.cs`
  - `src/SDK/Assets/ProjectileSwapRegistry.cs`
  - `src/Runtime/Bridge/ProjectileSwapSystem.cs`
  - `schemas/unit.schema.json`
  - `packs/warfare-starwars/assets/bundles/`
  - `packs/warfare-modern/assets/bundles/`
  - `src/Tests/ProjectileSwapRegistryTests.cs`
**Depends On**: [SW-003]
**Requirements**: EPIC-027-FR-016, EPIC-027-NFR-005, EPIC-027-NFR-006, EPIC-027-NFR-013, EPIC-027-NFR-014

---

## User Story

As a **mod player**, I want units in the Star Wars and Modern Warfare packs to fire themed
projectiles — blaster bolts for Republic/CIS, bullets and tracer rounds for Modern —
instead of the vanilla DINO projectile sprite, so that combat reinforces the mod's theme.

## Background

DINO's projectile system uses `ProjectileDataBase` ECS components and visual assets linked
to `visual_asset` keys in unit YAML. `AssetSwapSystem` already swaps 3D unit meshes;
projectile visuals need the same treatment.

The key gap is that no per-mod projectile assets exist yet, and there is no `projectile_visual`
override mechanism in the pack manifest.

## Acceptance Criteria

### Scenario 1 — Republic blaster bolt fires from Clone Trooper

**Given** `warfare-starwars` is active with a Clone Trooper unit engaged in combat,
**When** the unit fires,
**Then** a gold/blue blaster bolt projectile travels from the unit to the target
(not the vanilla DINO arrow/spear projectile).

### Scenario 2 — Separatist red blaster fires from B1 Droid

**Given** `warfare-starwars` is active with a B1 Battle Droid engaged in combat,
**When** the unit fires,
**Then** a red blaster bolt projectile is visible (not vanilla projectile).

### Scenario 3 — Modern Warfare tracer round fires from Infantry

**Given** `warfare-modern` is active with an infantry unit engaged in combat,
**When** the unit fires,
**Then** a tracer-round-style projectile (thin, fast, amber `#F5A623`) travels to target.

### Scenario 4 — Missile fires from anti-air / artillery unit

**Given** either mod is active with a ranged artillery or anti-air unit,
**When** the unit fires,
**Then** a missile-style projectile (cylindrical geometry, exhaust trail) is visible.

### Scenario 5 — Default projectile fallback if asset missing

**Given** a unit's `projectile_visual` references a non-existent bundle,
**When** the unit fires,
**Then** the default DINO projectile renders (no exception, no invisible projectile),
and a WARNING appears in the BepInEx log.

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | New `projectile_visual` key on unit YAML referencing a bundle-key for the projectile prefab. |
| F-02 | `ProjectileSwapRegistry` maps `unit_id → projectile_bundle_key` (mirrors `AssetSwapRegistry` pattern). |
| F-03 | Projectile override applied via `AssetSwapSystem` or a new `ProjectileSwapSystem` during the Fight group. |
| F-04 | At least 3 projectile visual variants per mod: light infantry, heavy, missile/rocket. |
| F-05 | Projectile bundles built with Unity 2021.3.45f2 and stored in `packs/<id>/assets/bundles/`. |
| F-06 | `projectile.schema.json` or extension to `unit.schema.json` validates `projectile_visual`. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | Projectile swap is event-driven (Fight group OnUpdate), not per-frame polling. |
| N-02 | All DINO entities queried with `EntityQueryOptions.IncludePrefab`. |
| N-03 | Default projectile fallback must not throw — Pattern #104 (no catch-swallow). |

## Engine Quirks / Dependencies

- `ProjectileDataBase` ECS component key: confirm via `dinoforge component-map` before
  implementing the swap hook.
- Projectile visual replacement timing: must fire at projectile spawn, not mid-flight
  (DINO's Fight group spawns projectiles during `AttackCooldown` resolution).
- Depends on SW-003 (real asset bundles) — projectile bundles follow the same pipeline.
- SW-010 (naval) and SW-011 (aerial) both consume projectile visuals as soft dependencies.

## Definition of Done

- [ ] ≥ 3 projectile visual variants per mod (infantry bolt, heavy bolt/round, missile).
- [ ] In-game screenshot/recording showing themed projectiles during combat (external judge receipt).
- [ ] Default fallback confirmed: no crash when projectile bundle missing.
- [ ] `ProjectileSwapRegistry` has unit tests.
- [ ] Schema extension for `projectile_visual` validated by `PackCompiler`.
- [ ] `dotnet test` green.

## Evidence Requirements

| Requirement ID | Evidence Type | Artifact Path Pattern | Transition Gate |
|----------------|---------------|-----------------------|-----------------|
| EPIC-027-FR-016 | ManualAttestation | `docs/proof/judge-receipts/SW-009-projectiles.md` (themed projectiles visible during combat per mod) | Implementing → Validated |
| EPIC-027-NFR-005 | CiOutput | CI build log (netstandard2.0 TFM check) | Implementing → Validated |
| EPIC-027-NFR-006 | ManualAttestation | Bundles built with Unity 2021.3.45f1 load under BepInEx 5.4.x (log confirmation) | Implementing → Validated |
| EPIC-027-NFR-013 | CiOutput | `LogOutput.log` grep: no TypeLoadException after clean launch | Implementing → Validated |
| EPIC-027-NFR-014 | TestResult | `docs/test-results/SW-009/ProjectileSwapRegistryTests.xml` (missing-bundle fallback test) | Implementing → Validated |
| SW-009 | TestResult | `docs/test-results/SW-009/ProjectileSwapRegistryTests.xml` | Implementing → Validated |
| SW-009 | CiOutput | `PackCompiler validate` exercises `projectile_visual` schema extension | Implementing → Validated |
| SW-009 | ReviewApproval | PR URL (auto-detected from WorkPackage.pr_url) | Validated → Shipped |
| SW-009 | CiOutput | GitHub Actions run URL (dotnet test green) | Implementing → Validated |

## Related

- `src/Domains/Warfare/` (combat archetype pattern)
- `src/Runtime/Bridge/AssetSwapSystem.cs`
- SW-003 (real asset bundles — bundle pipeline)
- SW-010, SW-011 (consumers of this feature)
