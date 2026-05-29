# SW-010: Naval Combat

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 4 — Mechanics
**Story Points**: 13
**Priority**: P2

---

## User Story

As a **mod player** using `warfare-modern` or `warfare-starwars`, I want naval units —
ships, submarines, landing craft — that can be built and ordered like ground units, so that
water-terrain maps feel fully playable and not limited to ground combat.

As a **mod author**, I want a naval unit archetype and a `naval_combat` domain plugin so I
can declare naval units in pack YAML without writing ECS code.

## Background

DINO runs a terrain-aware ECS pathfinding system. Water tiles exist in the vanilla map set
but no vanilla units use them as their primary terrain. The naval domain must:
1. Register a new `terrain_affinity: water` or `terrain_affinity: amphibious` attribute.
2. Define a `NavalUnitArchetype` that hooks into DINO's existing movement system on water tiles.
3. Provide at least one ground-truth ship unit per mod before calling the story done.

## Acceptance Criteria

### Scenario 1 — Naval units buildable from build panel

**Given** a map with water tiles and `warfare-modern` active,
**When** the player opens the build panel on a coastal structure,
**Then** at least one naval unit (e.g. "Frigate" for Modern, "Venator Star Destroyer" for SW)
appears as a buildable option with correct cost and build time.

### Scenario 2 — Naval units move on water tiles

**Given** a naval unit is built,
**When** the player right-clicks a water-tile destination,
**Then** the unit pathfinds to the destination over water tiles without using land paths.

### Scenario 3 — Naval units engage enemy units in range

**Given** an enemy unit is within the naval unit's attack range (land or water),
**When** the naval unit's attack cooldown fires,
**Then** the unit fires a projectile (using themed projectile from SW-009 if complete,
otherwise the default DINO projectile) and the target takes damage per the YAML stat block.

### Scenario 4 — Ground units cannot enter deep water

**Given** a vanilla DINO ground unit (e.g. spearman),
**When** the player orders it to a deep-water tile,
**Then** the unit stops at the water's edge and a "cannot move here" indicator fires.
(This confirms naval-terrain-affinity does not break ground pathfinding.)

### Scenario 5 — Naval units appear in both mods

**Given** both `warfare-starwars` and `warfare-modern` are installed (one active at a time),
**When** each is active and a water map is loaded,
**Then** each mod's naval unit list contains at least 2 distinct unit types.

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | `NavalDomainPlugin` registers naval unit archetypes via the existing `WarfareDomainPlugin` pattern. |
| F-02 | New YAML key `terrain_affinity: [water, amphibious, land]` on unit definitions. |
| F-03 | `NavalUnitArchetype` sets the ECS movement component to water-tile-preferring pathfinding cost. |
| F-04 | Naval units declared in `packs/warfare-modern/units/naval/` and `packs/warfare-starwars/units/naval/`. |
| F-05 | Min 2 naval unit definitions per mod in v0.27.0. |
| F-06 | `naval-combat.schema.json` added to the schema set; `PackCompiler validate` checks it. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | No changes to DINO's ECS pathfinding internals; extend via component data only. |
| N-02 | Naval unit ECS components must use `EntityQueryOptions.IncludePrefab` (CLAUDE.md). |
| N-03 | `dotnet test` includes at least 5 unit tests for `NavalUnitArchetype` stat validation. |

## Engine Quirks / Dependencies

- Water tile detection: confirm water-tile component name via `dinoforge component-map` before
  implementing terrain affinity — do not guess component names.
- DINO system groups fire during `Simulation → PathFinding` groups only; naval pathfinding
  hooks must target these groups.
- Depends on the Warfare domain plugin pattern (M4) — NavalDomain is a sub-domain plugin.
- SW-009 (projectile support) is a soft dependency — naval units fall back to default
  projectiles if blaster/missile support not yet landed.

## Definition of Done

- [ ] At least 2 naval unit types per mod declared in YAML and validated.
- [ ] Naval units build, move on water, and attack in gameplay (screenshot proof).
- [ ] Ground units blocked by deep water (screenshot confirmation of pathfinding).
- [ ] `naval-combat.schema.json` added; `PackCompiler validate` exercises it.
- [ ] External judge receipt: in-game screenshot of naval unit on water tile.
- [ ] `dotnet test` green with ≥ 5 new naval archetype tests.

## Related

- `docs/design/identity-starwars.md` (naval unit concepts for SW)
- `docs/design/identity-modern.md` (naval unit concepts for Modern)
- SW-009 (projectile support — soft dependency)
- `src/Domains/Warfare/` (archetype pattern to follow)
