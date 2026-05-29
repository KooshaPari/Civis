# SW-007: In-Game 2D Asset Takeover

**Status**: Proposed
**AgilePlus WP State**: planned
**Sequence**: 7
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**AgilePlus Feature Slug**: epic-027-full-conversion
**Sprint**: 3 — Assets
**Story Points**: 13
**Priority**: P1
**File Scope**:
  - `src/SDK/Assets/SpriteSwapRegistry.cs`
  - `src/Runtime/Bridge/TcSpriteLoader.cs`
  - `src/Runtime/UI/TcUiSpritePass.cs`
  - `src/Runtime/UI/TcCursorApplicator.cs`
  - `src/Runtime/Bridge/AddressablesSpritePatch.cs`
  - `src/SDK/Models/TotalConversionManifest.cs`
  - `schemas/total-conversion.schema.json`
  - `docs/reference/dino-sprite-key-map.yaml`
  - `docs/reference/dino-ui-atlas-spec.yaml`
  - `src/Tests/SpriteSwapRegistryTests.cs`
  - `src/Tests/TcSpriteLoaderTests.cs`
**Depends On**: [SW-006-P1]
**Requirements**: EPIC-027-FR-013, EPIC-027-FR-014, EPIC-027-NFR-002, EPIC-027-NFR-005, EPIC-027-NFR-006, EPIC-027-NFR-008, EPIC-027-NFR-011, EPIC-027-NFR-013, EPIC-027-NFR-014, EPIC-027-NFR-015, EPIC-027-NFR-020

---

## User Story

As a **mod player**, I want faction emblems, unit portrait icons, HUD sprite panels, and the
game cursor to be replaced by my total-conversion pack's artwork — so that no vanilla DINO
2D art is visible during a full gameplay session.

## Background

Full design spec: `docs/design/ingame-asset-takeover-spec.md`. Three interception strategies:
- **Strategy A** (Addressables Harmony Prefix): intercepts `LoadAssetAsync<Sprite>` by key —
  prevents vanilla texture from ever loading into GPU memory. Requires key-discovery pass.
- **Strategy B** (post-load component walk): extends existing `MainMenuThemer`-style scan;
  replaces `Image.sprite` on identified UGUI components. Works immediately, no key-discovery.
- **Strategy C** (cursor): `Cursor.SetCursor()` from `Plugin.Awake()`.

v0.27.0 delivery: **Strategy B** (Steps 1–4) + **Strategy C** as first release; Strategy A
(faction emblems + portraits via key interception) as Step 5 after key-discovery pass.

## Acceptance Criteria

### Scenario 1 — Main menu background replaced (Strategy B, Step 1)

**Given** `warfare-starwars` is active and the pack supplies `assets/ui/menu_bg.png`,
**When** the main menu scene is active,
**Then** the full-bleed background shows the mod PNG, not DINO's castle/fantasy art.
(This validates the sprite-swap path; overlaps with SW-005 verification.)

### Scenario 2 — Loading screen background replaced (Strategy B, Step 2)

**Given** `warfare-starwars` is active and the pack supplies `assets/ui/loading_bg.png`,
**When** the `InitialGameLoader` scene is active,
**Then** the loading screen shows the mod background.

### Scenario 3 — Button and panel sprites replaced (Strategy B, Step 3)

**Given** `warfare-starwars` is active and the pack supplies button-frame PNGs,
**When** the player is on the main menu and opens the build panel in gameplay,
**Then** button backgrounds and panel frames show the mod's 9-slice sprites, not DINO grey.

### Scenario 4 — Custom cursor active (Strategy C, Step 4)

**Given** `warfare-starwars` is active and the pack supplies `assets/ui/cursor.png`,
**When** the game window is focused,
**Then** the cursor shows the mod's 32×32 RGBA cursor image.
**And** the cursor is re-applied after scene changes (DINO resets cursor on scene load).

### Scenario 5 — Faction emblems replaced (Strategy A, Step 5)

**Given** the Addressables key-discovery pass has produced `docs/reference/dino-sprite-key-map.yaml`,
**And** `warfare-starwars` declares `asset_replacements.ui.keyed_sprites` for the emblem keys,
**When** gameplay loads and the HUD faction emblem appears,
**Then** the emblem shows the Republic cog (player faction) or CIS hex (enemy faction) PNG
from the mod, not DINO's vanilla faction icon.

### Scenario 6 — No GPU double-allocation for vanilla sprites replaced via Strategy A

**Given** Strategy A intercepts a `LoadAssetAsync<Sprite>` call for a keyed sprite,
**When** the intercept fires,
**Then** `SpriteSwapRegistry.GetReplacement(key)` returns the mod sprite and `return false`
prevents the original from being allocated.

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | `SpriteSwapRegistry` (SDK layer) maps `addressKey → Sprite` and `surfaceSlot → Sprite`. |
| F-02 | `TcSpriteLoader` loads PNG bytes → Texture2D → Sprite at scene-load time, populates registry. |
| F-03 | `TcUiSpritePass` walks live UGUI hierarchy to apply `TcUiSurfaces` slot replacements. |
| F-04 | `TcCursorApplicator` calls `Cursor.SetCursor` at startup and re-applies on `activeSceneChanged`. |
| F-05 | `AddressablesSpritePatch` Harmony Prefix patches `AddressablesImpl.LoadAssetAsync` for Strategy A. |
| F-06 | Sprite cache keyed by pack-relative PNG path (`StringComparer.Ordinal`); invalidated on HotReload. |
| F-07 | 9-slice border values declared in pack YAML alongside the PNG path; `Sprite.Create` uses the `Vector4 border` overload. |
| F-08 | Key-discovery output: `docs/reference/dino-sprite-key-map.yaml` published before Strategy A ships. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | Pre-load all pack sprites during `InitialGameLoader` scene (largest idle window). |
| N-02 | Loading 20 sprites at 512×512 must complete in < 30 ms. |
| N-03 | `Texture2D.LoadImage` only on the main thread. |
| N-04 | Do not call `Addressables.Release(originalHandle)` for always-resident bundle sprites. |

