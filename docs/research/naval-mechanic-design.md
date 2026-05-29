# Naval Combat Mechanic Design for DINOForge

## Purpose

This document proposes a naval-combat extension for DINOForge in the same style as the existing aerial extension:

- Pack-driven unit tagging and mapping
- Minimal marker components
- Small, single-purpose ECS systems
- Runtime detection of the smallest viable set of world facts
- Conservative assumptions about what DINO exposes at runtime

The goal is to support classic RTS naval play:

- Ships that move only on water
- Harbors/docks that act as production and repair anchors
- Ship-vs-ship combat with clear class roles
- Ship-vs-land combat with shoreline restrictions
- Naval transport and siege pressure as a strategic layer

The design below is intentionally practical for DINOForge. It assumes the game has a navigable terrain surface, an ECS world, and mod-side access to entity transforms and component lookups, but not a purpose-built naval API.

## Reference Pattern: Aerial Extension

The current aerial extension establishes the pattern naval should follow:

- `AerialUnitComponent` is a small marker/behavior component.
- `AerialUnitMapper` attaches it from pack data.
- `AerialMovementSystem` handles motion rules.
- `AerialTargetingSystem` handles attack eligibility and target selection.
- `AerialSpawnSystem` handles initial placement and one-time world sweeps.
- `vanilla_mapping: aerial_fighter` is intentionally skipped in the stat injector, because the special behavior is handled by a dedicated system.

Naval should mirror that pattern rather than being folded into generic unit logic.

## Gameplay Goals

### Core fantasy

- Water is a strategic lane, not just visual scenery.
- Naval units are powerful at range and on open water, but constrained by geography.
- Shore defenses matter because ships can support land assaults, but cannot freely enter land combat zones.
- Harbors are economic and military footholds, not just decorative ports.

### RTS design goals

The naval layer should support the standard RTS naval beats seen in games like Age of Empires and Red Alert:

- Early patrol boats for map control
- Mid-tier warships for line combat
- Siege ships or bombard ships for shoreline pressure
- Transports for amphibious maneuvers
- Harbors/docks as production and repair anchors
- Anti-ship, anti-structure, and anti-air specialisation
- Chokepoints, island play, and coastline denial

## Recommended Pack Surface

### Unit tags

Use behavior tags to activate naval behavior:

- `Naval` marks any water-bound combat unit.
- `NavalTransport` marks a transport or landing craft.
- `NavalSiege` marks ships specialized for shoreline or structure damage.
- `NavalAntiAir` marks ships with anti-air capability.
- `HarborBound` marks units that must be produced or repaired at a harbor/dock anchor.

The base tag should be `Naval`; the others are optional role tags.

### Suggested unit classes

The pack-side `unit_class` should get a naval family in the same style as the aerial mapping:

- `NavalScout`
- `NavalGunboat`
- `NavalDestroyer`
- `NavalCruiser`
- `NavalBattleship`
- `NavalTransport`
- `NavalSiegeShip`
- `NavalHarborDefense`

These names are suggested pack classes, not game vanilla IDs.

### Vanilla mapping

Follow the existing bridge convention:

- `vanilla_mapping: naval_*`

Recommended mapping values:

- `naval_scout`
- `naval_gunboat`
- `naval_destroyer`
- `naval_cruiser`
- `naval_battleship`
- `naval_transport`
- `naval_siege`
- `naval_harbor`

This mirrors the aerial pattern where special behavior is expressed through a dedicated mapping family rather than forcing it through generic land-unit archetypes.

### Pack data block

Add a `naval` block to `UnitDefinition`, analogous to `aerial`, for naval-specific motion and rule tuning.

Suggested fields:

- `DraftHeight` - vertical offset above water surface
- `WaterSpeedMultiplier` - baseline speed on water
- `TurnRate` - steering responsiveness
- `Acceleration` - optional move ramp-up
- `HarborRepairRate` - repair rate when docked or anchored
- `CanBeach` - whether the unit can enter shallow water / beach zones
- `CanEnterShallowWater` - more explicit water-depth gate
- `RequiresHarborForProduction` - production gate
- `PreferredRangeBand` - optional role hint for ranged ships

If the project wants to avoid a second top-level config block, these can live under `stats` plus a small `naval` subsection the same way aerial uses its own subsection.

## ECS Architecture

The safest implementation is a dedicated naval subsystem namespace, parallel to `Aviation`.

### New components

#### `NavalUnitComponent`

Marks a unit as water-bound and carries its water-motion parameters.

Suggested fields:

