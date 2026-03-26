# ADR-002: Native Menu Injector Design

## Status: Accepted

## Date: 2026-03-24

---

## Context

DINOForge needs a discoverable entry point into its mod menu that does not require players to memorise the F10 hotkey. DINO's UI is rendered via Unity UGUI and its main menu persists as a set of standard `Canvas` / `Button` hierarchy objects. Injecting a sibling "Mods" button next to the native "Settings" button is the lowest-friction, most idiomatic approach: it reuses the game's own visual skin and appears exactly where players already look for options.

### DINO Boot Sequence Constraints

DINO takes 60–90 seconds to fully boot. During that window:

1. Unity loads asset bundles from `StreamingAssets/aa/StandaloneWindows64/` (4.2 GB + 2.1 GB bundles).
2. An `InitialGameLoader` splash/loading screen is shown; the user must click through it manually before the main menu becomes visible.
3. The main-menu `Canvas` and its child buttons are not instantiated until after the splash screen is dismissed.

Any scan for UI elements that runs before the player advances past the splash screen will find zero suitable canvases and must retry without error.

### Why `SceneManager.activeSceneChanged` Was Chosen as the Scan Trigger

`SceneManager.activeSceneChanged` fires on the Unity main thread, immediately after a scene transition completes. This gives two guarantees that alternative mechanisms cannot provide:

- **Thread safety**: All UGUI object traversal (`Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()`, `GetComponentsInChildren&lt;Button&gt;()`) is safe only on the main thread. Calling these APIs from any other thread during asset loading causes a deadlock inside Unity's native object manager.
- **Timeliness**: The callback fires as soon as the incoming scene is fully loaded, so newly instantiated UI objects are already present in the hierarchy.

