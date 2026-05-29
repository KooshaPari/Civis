# SW-008: Mod Asset Completeness (SW + Modern 100%)

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 3 — Assets
**Story Points**: 13
**Priority**: P1

---

## User Story

As a **mod player**, I want every unit, building, and UI element in the Star Wars and Modern
Warfare packs to have a complete mod asset — no vanilla DINO placeholders, no missing icons,
no grey-box units — so that the full-conversion experience is visually seamless.

## Background

M5 milestone audit (MILESTONE-M5-example-packs.md):
- `warfare-starwars`: 14 unit definitions + 9 building definitions. 12 of 30 bundles are stubs.
  SW-003 (Sprint 1) fixes the 3D bundles. This story completes the 2D asset set: portraits,
  icons, emblems, and any remaining unit definitions.
- `warfare-modern`: similar gap in 2D UI art; 3D bundles partially done.

"100% asset-complete" is defined as: every defined unit/building has (a) a 3D bundle, (b) a
64×64 portrait PNG, (c) a 16×16 role-badge PNG, and (d) correct YAML references with no
dangling pointers.

## Acceptance Criteria

### Scenario 1 — warfare-starwars: all units have portraits

**Given** `warfare-starwars` is deployed,
**When** a unit is selected in gameplay and the unit panel opens,
**Then** the portrait slot shows the mod portrait PNG for every defined unit
(no blank grey portraits, no DINO vanilla portraits).

### Scenario 2 — warfare-starwars: all units have 3D bundles (dependency on SW-003)

**Given** SW-003 (real asset bundles) is complete,
**When** `dinoforge verify-mod --pack warfare-starwars` runs,
**Then** 0 missing-bundle errors and 0 stub-bundle errors.

### Scenario 3 — warfare-modern: all units have portraits

**Given** `warfare-modern` is deployed,
**When** a unit is selected in gameplay,
**Then** every defined unit shows a Modern Warfare portrait (crosshair-frame dog-tag style per
`identity-modern.md §7.6`).

### Scenario 4 — PackCompiler validate returns 0 errors for both packs

**Given** all assets are present and referenced,
**When** `PackCompiler validate packs/warfare-starwars` and `PackCompiler validate packs/warfare-modern` run,
**Then** both exit 0 with 0 errors and 0 warnings about missing assets.

### Scenario 5 — New unit definitions complete the coverage gap

**Given** the M5 audit shows 14/28 units and 9/15 buildings covered for SW,
**When** Sprint 3 completes,
**Then** both packs cover ≥ 22/28 units and ≥ 12/15 buildings (target ≥ 80% of vanilla count;
remaining 20% is acceptable as "planned expansion" documented in the pack README).

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | Every unit YAML in both packs references an existing bundle file and portrait PNG. |
| F-02 | Portrait PNGs: 64×64 RGBA, located at `packs/<id>/assets/ui/portraits/<unit-id>.png`. |
| F-03 | Role-badge PNGs: 16×16 RGBA, located at `packs/<id>/assets/ui/badges/<role>.png`. |
| F-04 | Faction emblems (player + enemy): 128×128 RGBA at `packs/<id>/assets/ui/<faction>-emblem.png`. |
| F-05 | `PackCompiler validate` detects and reports dangling asset references as errors (not warnings). |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | All portraits follow the 9-slice-frame convention declared in the identity specs (portrait frame overlaid by ThemeEngine). |
| N-02 | All art is original geometry / OFL-licensed. No ripped Lucasfilm / EA art. |

## Asset Checklist (warfare-starwars)

| Asset | Target | Blocks |
|---|---|---|
| Unit 3D bundles | 30/30 non-stub | SW-003 |
| Unit portrait PNGs | 14/14 defined units | This story |
| Role badge PNGs (7 roles) | 7/7 | This story |
| Republic emblem (128px) | 1 | SW-005 |
| CIS emblem (128px) | 1 | SW-005 |
| Cursor PNG (32×32) | 1 | SW-007 |
| Missing unit definitions (gap to ≥22) | 8 new unit YAMLs | This story |

## Asset Checklist (warfare-modern)

| Asset | Target | Blocks |
|---|---|---|
| Unit 3D bundles | ≥ 12 non-stub | SW-003 |
| Unit portrait PNGs | all defined units | This story |
| Role badge PNGs (7 roles) | 7/7 | This story |
| Alliance emblem (128px) | 1 | SW-005 |
| Enemy emblem (128px) | 1 | SW-005 |
| Cursor PNG (32×32) | 1 | SW-007 |

## Engine Quirks / Dependencies

- Depends on SW-003 for 3D bundles.
- Portrait PNGs loaded via `TcSpriteLoader` (SW-007) using `Texture2D.LoadImage` on main thread.
- `unit_portrait_prefix` in `asset_replacements.ui.surfaces` defines the directory prefix;
  `TcUiSpritePass` resolves `<prefix><unit-id>.png` per selection event.

## Definition of Done

- [ ] `PackCompiler validate` exits 0 for both packs.
- [ ] In-game: every spawnable unit in both packs shows a mod portrait (screenshot proof).
- [ ] 0 missing-bundle errors in `dinoforge verify-mod` for both packs.
- [ ] Unit definition count: SW ≥ 22/28, Modern ≥ 18/28.
- [ ] `dotnet test` green.

## Related

- `docs/milestones/MILESTONE-M5-example-packs.md`
- `docs/design/identity-starwars.md §7`
- `docs/design/identity-modern.md §7`
- SW-003 (3D bundles prerequisite)
- SW-007 (sprite loader that populates portraits at runtime)
