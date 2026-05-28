# SPEC: Fix MODS Button UX (Hover Effects + Click Target)

**Status**: Draft
**Date**: 2026-05-25
**Author**: DINOForge Agents
**Related**: [SPEC-002: Native Menu Button Injection](SPEC-002-native-menu-injector.md)

---

## Problem Statement

The injected MODS button on DINO's native main menu has two UX deficiencies:

1. **No hover/select visual feedback**: Native menu buttons (e.g., "New Game", "Load", "Options") display proprietary hover animations (red highlight, scale pulse, etc.) on mouse-over and selection. The MODS button shows no visual change on hover, making it feel dead or non-interactive compared to its siblings.

2. **Opens the wrong UI surface**: Clicking the MODS button opens the F10 UGUI overlay panel (`ModMenuPanel`) -- a floating dark panel centered on screen. Native DINO menus use a different visual language: full-width settings-style submenus that slide in and replace the button list. The MODS button's click target creates a jarring context switch.

These issues undermine the "native look and feel" goal stated in SPEC-002 requirement F-03: "The 'Mods' button shall be visually identical in style (font, colours, hover/pressed states) to the native 'Settings' button."

---

## Root Cause Analysis

### Issue A: Missing Hover/Select Effects

DINO's native menu buttons use a custom class `MainMenuButton` that inherits from `UnityEngine.UI.Selectable` (NOT from `UnityEngine.UI.Button`). This custom class implements proprietary hover animations -- likely through overridden `OnPointerEnter`/`OnPointerExit`/`OnSelect`/`OnDeselect` methods or through a custom `Animator`/`AnimationTriggers` setup tied to DINO's specific animation controller.

The injection code has two paths:

**Path 1 -- Button-typed clone** (`CloneButton` in `NativeUiHelper.cs`):
- `Instantiate(source.gameObject, ...)` clones the entire GameObject hierarchy
- `StripNonUiBehaviours(clone)` then destroys every `MonoBehaviour` whose namespace is not `UnityEngine.*`
- This strips the `MainMenuButton` component (the one that drives hover animations)
- `SyncButtonVisualStyle` copies `transition`, `colors`, `spriteState`, and `animationTriggers` from the source `Button` to the clone
- But the source is a Unity `Button` (if one exists), not the `MainMenuButton` -- so the copied `ColorBlock` contains Unity defaults, not DINO's red-highlight scheme

**Path 2 -- Selectable-donor clone** (`CloneSelectableAsButton` in `NativeUiHelper.cs`):
- Clones the donor `MainMenuButton` GameObject
- Calls `StripNonUiBehaviours` -- destroys the `MainMenuButton` component
- Calls `Object.Destroy(s)` on each Selectable matching the donor type
- Adds a fresh `UnityEngine.UI.Button` component
- Calls `CopySelectableVisualState(btn, donor)` which copies `transition`, `colors`, `spriteState`, `animationTriggers`

The problem in both paths is identical: **the `MainMenuButton` component is destroyed, and it is the component that drives the proprietary hover animation**. A plain `UnityEngine.UI.Button` with a copied `ColorBlock` can only do basic color tinting via `Selectable.DoStateTransition` -- it cannot replicate whatever custom animation logic `MainMenuButton` implements (e.g., Animator-driven scale/glow, custom shader transitions, programmatic DOTween-style interpolation).

Additionally, `Navigation.Mode` is set to `None` on the MODS button (line 1169 of `NativeMenuInjector.cs`), which prevents keyboard/gamepad selection entirely. While this was done to "isolate from native nav graph," it also prevents the button from ever entering the `Selected` state via EventSystem, which may suppress visual feedback even if the color block were correct.

### Issue B: Opens F10 Panel Instead of Native Submenu

`OnModsButtonClicked()` (line 1233) calls `_menuHost.Toggle()`, where `_menuHost` is the `ModMenuPanel` instance -- the same UGUI overlay toggled by the F10 hotkey. The code comment at line 1228 acknowledges this explicitly:

> "DESIGN NOTE: The MODS button opens the DFCanvas mod panel overlay, not a native settings-style submenu. This is intentional -- DINO has no native settings API to integrate with."

The `ModMenuPanel` is a programmatically-built UGUI panel (680x560px, dark theme, slide-in animation) that lives on the `DFCanvas` overlay canvas -- completely separate from DINO's native menu canvas. When opened from the main menu, it overlays on top of the native menu rather than replacing it, creating visual dissonance.

---

