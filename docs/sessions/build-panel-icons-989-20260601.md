# Build-panel icon and UI theming surface map — DINOForge #989 (2026-06-01)

## 1) Where build-menu + HUD button icons are currently themed

`src/Runtime/UI/MainMenuThemer.cs` is the active runtime theming path for native UIs:

- `ApplyToAuxiliaryMenus` picks active aux canvases via `IsAuxSurface` (`Canvas` + interactive/selectable/content heuristics):
  - lines ~931-968, 963-967.
- `ApplyTakeoverToSurface` runs per eligible canvas and currently applies:
  - panel backdrop (`InjectSurfaceBackground`) and theme frame sprites (`ApplyFramesToSurface`) — lines ~1040-1042
  - native controls (`RestyleNativeControls`) for sliders/selectors/tabs — lines ~1045-1046
  - build/HUD icon replacements (`ReplaceBuildPanelIcons`) — lines ~1048-1049
  - font/text recolor passes.
- `ReplaceBuildPanelIcons` touches `Image.sprite` for child button icon images when:
  - image has an existing sprite,
  - image is not the button targetGraphic,
  - icon-size heuristic passes (`IsLikelyIconImage`), and
  - a SW icon map key matches button text/name.
  - lines ~1250-1265.

## 2) Where reskin/injection copies icon sprite state

- `NativeUiHelper.CopySelectableVisualState` copies donor `spriteState` and the resolved background graphic `Image.sprite` when cloning menu buttons in `NativeMenuInjector` (`resolvedImage.sprite = donorImage.sprite`) — lines ~224-246.
- `NativeMenuInjector` fallback for mods button only sets `modsButton.targetGraphic` to an existing image when absent, but never sets sprite to null — lines ~686-694.
- `MainMenuThemer` now assigns non-null build/HUD icon sprites only when mapped assets are available (`icon_build_helmet.png`, `icon_droid_head.png`, `icon_gunship.png`) and leaves native art untouched otherwise.
- `CanvasReskinner` intentionally only recolors/selectable states and never nulls button sprites (`ReskinCanvas` comments + behavior) — see `src/Runtime/UI/CanvasReskinner.cs`.
- Search for direct nulling patterns in target surfaces: no `sprite = null` writes in these paths.

## 3) Surfaces found themed vs still untouched

Themed in this pass:

- Aux/native canvases reached by `ApplyToAuxiliaryMenus` (Options/Settings/Video/etc, create/select, in-game HUD/build/pause family) via frame+control styling.
- Button icon children that match keyword heuristics and size check in build/HUD-related UIs.

Still unthemed / intentionally untouched:

- Non-button pure icon/image-only UI nodes under auxiliary canvases are not force-themed (safety-first pass; no reliable semantic signal).
- Any button label/icon combinations that do not hit keyword mapping (`build`, `factory`, `unit`, `trooper`, `clone`, `droid`, `drone`, `air`, `fighter`, `ship`) remain native.
- Build/hud surfaces with no active `assets/ui/icon_*.png` assets in the active total-conversion pack remain native (no blanking fallback).

## 4) Changes made for #989

- Added SW icon swap stage for build/HUD controls in `MainMenuThemer.ApplyTakeoverToSurface`:
  - `ReplaceBuildPanelIcons` + heuristics + selector (`IsLikelyIconImage`, `PickBuildPanelIconSprite`).
- Added non-null load guards for `icon_build_helmet.png`, `icon_droid_head.png`, `icon_gunship.png`.
- Added simple SW-themed icon source files under:
  - `packs/warfare-starwars/assets/ui/icon_build_helmet.png`
  - `packs/warfare-starwars/assets/ui/icon_droid_head.png`
  - `packs/warfare-starwars/assets/ui/icon_gunship.png`
