# SPEC-002: Native Menu Button Injection

**Status**: Accepted
**Date**: 2026-03-24
**Author**: DINOForge Agents
**Related ADR**: [ADR-002: Native Menu Injector Design](../adr/ADR-002-native-menu-injector.md)

---

## Overview

`NativeMenuInjector` locates DINO's in-game UGUI main menu at runtime and injects a "Mods" button as a sibling of the native "Settings" button. When clicked, the button toggles the `ModMenuOverlay` (F10 menu) without requiring the player to know the keyboard shortcut.

The injector is a `MonoBehaviour` attached to the `DINOForge_Root` persistent `GameObject`. It is self-healing: it detects when the injected button has been destroyed (e.g., on scene unload) and re-injects automatically on the next scan cycle.

---

## Requirements

### Functional

| ID | Requirement |
|----|-------------|
| F-01 | A "Mods" button shall appear in DINO's main menu within 5 seconds of the menu becoming visible. |
| F-02 | Clicking the "Mods" button shall toggle the `ModMenuOverlay` to visible/hidden. |
| F-03 | The "Mods" button shall be visually identical in style (font, colours, hover/pressed states) to the native "Settings" button. |
| F-04 | The injector shall be idempotent: injecting into a canvas that already contains a `DINOForge_ModsButton` shall reuse the existing button, not create a second one. |
| F-05 | On scene change, the injector shall reset its injection state and attempt re-injection into the new scene's canvas. |
| F-06 | If no suitable canvas or button is found, the injector shall log a diagnostic message and retry without throwing an exception to the caller. |
| F-07 | A `static Action? OnScanNeeded` delegate shall be exposed to allow external code (e.g., tests, `RuntimeDriver`) to trigger a re-scan from the main thread. |

### Non-Functional

| ID | Requirement |
|----|-------------|
| N-01 | All `Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()` calls shall execute only on the Unity main thread. |
| N-02 | The injector shall never throw an unhandled exception. All injection paths are wrapped in `try/catch`. |
| N-03 | The periodic re-scan interval shall be no less than 1 second to limit per-frame GC allocation. |
| N-04 | The injected button shall not acquire EventSystem keyboard/gamepad focus automatically. |
| N-05 | Click debounce shall prevent the menu from toggling more than once per 200 ms. |

---

## Architecture

```
Plugin.Awake()
  └─ PersistentRoot.AddComponent<NativeMenuInjector>()
       └─ RuntimeDriver.Initialize()
            ├─ injector.SetLogger(log)
            ├─ injector.SetModMenuHost(modMenuOverlay)
            └─ NativeMenuInjector.OnScanNeeded = () => injector.TryInjectMenuButton()

NativeMenuInjector (MonoBehaviour on DINOForge_Root)
  ├─ Awake()  ──► SceneManager.activeSceneChanged += OnActiveSceneChanged
  ├─ Start()  ──► TryInjectMenuButton()
  ├─ Update() ──► every 2s: TryInjectMenuButton()   [if not injected]
  │               on button destroy: reset _injected flag
  └─ OnDestroy() ──► SceneManager.activeSceneChanged -= OnActiveSceneChanged

TryInjectMenuButton()
  ├─ Resources.FindObjectsOfTypeAll<Canvas>()
  ├─ foreach active canvas:
  │    └─ FindSettingsButton(canvas)
  │         ├─ NativeUiHelper.FindButtonByText(canvas.transform, "Settings")
  │         └─ NativeUiHelper.FindButtonByText(canvas.transform, "Options")
  └─ InjectButton(settingsButton)
       ├─ Duplicate-guard scan (DINOForge_ModsButton prefix)
       ├─ NativeUiHelper.CloneButton(settingsButton, "Mods")
       ├─ SyncButtonVisualStyle(modsButton, settingsButton)
       ├─ NativeUiHelper.PositionAfterSibling(modsRect, settingsRect)
       ├─ modsButton.SetActive(true); modsButton.interactable = true
       ├─ Raycast target validation + GraphicRaycaster check
       ├─ RewireModsButtonClick(modsButton)   // clears inherited listeners
       ├─ Navigation.Mode = None              // isolate from native nav graph
       └─ _injectedButton = modsButton; _injected = true
```

---

## Implementation Details

### Finding the DINO Main Menu

DINO does not expose a stable API surface for its menus. The injector uses a canvas-agnostic strategy:

