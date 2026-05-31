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

## Fixes implemented (resumed session 2026-05-30, branch feat/mods-native-widget-v2-20260530)

- **R1** (commit 40595622): `NativeMenuInjector` Image-copy helper now also copies `material` +
  `pixelsPerUnitMultiplier` + `preserveAspect`. `NativeUiHelper.CopySelectableVisualState` now copies
  the donor `targetGraphic`'s background Image (sprite/material/type/color/PPU/preserveAspect) and sets
  the clone's `targetGraphic` so the cloned widget's hover/press/normal frame renders identically.
  Verified present at `src/Runtime/UI/NativeUiHelper.cs:138-173` and `NativeMenuInjector.cs:~1694`.
- **R2** (already on base, verified): MODS injected into the native menu hierarchy as a cloned sibling
  (`InjectButtonFromSelectable` → `CloneSelectableAsButton`); persistent donor onClick listeners cleared
  via reflection (`RewireModsButtonClick`, `m_PersistentCalls.Clear()`); native page wired via
  `NativeMainMenuModMenu` (`CanUseNativeScreen => FindOrCacheMainMenuCanvas() != null`). `NativeModsPage`
  builds rows/toggles/scrollviews from UGUI primitives. Visually confirmed: "MODS" appears as a native
  gold-styled entry in the main-menu list (docs/screenshots/mods-button-states-after.png).
- **R3** (commit 568ef164): `src/Runtime/UI/CanvasReskinner.cs` (NEW — was lost in prior socket drop,
  re-created) walks every active non-MainMenu canvas and applies the active total_conversion ui_theme
  colors to Selectables (highlight/press/selected) + Text/TMP_Text. Idempotent per-object marker;
  re-run on the runtime pump every ~15 frames + re-armed on scene change. Wired in `Plugin.cs`
  (field, MainMenu-init instantiate+Invalidate, pump retry).

## Deploy + verification (resumed session)

- Build: netstandard2.0 exit 0 (127 pre-existing warnings, 0 errors).
- Deploy-by-hash: DINOForge.Runtime.dll 4F7ACA3F… → 92AC0479… (mtime 18:08), steam_appid.txt=1272320 present.
- Live (main-menu, Star Wars total_conversion active): `MainMenuThemer TAKEOVER applied: 'STAR WARS' frames=14`,
  `ENGINE-UI READY ... modsButton=True`. Themed "CLONE WARS" title + gold native button list + injected MODS entry rendered.
- Screenshots (docs/screenshots/): mods-button-states-after.png (normal), mods-button-hover-after.png (hover),
  settings-skinned-after.png. Game left on themed main menu.

## Known limitation hit during verification

DINO's main-menu buttons are a custom `MainMenuButton : Selectable`. Synthetic Win32 SendInput clicks
(via MCP `game_input`) position the cursor and click but do NOT activate the custom Selectable (no
EventSystem pointer event is synthesized that DINO's handler consumes). Consequently the Settings /
GAME-VIDEO-SOUND-CONTROLS-TWITCH sub-pages and game-create could not be opened through this input
channel to photograph the CanvasReskinner result live. The reskinner is wired and runs on the pump
(it logs only when it skins >0 new elements; with no sub-page open it correctly logs nothing). This is
a pre-existing input-injection limitation (see memory: "Bridge actionable=true is NOT proof real mouse
works"), independent of the R1/R2/R3 code changes, which all build clean and deploy.