## Approach A: Make the Injected Button Match Native Hover/Select Effects

### Strategy

Preserve the `MainMenuButton` component on the clone instead of destroying it. The custom component drives the hover animations; keeping it alive restores native visual behavior.

### Implementation Plan

1. **Modify `StripNonUiBehaviours` to whitelist `MainMenuButton`-like components**:
   - Instead of destroying every non-`UnityEngine.*` MonoBehaviour, detect components whose type name contains "MainMenuButton" (or more broadly, whose type inherits from `Selectable`) and preserve them.
   - Add a type-name allowlist: `{ "MainMenuButton", "MenuButton" }` checked via `typeName.IndexOf(..., OrdinalIgnoreCase)`.

2. **In `CloneSelectableAsButton`, do NOT destroy the donor Selectable and do NOT add a fresh `Button`**:
   - Keep the `MainMenuButton` component alive on the clone.
   - Instead of adding `Button`, use the existing `Selectable` directly.
   - Wire `onClick` via an `EventTrigger` with `PointerClick` entry (since `MainMenuButton` has no `onClick` event like `Button`), OR use reflection to find and hook the custom button's click callback.

3. **In `CloneButton`, if the source has a sibling `MainMenuButton`-type Selectable, preserve it**:
   - After `Instantiate`, check for components whose type name matches `MainMenuButton`.
   - If found, skip destroying them in `StripNonUiBehaviours`.
   - The cloned `MainMenuButton` will inherit the donor's `Animator` reference, hover state machine, and all visual transition data.

4. **Restore `Navigation.Mode` to `Automatic` or `Explicit`** (currently `None`):
   - `None` prevents the button from participating in EventSystem selection, which may be required for hover-state transitions.
   - Use `Automatic` to match native buttons, or `Explicit` with null neighbors to isolate while still allowing pointer-based selection.

5. **Re-wire the click handler**:
   - Since we are keeping the native `MainMenuButton` instead of a `Button`, we cannot use `Button.onClick`.
   - Option 1: Add an `EventTrigger` component with a `PointerClick` entry that calls `OnModsButtonClicked`.
   - Option 2: Use reflection to find the `MainMenuButton`'s click/action field and subscribe to it.
   - Option 3: Add a thin proxy `MonoBehaviour` with `IPointerClickHandler` that forwards to `OnModsButtonClicked`.

### Tradeoffs

| Pro | Con |
|-----|-----|
| Pixel-perfect native hover/select behavior | Fragile: depends on `MainMenuButton` internal implementation surviving `Instantiate` (may reference specific Animator controllers, sibling GOs, or singletons that break on clone) |
| No need to reverse-engineer animation parameters | `StripNonUiBehaviours` exists to prevent cloned game scripts from firing unintended side effects -- whitelisting them reintroduces that risk |
| Works automatically if DINO updates hover animation style | `MainMenuButton` may have internal state (menu index, callback registration) that causes errors when duplicated |
| Minimal new code | Navigation mode change may cause the MODS button to interfere with native keyboard/gamepad menu traversal |

### Risk Assessment

**Medium-high risk**. The `MainMenuButton` component is proprietary and may have dependencies beyond its own GameObject hierarchy (e.g., references to a menu controller singleton, animation clip bindings to specific child transforms, or initialization logic in `Start()`/`OnEnable()` that fails on a clone). If it throws during clone initialization, it could crash the entire menu canvas.

Mitigation: wrap the clone in a try/catch; if the preserved `MainMenuButton` throws or enters an error state, fall back to destroying it and using the current `Button`-based approach.

---

## Approach B: Open a Native-Style Submenu Instead of F10

### Strategy

Replace the `ModMenuPanel` (F10 UGUI overlay) with a submenu that integrates into DINO's native menu canvas using the same visual style, layout, and transition animations as DINO's own settings submenu.

### Implementation Plan

1. **Reverse-engineer DINO's settings submenu structure**:
   - When the "Options" button is clicked, DINO opens a settings panel within the same canvas.
   - Use `FindSettingsButton` + reflection to find the `onClick` target or the child panel that becomes active.
   - Log the hierarchy of the settings panel (GameObjects, components, layout) to understand the pattern.

2. **Create a "Mods" submenu panel as a sibling of the settings panel**:
   - Clone the settings panel's root GameObject (or build a new one matching its RectTransform layout, background image, and animation setup).
   - Populate it with DINOForge mod management content (pack list, enable/disable toggles, detail pane).
   - Parent it under the same canvas/container as the settings panel.