- `DraftHeight`
- `WaterSurfaceOffset`
- `CruiseSpeed`
- `TurnRate`
- `CanBeach`
- `CanEnterShallowWater`
- `IsDocked`
- `IsMovingOnWater`

This is the naval counterpart to `AerialUnitComponent`.

#### `HarborComponent`

Marks a building or static structure as a harbor/dock anchor.

Suggested fields:

- `DockRadius`
- `RepairRadius`
- `ProductionRadius`
- `DockSlots`
- `OwnerFactionId` if faction-correlation exists

This should be attached to harbor/dock buildings from pack metadata or a dedicated building mapping.

#### `WaterBoundComponent`

Optional helper component for generic water-bound entities.

Use this if the game has non-combat water entities such as fishing boats, civilian craft, or scripted transports.

#### `ShallowWaterOnlyComponent`

Optional restriction marker for coastal craft.

Use this for boats that can enter shallow water but not deep water.

#### `NavalTargetingComponent`

Optional targeting metadata for role-specific ship logic, such as:

- prefers naval targets
- prefers shore targets
- can bombard buildings
- can engage air units if `NavalAntiAir` is set

This is useful if target priorities need to differ from generic ground combat.

#### `DockedStateComponent`

Tracks whether a ship is docked at a harbor and therefore eligible for repair, reload, or production support.

### New systems

#### `NavalUnitMapper`

Responsible for attaching `NavalUnitComponent` and related role components based on pack YAML tags.

Responsibilities:

- Detect `behavior_tags: [Naval]`
- Read `naval` subsection values
- Attach `NavalUnitComponent`
- Attach `NavalTargetingComponent` when role tags indicate siege, transport, or anti-air
- Attach `WaterBoundComponent` or `ShallowWaterOnlyComponent` if needed

This should mirror `AerialUnitMapper` in structure and error handling.

#### `NavalSpawnSystem`

Responsible for placing ships correctly when they spawn.

Responsibilities:

- Snap spawn position to the local water surface
- Prevent ground-spawn artifacts
- Set initial draft offset
- Optionally reject invalid inland spawns unless a harbor anchor is present

This is the naval analogue of the aerial altitude initialization path.

#### `NavalMovementSystem`

Responsible for water-bound movement constraints.

Responsibilities:

- Keep ships constrained to water tiles
- Follow water-surface height and orientation
- Prevent movement onto dry land except at legal beach/shallow-water transitions
- Apply reduced speed or turning penalties in shallow water
- Support docking state when within harbor radius

This system should be the only place that directly mutates the Y/vertical position of naval units, just as aerial movement owns altitude.

#### `NavalTargetingSystem`

Responsible for target acquisition and combat selection.

Responsibilities:

- Ship-vs-ship engagement on water
- Ship-vs-land bombardment at shoreline or structure targets
- Anti-air engagement for ships with AA capability
- Transport avoidance rules unless escort behavior is desired

Targeting should be role-based:

- Gunboats prefer light ships and transports
- Destroyers prefer subs or anti-ship targets if those exist later
- Cruisers can target shore and medium ships
- Battleships prefer structures and shore targets

#### `HarborInteractionSystem`

Responsible for docking, repair, reload, and production gating.

Responsibilities:

- Detect ships inside harbor radius
- Set `DockedStateComponent`
- Freeze or heavily reduce movement while docked
- Allow repair or resupply if the pack/game supports it
- Unlock naval production only when a harbor exists

#### `WaterQuerySystem` or `TerrainProbeService`

Responsible for detecting whether a world position is water.

This is the key integration seam. Do not bury water detection inside movement or spawning logic; keep it as a service or helper so the implementation can be swapped later.

## Water Detection in DINO ECS

### Constraint

The design needs a reliable answer to a simple question:

> Is this world position water?

The repo currently shows a pattern for reflection-based ECS component resolution. It does not yet expose a clear water API in the code paths reviewed here, so the naval design should support multiple detection strategies.

### Preferred detection order

1. Use a native terrain/water component if DINO exposes one.
2. Use a map tile query if the game has tile ownership or terrain-type data in ECS.
3. Use a raycast/probe against the rendered water surface if terrain data is unavailable.
4. Fall back to a pack-authored water mask or naval zone metadata if the game exposes none of the above.

### Possible ECS surfaces

Depending on what DINO exposes, the runtime could resolve one of these patterns:

- `Components.Water`
- `Components.TerrainWater`
- `Components.TileWater`
- `Components.Shoreline`
- `Components.NavMeshWater`
- `Components.BuildableWater`

If none exist, use a bridge helper that samples the terrain and returns a boolean for a given position.

### Practical implementation shape

