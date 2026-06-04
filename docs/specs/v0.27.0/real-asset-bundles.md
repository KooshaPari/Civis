# SW-003: Real Asset Bundles (Kill #101 Stub-Bundle Regression)

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 1 — Foundation
**Story Points**: 13
**Priority**: P0 — Sprint blocker

---

## User Story

As a **mod player**, I want the Star Wars and Modern Warfare packs to load real 3D unit
models in-game — not the 90-byte stub bundles that cause 0/36 units to render with mod
visuals — so that units look like they belong to the theme.

## Background

Issue #101 (open since M5): `warfare-starwars` ships 12 of 30 unit bundles as 90-byte stub
files. `AssetSwapSystem` Phase 2 (live entity swap) finds these bundles, attempts
`bundle.LoadAllAssets()`, gets nothing, and leaves all 36 matched entities with vanilla DINO
models. The same problem affects `warfare-modern` units that have no bundle yet.

Root cause: the content pipeline from GLB → stylize → Unity import → AssetBundle build
was never fully completed for all units. The stubs were placeholders.

CLAUDE.md requirement: AssetBundles MUST be built with **Unity 2021.3.45f2** (not any other
version) to be loadable by DINO's Mono CLR 4.0 runtime.

## Acceptance Criteria

### Scenario 1 — All SW unit bundles produce at least one loadable prefab

**Given** the `warfare-starwars` pack is deployed and a gameplay session begins,
**When** `AssetSwapSystem` scans matched entities (≥36 entities expected based on M5 analysis),
**Then** `bundle.LoadAllAssets()` returns at least 1 asset for each of the 30 unit bundles
(no 90-byte stubs remain).
**And** `dinoforge verify-mod --pack warfare-starwars` reports 0 stub-bundle errors.

### Scenario 2 — SW units render mod visuals in gameplay

**Given** all SW bundles are real and deployed,
**When** a Star Wars gameplay session is active and units are spawned,
**Then** at least 24 of 36 matched entities display non-vanilla mesh
(external judge screenshot confirms clone trooper or droid geometry, not vanilla DINO units).
**And** `BepInEx/dinoforge_debug.log` shows `[AssetSwap] Phase2 swap: <entity_id> → sw-rep-<unit>` 
for at least 24 entities.

### Scenario 3 — Modern Warfare key units have real bundles

**Given** `warfare-modern` is deployed,
**When** a Modern Warfare gameplay session is active,
**Then** at least 12 of 20 matched entities display mod mesh geometry (tanks, infantry, artillery).

### Scenario 4 — Bundle version mismatch is detected, not silently broken

**Given** an AssetBundle was built with the wrong Unity version (not 2021.3.45f2),
**When** `AssetSwapSystem` attempts to load it,
**Then** a `[AssetSwap] WARNING: bundle version mismatch — skipping` message appears in the log
and the entity retains the vanilla model (no crash, no exception swallowing — Pattern #111).

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | Run the full asset pipeline (`asset import → validate → optimize → generate → build`) for all remaining stub-bundle units in both packs. |
| F-02 | Every bundle file > 90 bytes after the pipeline completes. |
| F-03 | `AssetSwapRegistry` maps all SW unit IDs to their bundle keys. |
| F-04 | `AssetSwapSystem` Phase 2 swap log shows > 0 successful swaps per gameplay session. |
| F-05 | CI check: `scripts/ci/detect_stub_bundles.py` fails if any bundle < 1 KB in either pack. |
| F-06 | TMP_FontAsset bundles for SW-005 also built in Unity 2021.3.45f2 and tracked here. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | Bundle build process documented in `docs/guide/asset-bundle-build.md`. |
| N-02 | Bundle filename convention: `<pack-id>-<unit-id>` (e.g. `sw-rep-clone-trooper`). |
| N-03 | Pattern #233 (stale obj/ cache): after any TFM change, clean obj/ + dotnet clean before claiming bundles deployed. |

## Asset Pipeline Steps (ordered)

Per CLAUDE.md "Asset Pipeline Governance":
1. `PackCompiler assets import <pack>` — GLB/FBX → JSON
2. `PackCompiler assets validate <pack>`
3. `PackCompiler assets optimize <pack>` — LOD generation
4. `PackCompiler assets generate <pack>` — prefab generation
5. Unity 2021.3.45f2 manual step: open project, import prefabs, build AssetBundles to
   `packs/<id>/assets/bundles/`
6. `PackCompiler assets build <pack>` — full pipeline + tests
7. Verify bundle size > 1 KB each.

## Engine Quirks / Dependencies

- Unity 2021.3.45f2 is a manual step — no CI automation for the Unity editor phase.
  Document the exact Unity project path and export settings.
- `bundle.LoadAllAssets()` fallback handles name mismatches (CLAUDE.md "AssetSwapSystem
  name-mismatch fallback") — ensure this path is tested with real bundles.
- All DINO entities are ECS Prefab entities — every `EntityQuery` MUST use
  `EntityQueryOptions.IncludePrefab` or returns 0 results.
- Phase 2 swap requires `SetSharedComponentData` — only valid during Simulation/Fight
  system groups, not main menu.

## Definition of Done

- [ ] 0 stub bundles (< 1 KB) in `warfare-starwars/assets/bundles/`.
- [ ] 0 stub bundles in `warfare-modern/assets/bundles/` for units that have completed pipeline.
- [ ] In-game screenshot: ≥24/36 SW entities showing clone/droid geometry (external judge receipt).
- [ ] `dinoforge verify-mod --pack warfare-starwars` exits 0.
- [ ] CI stub-bundle detector added to `scripts/ci/detect_stub_bundles.py`.
- [ ] TMP_FontAsset bundles for SW-005 present and loadable.
- [ ] `dotnet test` green.

## Related

- `docs/milestones/MILESTONE-M5-example-packs.md` (#101 tracking)
- CLAUDE.md "Asset Bundle Creation" and "Asset Pipeline Governance"
- `src/Runtime/Bridge/AssetSwapSystem.cs`
- Pattern #233 (stale obj/ cache), Pattern #102 (orphan process handles during bundle build)