3. **Wire the MODS button to show/hide this native submenu**:
   - On click, hide the main menu button list and show the mods submenu (mirroring how Options shows the settings panel).
   - Add a "Back" button that reverses the transition.

4. **Keep F10 as secondary access point**:
   - F10 continues to toggle `ModMenuPanel` (the UGUI overlay) for in-gameplay access.
   - The native submenu is only used when accessed from the main menu / pause menu.

### Tradeoffs

| Pro | Con |
|-----|-----|
| Fully integrated native UX -- mods panel looks and feels like a first-party settings tab | Requires reverse-engineering DINO's settings panel structure, which may change across game updates |
| No floating overlay on top of native menu | Significantly more implementation effort (panel cloning, content population, back-button wiring, animation matching) |
| Clear separation: native menu context uses native panel, gameplay context uses F10 overlay | Two separate mod UIs to maintain (native submenu + F10 overlay), increasing maintenance burden |
| | Settings panel structure is unknown and may use custom layout components that resist cloning |

### Risk Assessment

**High risk, high reward**. This approach produces the best user experience but depends heavily on the internal structure of DINO's settings submenu being clonable and stable across versions. If DINO uses a custom panel management system (e.g., a `MenuController` that tracks active panels by index), injecting a new panel could break the menu state machine.

Mitigation: Start with a diagnostic pass that dumps the settings panel hierarchy. If the structure is too complex or tightly coupled to game code, fall back to Approach A.

---

## Recommended Approach

**Approach A (native hover effects)** is recommended as the primary fix, with a phased plan:

### Phase 1: Fix hover/select effects (Approach A, reduced scope)

Focus on the visual feedback issue only, which is the higher-impact UX complaint. The concrete steps:

1. Modify `StripNonUiBehaviours` to preserve `Selectable`-derived components whose type name contains "MenuButton" (case-insensitive). Guard with try/catch per-component.

2. In `CloneSelectableAsButton`, do NOT destroy the donor-typed Selectable. Instead, leave it alive and add an `EventTrigger` with a `PointerClick` entry for the click handler. Do not add a `Button` component (it would conflict with the existing Selectable).

3. Change `Navigation.Mode` from `None` to `Automatic` on the MODS button so it participates in pointer-based selection state transitions.

4. Retain `SyncButtonVisualStyle` / `CopySelectableVisualState` as a fallback: if the preserved `MainMenuButton` is destroyed or throws, the copied `ColorBlock` on a plain `Button` still provides basic tint feedback.

5. Add a diagnostic log dump of the preserved `MainMenuButton` component's type, fields, and `Animator` reference so future sessions have data on its internal structure.

### Phase 2: Investigate native submenu (Approach B, diagnostic only)

After Phase 1 ships, add a diagnostic pass that:

1. Hooks the native Options button's `onClick` (or the `MainMenuButton`'s click path) and logs what panel/GameObject becomes active.
2. Dumps the full hierarchy of the settings submenu panel.
3. Produces a feasibility report on whether Approach B is viable for a future iteration.

### Phase 3: Native submenu (conditional on Phase 2 findings)

If Phase 2 shows the settings panel structure is clonable and stable, implement Approach B as an enhancement. Otherwise, keep Phase 1 as the final solution.

---

## Files Affected

| File | Change |
|------|--------|
| `src/Runtime/UI/NativeUiHelper.cs` | Modify `StripNonUiBehaviours` allowlist; modify `CloneSelectableAsButton` to preserve donor Selectable; add `EventTrigger`-based click wiring |
| `src/Runtime/UI/NativeMenuInjector.cs` | Change `Navigation.Mode.None` to `Automatic`; update `InjectButtonFromSelectable` to skip `Button` addition when donor is preserved; add diagnostic dump of preserved component |
| `src/Runtime/UI/ModMenuPanel.cs` | No changes in Phase 1 |
| `src/Tests/` | Add characterization tests for new `StripNonUiBehaviours` allowlist behavior |

---

## Success Criteria

1. The MODS button displays the same hover/select visual effect as the native "Options" button.
2. Clicking the MODS button still opens the mod management panel (F10 overlay in Phase 1, native submenu in Phase 3 if feasible).
3. No regressions: button injection still succeeds within 5 seconds of menu load, click handler still fires, debounce still works.
4. The MODS button does not interfere with keyboard/gamepad navigation of native menu buttons.
5. If the preserved `MainMenuButton` component throws on clone, the fallback path (plain `Button` with copied `ColorBlock`) activates silently.