1. Call `Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()` to enumerate every loaded `Canvas` object, including inactive ones.
2. Filter to canvases where `canvas.gameObject.activeInHierarchy == true`.
3. For each active canvas, search for a button whose `UnityEngine.UI.Text` child contains the text "Settings" (primary) or "Options" (fallback).
4. The first canvas that yields a matching button is used as the injection target.

Canvas names are logged for diagnostics but are not used as a filter criterion. The assumption that a "Settings" or "Options" labelled button exists in the main menu has been validated against DINO v1.4.x and is expected to remain stable.

### Finding the Correct Button Container

The injected "Mods" button is placed as a sibling of the "Settings" button, within the same parent `Transform`. This preserves the vertical stack layout used by DINO's menu (typically a `VerticalLayoutGroup`). The sibling index is set to immediately follow the Settings button via `NativeUiHelper.PositionAfterSibling(modsRect, settingsRect)`.

### Injecting the "Mods" Button

Injection proceeds in eight logged steps:

| Step | Action |
|------|--------|
| 1 | `NativeUiHelper.CloneButton(settingsButton, "Mods")` — `Object.Instantiate` + rename |
| 2 | `NativeUiHelper.PositionAfterSibling` — sets sibling index after Settings |
| 3 | `modsButton.gameObject.SetActive(true)` + `interactable = true` |
| 4 | `CanvasGroup` check — enables `interactable` and `blocksRaycasts` if present |
| 5 | Raycast diagnostics — verifies `targetGraphic.raycastTarget`, parent `CanvasGroup` chain, and `GraphicRaycaster` on the canvas |
| 6 | `RewireModsButtonClick` — replaces all `onClick` listeners with `OnModsButtonClicked` |
| 7 | `Navigation.Mode = None` — isolates button from EventSystem navigation graph |
| 8 | Final state verification — logs all critical fields before setting `_injected = true` |

If any step throws, the exception is caught and logged as a warning. `_injected` is not set to `true` and the injector will retry on the next scan cycle.

### `OnScanNeeded` Delegate

```csharp
public static System.Action? OnScanNeeded;
```

This static delegate is set by `RuntimeDriver` after the injector is created:

```csharp
NativeMenuInjector.OnScanNeeded = () => injector.TryInjectMenuButton();
```

It allows any main-thread caller — including tests, `RuntimeDriver.Update()`, or future tooling — to request an immediate scan without coupling directly to the `NativeMenuInjector` instance. The delegate is `null`-safe: callers must check before invoking.

### Retry Logic

Retry is automatic and uses two complementary mechanisms:

1. **Scene change trigger**: `OnActiveSceneChanged` resets `_injected = false`, clears `_injectedButton`, and calls `TryInjectMenuButton()` immediately.
2. **`Update()` timer**: every `RescanInterval` (2 seconds), if `_injected` is false or `_injectedButton` has been destroyed, `TryInjectMenuButton()` is called again.

There is no maximum retry count. The injector keeps trying indefinitely because:
- DINO's boot takes 60–90 seconds and the menu may not appear until the player clicks through the splash screen.
- Scene transitions can reload the menu at any time during a play session.

### `InitialGameLoader` Click-Through Requirement

DINO presents a loading/splash screen (`InitialGameLoader`) immediately after launch. The main menu `Canvas` is not instantiated until the player manually clicks through this screen. This is a game-side design constraint with no programmatic bypass available without patching DINO internals.

Consequence: the injector will log "0 Settings buttons found" for every scan attempt during the 60–90 second boot window. This is expected behaviour, not an error. The player must advance past the splash screen manually before the Mods button appears.

### Visual Style Synchronisation

`SyncButtonVisualStyle(Button target, Button source)` copies the following properties from the Settings button to the Mods button:

- `Button.transition` (ColorTint / SpriteSwap / Animation)
- `Button.colors` (color block: normal, highlighted, pressed, disabled, fade duration)
- `Button.spriteState` (highlighted, pressed, selected, disabled sprites)
- `Button.animationTriggers` (animator trigger names)
- `Button.targetGraphic` (resolved by relative path from button root to graphic transform)
- `UnityEngine.UI.Text`: `font`, `fontStyle`, `fontSize`, `color`, `alignment`, `material`

This ensures the Mods button matches the game's visual skin without requiring any bundled assets.

---

## Known Constraints

