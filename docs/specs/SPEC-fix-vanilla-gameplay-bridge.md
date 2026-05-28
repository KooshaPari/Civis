# SPEC: Fix Vanilla Gameplay Bridge

**Status**: Draft  
**Date**: 2026-05-25  
**Author**: Agent (iter-147 investigation)  
**Tracking**: Replaces ad-hoc issues #101, #102  

## Problem Statement

9 packs load successfully at main menu (packCount=9, success=True), registering factions, units, buildings, weapons, doctrines, economy profiles, scenarios, and stat overrides into SDK registries. However, ALL gameplay remains 100% vanilla. No stat modifications, no visual swaps, no wave composition changes, and no custom unit spawns are observable in-game.

This document traces the full chain from pack YAML to ECS entity modification for each bridge system, inventories every gap where the chain breaks, and provides a prioritized fix plan.

## Architecture Overview

```
Pack YAML --> ContentLoader --> SDK Registries --> Bridge Systems --> ECS Entities
                                     |
                  +---------+---------+---------+---------+
                  |         |         |         |         |
             StatMod   AssetSwap  WaveInj  PackSpawn  Faction
```

The bridge layer has 5 ECS systems that translate SDK registry data into live ECS entity modifications:

| System | SimGroup | Purpose |
|--------|----------|---------|
| StatModifierSystem | Simulation | Apply stat overrides to entity component fields |
| AssetSwapSystem | Presentation | Swap RenderMesh on entities (visual replacement) |
| WaveInjector | Simulation | Inject pack wave definitions as spawn sequences |
| PackUnitSpawner | Simulation | Clone vanilla archetypes to create pack-defined units |
| FactionSystem | Simulation | Maintain faction registry (logical grouping only) |

## Gap Inventory

### 1. StatModifierSystem -- PARTIAL (closest to working)

**Chain**: pack YAML `stats:` section --> `StatOverrideDefinition` --> `OverrideApplicator.ApplyStatOverrides()` --> `StatModifierSystem.EnqueueRange()` --> `OnUpdate()` processes queue --> reflection-based field write on ECS entities

**Status**: The machinery is fully implemented and well-engineered. Two independent paths exist:

- **Path A (YAML global overrides)**: `starwars_buffs.yaml` defines 4 overrides (hp multiply 1.5 on MeleeUnit, damage multiply 1.25 on MeleeUnit, hp multiply 0.8 on RangedUnit, speed multiply 1.2 on RangedUnit). These are loaded via `ContentLoader.LoadedOverrides` and enqueued into StatModifierSystem.

- **Path B (PackStatInjector per-unit)**: For each unit with `vanilla_mapping`, resolves to ECS component type via `PackStatMappings`, then calls `StatModifierSystem.ApplyImmediate()` to write individual stat fields. This runs in `RebuildCatalogAndApplyStats` after entity count exceeds 1000.

**Gaps found**:

| Gap | Severity | Detail |
|-----|----------|--------|
| **G1: `unit.stats.damage` has NO ComponentMapping** | HIGH | The starwars_buffs.yaml override targets `unit.stats.damage` with `mode: multiply`. ComponentMap has NO entry for `unit.stats.damage` -- only `projectile.damage` (maps to `Components.RawComponents.ProjectileFlyData`). StatModifierSystem silently skips unmapped paths (line 379: "No ComponentMapping... skipping (not retryable)"). The damage override is silently discarded every load. |
| **G2: Filter `Components.RangedUnit` does not exist** | HIGH | starwars_buffs.yaml uses `filter: Components.RangedUnit` but the actual game component is `Components.RangeUnit` (no 'd'). `EntityQueries.ResolveComponentType("Components.RangedUnit")` returns null, causing the filter resolution to fail. The hp/speed overrides for ranged units are silently discarded. |
| **G3: 1800-frame delay (30 seconds)** | MEDIUM | StatModifierSystem waits 1800 frames (~30s at 60fps) before processing any queued modifications. If the game scene loads entities within 10-15 seconds, stats are applied ~15-20 seconds after entities are already visible and fighting with vanilla stats. However, `PackStatInjector.Apply()` uses `ApplyImmediate()` which bypasses this delay. The delay only affects Path A (YAML overrides). |
| **G4: Stat overrides apply to ALL entities of a component type** | DESIGN | `filter: Components.MeleeUnit` applies the 1.5x HP multiplier to ALL melee units (player AND enemy). There is no per-faction or per-unit-type filtering. This is documented in OverrideApplicator comments but means the YAML override system cannot target specific units -- it is a global balance knob only. |