Build a small `WaterProbe` abstraction with methods like:

- `bool IsWater(float3 position)`
- `bool IsShallowWater(float3 position)`
- `float SampleWaterSurfaceY(float3 position)`
- `float SampleDepth(float3 position)`

The naval systems should consume the abstraction, not a concrete game-specific component type.

### Why this matters

Naval movement requires more than pathfinding:

- Spawn placement must know whether a point is legal.
- Movement must know whether a step leaves water.
- Docking must know whether a harbor sits on navigable coastline.
- Ship combat range needs to understand shoreline geometry for land bombardment.

## Ship-vs-Ship Combat Model

### Core rules

- Ships target ships first when in open water.
- Short-range ships win at close distances.
- Long-range ships dominate open lanes.
- Bigger ships are slower but have better survivability and better shore bombardment.

### Recommended ship classes

#### Scout boat

- Fastest naval unit
- Low cost, low health
- Good vision and map scouting
- Weak combat value

#### Gunboat / patrol boat

- Early anti-ship unit
- Good against scouts and transports
- Weak against fortifications

#### Destroyer

- Mid-game anti-ship specialist
- Strong against smaller craft
- Can carry light anti-air or anti-sub role if the game later adds it

#### Cruiser

- General-purpose artillery ship
- Can pressure shoreline defenses
- More expensive and slower than destroyers

#### Battleship

- Long-range siege platform
- Very vulnerable if isolated
- Key unit for breaking fortified coasts

#### Transport

- Non-combat or lightly armed
- Carries land units to beachheads
- Should avoid front-line naval fights

### Target priority suggestions

Give each class a clear target bias:

- Scout boat: scouts and transports
- Gunboat: light ships
- Destroyer: medium ships, transports
- Cruiser: shore defenses, medium ships
- Battleship: buildings, shore batteries, clustered units
- Transport: flee/avoid combat

This preserves the classic RTS triangle of visibility, control, and siege pressure.

## Ship-vs-Land Combat Model

### Shore bombardment

Naval units should be able to attack land targets only when all of the following are true:

- The ship has a bombard-capable role
- The target is within range from the water edge
- The target is on or near a reachable shoreline segment
- Line-of-sight rules, if any, do not block the attack

### Land target categories

Ships should be able to hit:

- Coastal buildings
- Harbor/dock structures
- Towers and fixed defenses
- Beachhead infantry and artillery

Ships should usually not be able to:

- Chase deep inland targets
- Path onto land to finish a fight
- Ignore terrain constraints to attack arbitrary ground units

### Design balance

To keep naval play readable:

- Land bombardment should be strong but situational.
- Battleships should have high structure pressure but poor self-protection.
- Cruisers should be flexible but not dominant against every target.
- Coastal anti-ship defenses should punish overextension.

## Harbors and Docks

### Role in the economy

Harbors should serve as:

- Naval production buildings
- Repair and resupply anchors
- Transport embark/disembark points
- Control points on water-heavy maps

### Harbor behavior

A harbor/dock should:

- Spawn naval units only if adjacent water exists
- Repair docked ships over time
- Unlock advanced naval classes or tech tiers
- Serve as a rally-point anchor for ships

### Harbor placement rules

The harbor must verify:

- At least one adjacent water tile
- A valid shoreline or dock face
- Enough clearance for spawn radius
- No blocking terrain or forbidden slope

### Harbor combat value

Harbors should not be passive production nodes only.

Good RTS harbor design includes:

- Limited defensive capability
- Vulnerability to bombardment
- Strategic importance because losing a harbor cuts naval reinforcement

## Spawning and Placement Rules

Naval units should not spawn the way land units do.

### Spawn rules

- Spawn on or adjacent to water only
- If produced from harbor, spawn at the harbor exit or dock slip
- If spawned by script, snap to nearest legal water tile
- Reject invalid inland spawns unless the unit is amphibious

### Placement resolution

Recommended spawn fallback order:

1. Use harbor exit point if available
2. Use nearest legal water surface position
3. Use shallow-water fallback for coastal craft
4. Reject spawn if no valid water exists

### Initial facing

Ships should orient along the water channel or shoreline tangent where possible, not always toward world north.

## Amphibious Edge Cases

Not every naval unit should be hard-restricted to deep water.

Support two special cases:

- Coastal craft that can operate in shallow water and beach-adjacent areas
- Landing craft or transports that can temporarily touch beach zones for unloading

This allows more interesting map play without turning every ship into a land unit.

Recommended model:

- Deep-water ships: cannot enter shallow water or land-adjacent grids
- Shallow-water boats: can enter shallow water, but not land
- Amphibious transports: can beach briefly for unload behavior