Additional scans are performed on a `RescanInterval = 2f` second timer inside `Update()` to handle cases where the menu canvas becomes active after the scene-changed event (e.g., delayed activation by DINO's own initialisation scripts).

### Why Periodic Background Scanning Was Removed

An earlier design polled for canvases from the `DINOForge-KeyInputWatcher` background thread at 50 ms intervals. This caused a deadlock during asset loading:

- Unity's asset-loading subsystem holds an internal lock while deserialising bundles.
- `Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()` attempts to acquire the same lock from the background thread.
- With the main thread blocked in asset I/O, neither thread can proceed.

The deadlock manifested as a hard freeze (no crash, no log output) lasting until the OS killed the process. The fix was to move all `Resources.FindObjectsOfTypeAll` calls exclusively onto the main thread and to rely on `SceneManager.activeSceneChanged` (plus `Update()` polling) instead of background-thread scheduling. The watcher thread comment in `Plugin.cs` documents this explicitly:

```
// NOTE: NativeMenuInjector scanning is NOT done from this watcher thread —
// Resources.FindObjectsOfTypeAll<Canvas>() can deadlock when called from
// a background thread during Unity's asset-loading phase.
// Scans are triggered by OnActiveSceneChanged (main-thread-safe).
```

### Why the Button Is Injected into DINO's Native Menu

Alternatives considered:

| Alternative | Rejected Because |
|---|---|
| Separate overlay canvas (always-on) | Covers game elements; breaks DINO's pause flow |
| F10 hotkey only | Non-discoverable; players unfamiliar with mods will never find it |
| BepInEx Configuration Manager | Requires a separate plugin dependency; not integrated with DINOForge's pack system |
| ImGui / Dear ImGui | No maintained Unity Mono port compatible with Unity 2021.3 + BepInEx 5 |

Cloning the native "Settings" or "Options" button and re-labelling it "Mods" is safe and self-contained: it inherits all UGUI visuals (colours, sprite states, font, layout) from the existing button, requires no additional assets, and lives inside the canvas that DINO already shows and hides correctly during pause/resume transitions.

---

## Decision

`NativeMenuInjector` is a `MonoBehaviour` attached to the `DINOForge_Root` persistent GameObject. It:

1. Subscribes to `SceneManager.activeSceneChanged` in `Awake()`.
2. On each scene change and on a 2-second `Update()` timer, calls `TryInjectMenuButton()` on the main thread.
3. `TryInjectMenuButton()` enumerates all active `Canvas` objects via `Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()`, searches each for a button labelled "Settings" or "Options", and — if found — clones that button, relabels it "Mods", wires its `onClick` to `IModMenuHost.Toggle()`, and isolates its UGUI navigation graph (`Navigation.Mode.None`) so it does not interfere with DINO's controller/keyboard navigation.
4. Once injection succeeds, `Update()` does nothing until the injected button is destroyed (e.g., on scene unload), at which point the cycle restarts automatically.
5. A `static Action? OnScanNeeded` delegate is provided as an external trigger for test harnesses or future tooling that need to force a re-scan from a known main-thread context.

`SceneManager.activeSceneChanged` is the primary trigger. The 2-second `Update()` poll is the secondary trigger for late-activating canvases.

---

## Consequences

### Positive

- Players discover the mod menu without documentation.
- The injected button is visually indistinguishable from native buttons.
- All UGUI traversal is main-thread-only, eliminating the deadlock class.
- Injection is idempotent and self-healing: re-injects on scene reload, detects pre-existing button to avoid duplicates.
- Full diagnostic logging (attempt counter, session ID, per-step state) aids debugging without impacting production performance.

### Negative / Limitations

- **InitialGameLoader dependency**: The user must manually click through DINO's splash screen before the main menu appears. There is no programmatic way to skip or detect this step without hooking DINO internals. The injector will silently retry until the menu is ready.
- **Canvas name independence**: The injector cannot rely on fixed canvas names because DINO may change them across game versions. It instead searches all active canvases for a "Settings" or "Options" button text, which is more fragile than a stable identifier but more resilient to canvas renames.
- **UGUI-only**: The approach is specific to Unity's UGUI system. If DINO ever migrates to UI Toolkit, the injector will need to be replaced.
- **No EventSystem focus transfer**: The injected button deliberately does not steal EventSystem focus (`currentSelectedGameObject`) to avoid triggering DINO's native navigation handlers unexpectedly. This means keyboard/gamepad navigation does not auto-select the Mods button.
- **Scan cost**: `Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()` traverses all loaded Unity objects. In a scene with many objects this has a small GC allocation cost. The 2-second scan interval keeps this negligible.
- **Persistent root required**: `NativeMenuInjector` relies on the `DINOForge_Root` persistent GameObject surviving scene transitions. If that object is destroyed, the injector is lost and no resurrection path currently exists for the injector specifically (resurrection targets `RuntimeDriver`).

---

## Implementation Notes

- **File**: `src/Runtime/UI/NativeMenuInjector.cs`
- **Namespace**: `DINOForge.Runtime.UI`
- **Wired by**: `RuntimeDriver.Initialize()` via `PersistentRoot.AddComponent&lt;NativeMenuInjector&gt;()`
- **Canvas candidates list** (`CanvasCandidateNames`): kept for reference diagnostics; actual filtering uses the Settings/Options button text, not canvas names.
- **Button cloning**: delegated to `NativeUiHelper.CloneButton()` and `NativeUiHelper.PositionAfterSibling()`.
- **Visual sync**: `SyncButtonVisualStyle()` mirrors transition type, color block, sprite state, animation triggers, and `UnityEngine.UI.Text` properties from source to clone.
- **Click debounce**: 200 ms (`ClickDebounceSeconds`) using `Time.unscaledTime` to prevent double-fire during rapid UGUI event delivery.
- **Duplicate guard**: before cloning, scans the parent's children for an existing `DINOForge_ModsButton` prefixed object and reuses it rather than injecting a second button.
- **Debug log**: supplemental file log at `BepInEx/dinoforge_debug.log` via `WriteDebug()` (non-blocking, `Monitor.TryEnter` with 100 ms timeout).

---

## References

- `src/Runtime/UI/NativeMenuInjector.cs` — full implementation
- `src/Runtime/Plugin.cs` — `StartResurrectionWatcher()` comment documenting the deadlock rationale
- `docs/specs/SPEC-002-native-menu-injector.md` — companion specification
- Memory note: `project_dino_runtime_execution_model.md` — definitive map of safe execution contexts in DINO
- ADR-009: Runtime Orchestration — persistent root and `RuntimeDriver` lifecycle