**What works**: The reflection pipeline (GetComponentData/SetComponentData via MethodInfo), ComponentMap resolution for hp/armor/speed/attack_cooldown/range, IncludePrefab on queries, retry logic, PackStatInjector per-unit application with correct filter component types.

### 2. AssetSwapSystem -- BROKEN (two critical bugs)

**Chain**: unit YAML `visual_asset:` --> `ContentLoader.RegisterAssetSwaps()` --> `AssetSwapRegistry.Register()` --> `AssetSwapSystem.OnUpdate()` drains pending --> `ApplySwap()` --> Phase 1 (disk patch) + Phase 2 (entity RenderMesh swap)

**Status**: Phase 1 (disk patch) has limited applicability. Phase 2 (live entity swap) is completely broken.

**Gaps found**:

| Gap | Severity | Detail |
|-----|----------|--------|
| **G5: Entity query MISSING `IncludePrefab`** | CRITICAL | `AssetSwapSystem.cs` line 361-362: `EntityManager.CreateEntityQuery(new EntityQueryDesc { All = queryComponents })` -- NO `Options = EntityQueryOptions.IncludePrefab`. Since ALL DINO entities are ECS Prefab entities, this query returns 0 entities every time. The swap loop at line 431 iterates over an empty array. This is the #1 reason visual swaps never apply. |
| **G6: 12/30 Star Wars bundles are 90-byte stubs** | HIGH | Known issue per MEMORY.md. Even if G5 were fixed, most bundles contain no actual mesh/material data, so `LoadAsset<Mesh>` / `LoadAsset<Material>` returns null. |
| **G7: HRV2 bail-out may fire on HRV1 games** | MEDIUM | Lines 322-325: If `ResolveRenderMeshType()` resolves a type name matching the HRV2 list, the swap is skipped. DINO 2021.3 should use HRV1 (`Unity.Rendering.RenderMesh`) but this has not been confirmed in-game. If the rendering assembly exposes both types, the wrong one could be resolved first. |
| **G8: Phase 1 disk patch address mismatch** | LOW | Mod packs use bundle filenames as `AssetAddress` (e.g. `sw-clone-trooper-republic`), but the Addressables catalog uses Unity-internal keys. Catalog lookup fails silently (line 238: "address not in catalog -- skipping disk patch"). This is expected per comments but means Phase 1 never succeeds for pack bundles. |

### 3. WaveInjector -- FUNCTIONAL but PASSIVE (never triggered)

**Chain**: pack YAML `waves:` --> registry `_registry.Waves.Get(id)` --> `WaveInjector.QueueWave()` --> `OnUpdate()` processes queue --> `PackUnitSpawner.RequestSpawnStatic()` for each unit

**Status**: The system is fully implemented and has correct logic. However, it is entirely passive -- it only processes waves that are explicitly queued via `QueueWave()` or `QueueWaveSimple()`.

**Gaps found**:

| Gap | Severity | Detail |
|-----|----------|--------|
| **G9: No automatic wave injection** | HIGH | WaveInjector has no code to automatically replace or augment vanilla game waves with pack-defined waves. It sits idle waiting for `QueueWave()` calls that never come. The game's native wave system runs independently. Nobody calls `QueueWave()` during normal gameplay -- it is only callable from the MCP bridge, debug console, or manual code. |
| **G10: 1800-frame delay** | MEDIUM | Same 30-second delay as StatModifierSystem. Even if waves were queued, they would not start processing for 30 seconds. |

### 4. PackUnitSpawner -- FUNCTIONAL but PASSIVE (never triggered)

**Chain**: `RequestSpawnStatic(unitId, x, z)` --> queue --> `OnUpdate()` dequeues --> look up unit in registry --> `VanillaArchetypeMapper` resolves component type --> query vanilla entities --> `EntityManager.Instantiate(template)` --> set position + Enemy tag