## Pack Validation Rules

Add validation so packs stay coherent.

### Unit validation

If `behavior_tags` contains `Naval`, require:

- `vanilla_mapping` starts with `naval_`
- `unit_class` is one of the naval classes
- `naval` block exists, or defaults are accepted explicitly

If `behavior_tags` contains `NavalTransport`, require:

- Transport capacity metadata, if the game exposes it
- A valid embark/disembark pattern

If `behavior_tags` contains `HarborBound`, require:

- A matching harbor or dock production chain

### Building validation

If a building declares harbor behavior, require:

- Dock radius or exit point
- Water adjacency compatibility
- Production or repair role definition

### Rejection cases

Reject packs that try to:

- Mark a land unit as naval without a water-facing mapping
- Use `naval_*` mapping while leaving the class in a land-only family
- Declare a harbor with no adjacent water requirement

## Suggested Runtime File Layout

Keep naval code parallel to the aerial namespace:

- `src/Runtime/Naval/NavalUnitComponent.cs`
- `src/Runtime/Naval/NavalMovementSystem.cs`
- `src/Runtime/Naval/NavalTargetingSystem.cs`
- `src/Runtime/Naval/NavalSpawnSystem.cs`
- `src/Runtime/Naval/NavalUnitMapper.cs`
- `src/Runtime/Naval/HarborComponent.cs`
- `src/Runtime/Naval/HarborInteractionSystem.cs`
- `src/Runtime/Naval/WaterProbe.cs`

If a shared terrain probe belongs elsewhere, place it under `src/Runtime/Bridge/` or a new `Terrain/` helper namespace, but keep it reusable by both movement and spawning.

## Suggested SDK Additions

The pack models should gain a naval subsection similar to aerial:

- `UnitDefinition.Naval`
- `NavalProperties` model

Possible `NavalProperties` fields:

- `DraftHeight`
- `WaterSurfaceOffset`
- `TurnRate`
- `Acceleration`
- `HarborRepairRate`
- `CanBeach`
- `CanEnterShallowWater`
- `RequiresHarborForProduction`

If production gating is handled at the building level instead, keep the unit block focused on movement and combat only.

## SOTA RTS Naval Design Notes

### Age of Empires style lessons

- Navies are map-control tools on water-heavy maps.
- Transports create strategic landings and force defensive spread.
- Siege ships matter because shoreline fortifications otherwise dominate water lanes.
- Harbors create a second production frontier, not just a dock animation.

### Red Alert style lessons

- Naval units should feel fast, lethal, and decisive.
- Surface ships need clear counters, especially through shore defenses and special-role vessels.
- Naval force projection should threaten economy and production, not just combat units.
- Water mobility should create unique flanking routes and base pressure options.

### What to avoid

- Ships that behave like floating land tanks
- Harbors that are just reskinned barracks
- Infinite shore bombardment with no counterplay
- Amphibious movement that ignores all terrain logic
- A single universal ship class that does every job

## Minimal Implementation Slice

If this is implemented in phases, the first production-safe slice should be:

1. `NavalUnitComponent`
2. `NavalUnitMapper`
3. `NavalMovementSystem`
4. `NavalTargetingSystem`
5. `WaterProbe` abstraction
6. `naval_*` vanilla mapping support

Second phase:

1. `HarborComponent`
2. `HarborInteractionSystem`
3. production gating and repair
4. transport and shoreline edge cases

Third phase:

1. deep-vs-shallow water distinctions
2. beaching rules
3. amphibious transport logic
4. specialized naval tech and balance tuning

## Open Technical Risk

The biggest unknown is how DINO exposes water and coastline data at runtime.

If there is no reliable ECS terrain surface query, naval movement will need one of:

- reflection-based lookup of a native water component
- geometry or raycast probing
- pack-authored water masks / zone metadata

The rest of the system can still be designed now, but the water-detection seam should be treated as the hard dependency.

## Summary

Naval combat should be implemented as a parallel ECS feature family, not as a special case inside ground combat.

The most robust design is:

- `NavalUnitComponent` for water-bound behavior
- `NavalMovementSystem` for water constraints and surface following
- `NavalTargetingSystem` for ship-vs-ship and ship-vs-land combat
- `HarborComponent` plus `HarborInteractionSystem` for docks and repairs
- a small `WaterProbe` abstraction for terrain detection
- pack support via `behavior_tags: [Naval]` and `vanilla_mapping: naval_*`

That keeps the architecture consistent with the existing aerial extension while leaving enough room for classic RTS naval identity.