## Engine Quirks / Dependencies

- Strategy A requires `AddressablesImpl` type path via `AppDomain.CurrentDomain.GetAssemblies()`
  scan — confirm with `dinoforge dump` before shipping the Harmony patch.
- DINO likely uses `SpriteAtlas` — replacing the whole atlas covers all sprites in it. Atlas
  spec: `docs/reference/dino-ui-atlas-spec.yaml` (produced in the key-discovery pass).
- `Cursor.SetCursor` must be called after `Texture2D.LoadImage`; reset after scene change
  because DINO's loading code resets the cursor.
- Depends on ThemeEngine Phase 1 (SW-006) for surface identification in Strategy B.

## Definition of Done

- [ ] Strategy B Steps 1–4 implemented and screenshot-verified (menu bg, loading bg, buttons, cursor).
- [ ] Strategy A Step 5 implemented for faction emblems (external judge receipt showing mod emblems in HUD).
- [ ] `dino-sprite-key-map.yaml` published in `docs/reference/`.
- [ ] `SpriteSwapRegistry`, `TcSpriteLoader`, `TcUiSpritePass`, `TcCursorApplicator`, `AddressablesSpritePatch` all have unit tests.
- [ ] Sprite cache invalidation verified on HotReload signal.
- [ ] `dotnet test` green.

## Evidence Requirements

| Requirement ID | Evidence Type | Artifact Path Pattern | Transition Gate |
|----------------|---------------|-----------------------|-----------------|
| EPIC-027-FR-013 | ManualAttestation | `docs/proof/judge-receipts/SW-007-2d-takeover.md` (full play session: every spawnable unit shows mod portrait + faction emblems replaced in HUD, no vanilla DINO 2D art — screenshot per mod) | Implementing → Validated |
| EPIC-027-FR-014 | TestResult | `docs/test-results/SW-007/SpriteSwapRegistryTests.xml` — Strategy B (sprite swap without key-discovery) confirmed; Strategy A (Addressables intercept) activated after key-discovery pass | Implementing → Validated |
| EPIC-027-NFR-002 | CiOutput | Profiler log confirms no per-frame canvas walk from `TcUiSpritePass`; all sprite swaps applied in single frame budget at scene load | Implementing → Validated |
| EPIC-027-NFR-005 | CiOutput | CI build log (Runtime `netstandard2.0`; Strategy A Harmony Prefix uses reflection-resolved type, no compile-time Addressables ref) | Implementing → Validated |
| EPIC-027-NFR-006 | ManualAttestation | Sprite bundles (if any) built with Unity 2021.3.45f2 load under BepInEx 5.4.x; `Texture2D.LoadImage` for raw PNGs confirmed on main thread (log) | Implementing → Validated |
| EPIC-027-NFR-008 | CodeReview | All injected cursor/overlay objects carry `DINOForge_` prefix; no unnamed injected objects (grep `new GameObject` in TcCursorApplicator + TcUiSpritePass) | Implementing → Validated |
| EPIC-027-NFR-011 | SchemaValidation | `PackCompiler validate` rejects a manifest with `..` or absolute-path asset references in `asset_replacements.ui` | Implementing → Validated |
| EPIC-027-NFR-013 | CiOutput | `LogOutput.log` grep: no `TypeLoadException` after clean launch with Strategy A Harmony Prefix active | Implementing → Validated |
| EPIC-027-NFR-014 | TestResult | `docs/test-results/SW-007/TcSpriteLoaderTests.xml` — missing/failed sprite asset falls back to vanilla, warning logged, no crash | Implementing → Validated |
| EPIC-027-NFR-015 | CodeReview | Overlay Image components in `TcUiSpritePass` have `raycastTarget = false`; EventSystem guard present before any GraphicRaycaster add | Implementing → Validated |
| EPIC-027-NFR-020 | ManualAttestation | Full play session (cross-reference with FR-013 receipt): no native medieval 2D art visible with TC active (judge receipt per mod) | Implementing → Validated |
| SW-007 | ManualAttestation | `dino-sprite-key-map.yaml` published in `docs/reference/`; Strategy A intercepts at least one Addressables key (log confirmation + receipt) | Implementing → Validated |
| SW-007 | ManualAttestation | Sprite cache invalidation verified on HotReload signal (hot-reload + re-scan shows updated sprite, no crash) | Implementing → Validated |
| SW-007 | ReviewApproval | PR URL (auto-detected from WorkPackage.pr_url) | Validated → Shipped |
| SW-007 | CiOutput | GitHub Actions run URL (dotnet test green) | Implementing → Validated |

## Related

- `docs/design/ingame-asset-takeover-spec.md` (full strategy spec)
- `docs/design/identity-starwars.md §7` (asset manifest)
- `docs/design/identity-modern.md §7` (asset manifest)
- SW-006 (ThemeEngine — surface detection dependency)
- SW-003 (real asset bundles — 3D meshes; this story is 2D only)