**Status**: Fully implemented with correct IncludePrefab queries (via EntityQueries.GetUnitsByComponentType). However, like WaveInjector, it is entirely passive.

**Gaps found**:

| Gap | Severity | Detail |
|-----|----------|--------|
| **G11: No automatic spawn integration** | HIGH | PackUnitSpawner only processes manually queued requests. The game's native unit spawning (from barracks, wave spawns, etc.) runs independently and has no hook to use pack-defined units instead. There is no interception of vanilla spawn events. |
| **G12: Stat overrides not applied to spawned units** | MEDIUM | Line 212 has a comment "Queue stat modifications for the spawned unit if there are any stat overrides" but NO actual implementation follows. Spawned units are clones of vanilla templates with vanilla stats. |
| **G13: 1800-frame delay** | MEDIUM | Same 30-second delay. |

### 5. FactionSystem -- WORKING (but cosmetic only)

**Chain**: pack YAML `factions:` --> `RegistryManager.Factions` --> `FactionSystem.InitializeFactions()` --> static dictionary of FactionRuntime

**Status**: Correctly loads and registers pack factions. Provides lookup/tagging APIs.

**Gaps found**:

| Gap | Severity | Detail |
|-----|----------|--------|
| **G14: OnUpdate is empty** | LOW | The system maintains a static registry but performs no per-frame work. Entity count tracking (mentioned in comments) is not implemented. |
| **G15: No integration with game faction logic** | DESIGN | DINO uses a binary player/enemy split via `Components.Enemy` tag. Pack factions (republic, cis, west, etc.) are purely logical labels with no ECS representation. The system provides SetEntityFaction() but nothing calls it automatically. |

## Root Cause Summary

The fundamental problem is a **bridge execution gap**: the SDK layer correctly parses, validates, and registers all pack content into typed registries, but the Runtime bridge layer fails to translate that registry data into actual ECS entity modifications for three reasons:

1. **Data errors in the one system that IS wired** (StatModifier): typo in filter component name (`RangedUnit` vs `RangeUnit`), missing ComponentMap entry for `damage`, silently discarded overrides
2. **Missing IncludePrefab on AssetSwapSystem query**: returns 0 entities, all visual swaps fail silently
3. **Passive-only design for WaveInjector and PackUnitSpawner**: these systems wait for explicit API calls that never come during normal gameplay; there is no hook into the game's native spawn/wave systems

## Prioritized Fix Plan

### Fix 1: StatModifierSystem data corrections (EASIEST WIN)

**Effort**: ~30 minutes  
**Files**: `packs/warfare-starwars/stats/starwars_buffs.yaml`, `src/Runtime/Bridge/ComponentMap.cs`

**Changes**:
1. Fix typo: `Components.RangedUnit` --> `Components.RangeUnit` in starwars_buffs.yaml (2 entries)
2. Add `unit.stats.damage` mapping to ComponentMap pointing to `Components.RawComponents.ProjectileFlyData` / `damage` field (or document that damage cannot be overridden via stats and remove the override)
3. Validate all filter component names in stats YAML files match actual game component names

**User will see**: After entering a campaign/skirmish and waiting ~30 seconds, ALL melee units (player and enemy) will have 1.5x HP. ALL ranged units will have 0.8x HP and 1.2x speed. The per-unit stat injection via PackStatInjector will also apply individual unit HP/armor/speed/attack_cooldown/range values to matching vanilla archetypes.

### Fix 2: AssetSwapSystem IncludePrefab (CRITICAL, small change)

**Effort**: ~15 minutes  
**Files**: `src/Runtime/Bridge/AssetSwapSystem.cs`

**Change**: Line 362, add `Options = EntityQueryOptions.IncludePrefab` to the EntityQueryDesc.

```csharp
// BEFORE:
EntityQuery query = EntityManager.CreateEntityQuery(
    new EntityQueryDesc { All = queryComponents });

// AFTER:
EntityQuery query = EntityManager.CreateEntityQuery(
    new EntityQueryDesc { All = queryComponents, Options = EntityQueryOptions.IncludePrefab });
```

