# MODS Native Widget + Native Extend + Full-Page Skin — Session Findings (2026-05-30)

Branch: `feat/mods-native-widget-v2-20260530` (base `integration/v0.27.0-reconcile-20260530`, HEAD f75c3105).

## Native UI dump findings (from source on the reconcile branch)

DINO menu buttons are a **custom `MainMenuButton : Selectable`** (NOT `UnityEngine.UI.Button`).
`GetComponentsInChildren<Button>()` returns 0 on the MainMenu canvas; injection relies on the
Selectable-donor path (`NativeMenuInjector.InjectButtonFromSelectable` →
`NativeUiHelper.CloneSelectableAsButton`).

Donor interactive state that must carry onto the injected button:
- `Selectable.transition` (DINO MainMenuButton uses ColorTint / SpriteSwap depending on skin)
- `Selectable.colors` (ColorBlock: normal/highlighted/pressed/selected/disabled)
- `Selectable.spriteState` (SpriteState: highlighted/pressed/disabled sprites)
- `Selectable.animationTriggers`
- `targetGraphic` — the background `Image` (sprite + material + type)

## Requirement status on the base branch (what already exists)

1. **State-copy (R1)**: ALREADY present.
   - `NativeMenuInjector.SyncButtonVisualStyle` (Button-clone path) copies transition/colors/
     spriteState/animationTriggers, resolves `targetGraphic` by relative path, and copies the
     BG Image sprite/type/color.
   - `NativeUiHelper.CopySelectableVisualState` (Selectable-donor path) copies transition/colors/
     spriteState/animationTriggers; `CloneSelectableAsButton` re-applies it.
   - GAP: BG Image `material` was NOT copied → a custom-material native frame loses its shader on
     the clone, which can read as a flat/non-reactive background. Fixed (see R1 fix below).

2. **Native extend (R2)**: ALREADY present.
   - MODS button is a cloned **sibling inside the native menu hierarchy** (not a separate canvas).
   - `NativeQuickModPanel` builds rows by **cloning the native donor button** and themes via
     `MenuThemeReader`, so it inherits native chrome and is reskinned downstream.

3. **Skin all non-main-menu pages (R3)**: MISSING.
   - The described `#962 ThemeEngine (ISurfaceReskinner/CanvasWalker/ThemeParser)` does **not exist**
     on this base branch (grep: no such types). The only themer is `MainMenuThemer`, which themes
     **only the MainMenu canvas** (`FindMainMenuCanvas` returns the single MainMenu canvas).
   - Settings / GAME / VIDEO / SOUND / CONTROLS / TWITCH / game-create render unskinned-native.
   - FIX: new generic `CanvasReskinner` that walks **all active canvases** (excluding MainMenu,
     which `MainMenuThemer` owns, and DINOForge-owned objects) and applies the active
     total_conversion color theme to every Selectable / Text / TMP_Text. Invoked on scene change
     and on a bounded pump retry so late-opened pages (Settings sub-tabs, game-create) get skinned.

## Fixes implemented

- **R1**: `NativeMenuInjector.CopyImageVisualStyle` now also copies `material` (+ `pixelsPerUnitMultiplier`).
  `NativeUiHelper.CopySelectableVisualState` now also copies the donor `targetGraphic`'s background
  Image material so the cloned widget's hover/press/normal frame renders identically.
- **R3**: `src/Runtime/UI/CanvasReskinner.cs` (new) + wiring in `Plugin.cs` MainMenu-init + pump retry.
</content>
</invoke>
