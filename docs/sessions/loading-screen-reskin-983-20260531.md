# Loading-screen reskin findings for #983 (2026-05-31)

## Scope inspected
- `src/Runtime/UI/LoadingScreenController.cs`
- `src/Runtime/UI/MainMenuThemer.cs`
- `src/Runtime/UI/NativeMenuInjector.cs`

## What is currently themeable from `src/Runtime`
- `LoadingScreenController` builds and shows a full-screen overlay during `InitialGameLoader` and during DINOForge pack load.
- Theme input is read from `pack.yaml` by direct text scanning (first enabled `type: total_conversion` pack that has `ui_theme.loading_screen.background` or `ui_theme.loading_background`).
- The overlay now applies SW-themed values for:
  - backdrop/backdrop-overlay image
  - title/subtitle/tip/progress text colors
  - progress track/fill/shimmer colors
  - font family / font-file fallback (`ui_theme.loading_screen|ui_theme` font fields)
  - sorting order raised to `32000` so it cleanly covers the native loader while active

## Native loading symbol reachability
- I found **no runtime reference** in the above files to a native Unity `MonoBehaviour`/`GameObject` controlling DINO’s own loading bar/progress UI.
- `LoadingScreenController` does **not** query a native loading component or scene object by type/name; it injects its own overlay.
- `MainMenuThemer` and `NativeMenuInjector` only target main-menu and mod-button flows.

## Native symbol still needed (if native loading UI should be directly themed)
- A project-accessible native loader symbol is needed (type + hierarchy handle), e.g.:
  - a concrete `MonoBehaviour` type for the native loading root in `InitialGameLoader`, and
  - child references for the native progress track/fill/text fields.
- Without that symbol, direct native-style progress bar override must remain a forked overlay pass.

## Additional findings
- MODS active/select state is still unstyled in the native menu path (active/highlighted state visuals remain using native/default state colors on injected MODS UI in the current implementation).
- Native symbols for load progress and mods active/select are not surfaced together in one reachable symbol chain.