| Constraint | Detail |
|------------|--------|
| Main-thread only | `Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()` deadlocks if called from a background thread during asset loading. All scans must run on the Unity main thread. |
| Splash screen gate | Player must click through `InitialGameLoader` before the main menu is visible. Scans before this point return no results and retry silently. |
| UGUI dependency | Approach is specific to Unity UGUI. A future DINO migration to UI Toolkit would require a full rewrite of the injector. |
| No keyboard/gamepad auto-focus | The Mods button does not take EventSystem focus automatically to avoid interfering with DINO's native menu navigation. |
| Canvas name variability | Canvas names are not stable across DINO versions; the injector relies on button text instead, which is more stable but still DINO-version-dependent. |
| Persistent root required | The injector lives on `DINOForge_Root`. If that object is destroyed (e.g., resurrection fails), the injector is lost until the next resurrection cycle. |
| `UnityEngine.UI.Text` only | The button text scanner uses the legacy `UnityEngine.UI.Text` component. If DINO switches to `TextMeshPro`, `FindButtonByText` will need to be updated. |

---

## Test Plan

### Unit Tests

| Test | Description |
|------|-------------|
| `TryInjectMenuButton_NoCanvases_DoesNotThrow` | Call with no canvases present; verify no exception and `_injected == false`. |
| `TryInjectMenuButton_ActiveCanvasNoSettingsButton_DoesNotThrow` | Active canvas with no Settings/Options button; verify retry state. |
| `TryInjectMenuButton_FindsSettingsButton_SetsInjected` | Provide a mock canvas with a Settings button; verify `_injected == true` after call. |
| `InjectButton_DuplicateGuard_DoesNotDoubleInject` | Call inject twice with same parent; verify only one `DINOForge_ModsButton` child exists. |
| `RewireModsButtonClick_ClearsInheritedListeners` | Verify that the cloned button's original listeners are replaced, not appended to. |
| `SyncButtonVisualStyle_CopiesColorBlock` | Verify color block matches source after sync. |
| `OnModsButtonClicked_Debounce_SecondClickIgnored` | Fire two clicks within 200 ms; verify `Toggle()` called exactly once. |
| `OnModsButtonClicked_NullMenuHost_DoesNotThrow` | Click with `_menuHost == null`; verify warning logged, no exception. |

### Integration Tests

| Test | Description |
|------|-------------|
| `SceneChange_ResetsInjectionState` | Simulate `OnActiveSceneChanged`; verify `_injected` reset and re-scan triggered. |
| `Update_ButtonDestroyed_ResetsAndRescans` | Destroy `_injectedButton` externally; advance timer; verify `_injected` reset and re-scan. |
| `OnScanNeeded_TriggersInjection` | Set `OnScanNeeded`, invoke it, verify `TryInjectMenuButton` is called. |
| `FullBoot_InjectionSucceeds` | End-to-end test with a mock DINO main menu canvas; verify Mods button present and clickable. |

### Manual Acceptance Criteria

1. Launch DINO with DINOForge installed.
2. Wait for the splash screen; click through `InitialGameLoader`.
3. Main menu appears; "Mods" button is visible adjacent to "Settings".
4. "Mods" button has identical visual style to "Settings".
5. Clicking "Mods" opens the DINOForge mod menu overlay.
6. Clicking "Mods" again closes the overlay.
7. Start a game, pause; verify "Mods" button appears in the pause menu.
8. Return to main menu; verify "Mods" button persists after scene transition.

---

## Open Issues

| ID | Issue | Priority |
|----|-------|----------|
| OI-01 | `InitialGameLoader` has no programmatic skip path. Players on slow machines may wait 90+ seconds before the Mods button appears. A future option: hook `InitialGameLoader.OnDestroy` to trigger an immediate scan. | Low |
| OI-02 | If DINO uses `TextMeshPro` for button labels in a future update, `NativeUiHelper.FindButtonByText` will need a `TMP_Text` fallback. | Medium |
| OI-03 | `Navigation.Mode.None` on the Mods button means it is unreachable via gamepad/keyboard navigation. A future enhancement could wire explicit `selectOnUp`/`selectOnDown` links to adjacent buttons. | Low |
| OI-04 | The 2-second rescan timer fires even during active gameplay (when the menu is not visible), causing minor unnecessary scans. A scene-name or canvas-visibility pre-check could eliminate these. | Low |

---

## Status

**Implementation**: Complete (`src/Runtime/UI/NativeMenuInjector.cs`)
**Tests**: Partial (manual acceptance passing; automated unit/integration tests pending)
**Documentation**: This specification + ADR-002
