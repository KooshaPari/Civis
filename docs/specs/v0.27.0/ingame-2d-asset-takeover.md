# SW-007: In-Game 2D Asset Takeover

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 3 — Assets
**Story Points**: 13
**Priority**: P1

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

## Related

- `docs/design/ingame-asset-takeover-spec.md` (full strategy spec)
- `docs/design/identity-starwars.md §7` (asset manifest)
- `docs/design/identity-modern.md §7` (asset manifest)
- SW-006 (ThemeEngine — surface detection dependency)
- SW-003 (real asset bundles — 3D meshes; this story is 2D only)