**User will see**: For the ~18/30 Star Wars bundles that contain real mesh data (not 90-byte stubs), visual model swaps will be attempted on matching entities. Success depends on G7 (HRV1 vs HRV2 resolution) and bundle content quality. At minimum, the debug log will show non-zero swap counts instead of the current silent 0/0.

### Fix 3: Replace 90-byte stub bundles (MEDIUM effort)

**Effort**: ~2-4 hours (asset pipeline work)  
**Files**: `packs/warfare-starwars/assets/bundles/*`

Rebuild the 12 stub bundles with actual mesh data using Unity 2021.3.45f2. Alternatively, mark them as known-broken and exclude from swap registration.

**User will see**: More units will receive visual swaps (up to 30/30 instead of ~18/30).

### Fix 4: Wire automatic stat injection on scene load (MEDIUM effort)

**Effort**: ~1 hour  
**Files**: `src/Runtime/Plugin.cs`, `src/Runtime/ModPlatform.cs`

The `RebuildCatalogAndApplyStats` method already calls `PackStatInjector.Apply()` which uses `ApplyImmediate()` (bypasses the 1800-frame delay). Verify this path fires reliably when entering gameplay. If the background polling thread's `_catalogRebuilt` check (entity count > 1000) races the actual entity creation, stats may apply before all units exist.

**Changes**:
1. Add a second stat injection pass after a longer delay (e.g., 3000 frames) to catch late-spawning entities
2. Log the entity count at injection time to confirm timing

**User will see**: Pack unit stats (hp, armor, speed, attack_cooldown, range) applied to vanilla entities matching each unit's `vanilla_mapping` more reliably.

### Fix 5: Automatic wave replacement (LARGE effort, future)

**Effort**: ~4-8 hours  
**Files**: New system or extension of WaveInjector

To automatically replace vanilla waves with pack-defined waves, DINOForge would need to:
1. Intercept the game's native wave spawning (likely via `Systems.WaveSystem` or similar)
2. Detect when a vanilla wave triggers
3. Substitute pack-defined wave composition

This requires reverse-engineering DINO's wave spawning internals. Until then, waves can only be manually triggered via MCP/debug.

**User will see**: Enemy attack waves composed of pack-defined units instead of vanilla units.

### Fix 6: Automatic unit substitution in spawners (LARGE effort, future)

**Effort**: ~8-16 hours  
**Files**: New system intercepting vanilla spawn pipeline

To automatically spawn pack units from barracks and wave spawners:
1. Hook into the game's unit-creation pipeline (identify the system that handles `Components.Barraks` production)
2. Intercept the entity archetype selection
3. Substitute pack unit archetypes and apply stat overrides

**User will see**: Training units from barracks produces pack-themed units with custom stats.

## Expected Outcome After Fixes 1+2

With just Fix 1 and Fix 2 (combined ~45 minutes of work):

- **Stat overrides**: All melee units get 1.5x HP; ranged units get 0.8x HP + 1.2x speed. Per-unit stats from pack YAML (e.g., Clone Trooper 125 HP, Clone Heavy 155 HP) applied to matching vanilla archetypes via PackStatInjector.
- **Visual swaps**: For the ~18 non-stub bundles, RenderMesh replacement will be attempted on matching entities. Even partial success means some units visually change appearance.
- **Waves/spawning**: Still vanilla (requires Fix 5/6). But the game world will no longer be 100% vanilla -- stats and (some) visuals will reflect pack content.

## Appendix: Component Name Reference

| Pack YAML name | Correct ECS name | Notes |
|----------------|-------------------|-------|
| Components.MeleeUnit | Components.MeleeUnit | Correct |
| Components.RangeUnit | Components.RangeUnit | Correct (no 'd') |
| Components.RangedUnit | DOES NOT EXIST | Typo in starwars_buffs.yaml |
| Components.CavalryUnit | Components.CavalryUnit | Correct |
| Components.SiegeUnit | Components.SiegeUnit | Correct |
| Components.Archer | Components.Archer | Correct |
| Components.Enemy | Components.Enemy | BlobAssetReference, not zero-sized |
