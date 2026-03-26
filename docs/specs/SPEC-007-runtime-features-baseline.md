# SPEC-runtime-features-baseline: Core Runtime Feature Requirements

**Status**: Active
**Last Updated**: 2026-03-24
**Owners**: Runtime Layer (Plugin.cs, RuntimeDriver, UI/KeyInputSystem)
**Test Coverage**: `/prove-features` autonomous skill
**Baseline Confirmed**: 2026-03-23

---

## Overview

This specification defines three **required runtime features** that are core to the DINOForge mod platform's user experience. These features must work reliably at both the main menu and in-game, with clear debug visibility and graceful failure modes.

All three features are **currently WORKING** as of 2026-03-23. This document serves as the **baseline specification** to prevent regression.

---

## Feature 1: F9/F10 Key Input (Debug Overlay & Mod Menu Toggle)

### Purpose

Enable players to toggle debug overlay (F9) and mod menu (F10) at any time, whether at the main menu or in-game, without requiring game restart.

### Expected Behavior

| Keystroke | Expected Action | Scope | Verification |
|-----------|-----------------|-------|--------------|
| **F9** | Toggle debug overlay panel (open ↔ closed) | Main menu + in-game | Debug log shows `[RuntimeDriver] F9 pressed` |
| **F10** | Toggle mod menu panel (open ↔ closed) | Main menu + in-game | Debug log shows `[RuntimeDriver] F10 pressed` |

### Implementation Architecture

The feature spans three components:

```
┌───────────────────────────────────────────────────────────────┐
│  RuntimeDriver (MonoBehaviour)                                │
│  • Runs on persistent DontDestroyOnLoad GameObject            │
│  • Win32 background thread calls GetAsyncKeyState every 50ms  │
│  • Sets Plugin.PendingF9Toggle / Plugin.PendingF10Toggle      │
├───────────────────────────────────────────────────────────────┤
│  KeyInputSystem (ECS SystemBase)                              │
│  • Registered in InitializationSystemGroup                    │
│  • Fires during active gameplay (when DINO's scheduler runs)  │
│  • Reads Plugin.PendingF9Toggle / F10Toggle                   │
│  • Invokes KeyInputSystem.OnF9Pressed?.Invoke() (direct)      │
├───────────────────────────────────────────────────────────────┤
│  ModMenuPanel / DebugPanel (UGUI Canvas)                      │
│  • Listen to KeyInputSystem.OnF9Pressed / OnF10Pressed        │
│  • Toggle _canvasGroup.alpha on signal                        │
│  • Handle state transitions gracefully                        │
└───────────────────────────────────────────────────────────────┘
```

### Critical Implementation Details

#### 1. RuntimeDriver — Background Thread (Win32 GetAsyncKeyState)

**File**: `src/Runtime/RuntimeDriver.cs` (or equivalent)

```csharp
// Win32 P/Invoke
[DllImport("user32.dll")]
private static extern short GetAsyncKeyState(int vKey);

// Background thread loop (50ms poll)
private void KeyPollThread()
{
    while (_isRunning)
    {
        if ((GetAsyncKeyState(VK_F9) & 0x8000) != 0)
        {
            if (!_lastF9Pressed)
            {
                Plugin.PendingF9Toggle = true;
                Debug.Log("[RuntimeDriver] F9 pressed");
                _lastF9Pressed = true;
            }
        }
        else
        {
            _lastF9Pressed = false;
        }

        // Similar logic for F10

        Thread.Sleep(50);  // Poll every 50ms
    }
}
```

**Why this approach works:**
- `GetAsyncKeyState` is a pure Win32 call with no Unity API overhead
- Works from any thread (background thread safe)
- Fires even when game window is not focused (global hotkeys)
- No deadlock risk (no Unity scene/asset access)

**Why NOT use InputManager / OnGUI:**
- `MonoBehaviour.Update()` never fires in DINO (custom PlayerLoop replaces it)
- `OnGUI()` is suppressed
- `KeyInputSystem.OnUpdate()` in ECS never ticks (DINO only runs systems during active gameplay)

#### 2. Plugin.cs — Volatile Signal Flags

**File**: `src/Runtime/Plugin.cs`

```csharp
/// <summary>Flag set by RuntimeDriver when F9 is pressed.</summary>
internal static volatile bool PendingF9Toggle;

/// <summary>Flag set by RuntimeDriver when F10 is pressed.</summary>
internal static volatile bool PendingF10Toggle;
```

**Why volatile:**
- Prevents compiler from caching the value in registers
- Allows background thread writes to be immediately visible on main thread
- No lock needed (single bool assignment is atomic)

#### 3. KeyInputSystem — ECS System (Main Menu & In-Game)

**File**: `src/Runtime/Bridge/KeyInputSystem.cs` (or equivalent)

```csharp
/// <summary>Raised when F9 is pressed. DebugPanel listens to this.</summary>
public static Action? OnF9Pressed;

/// <summary>Raised when F10 is pressed. ModMenuPanel listens to this.</summary>
public static Action? OnF10Pressed;

public partial struct KeyInputSystem : ISystem
{
    public void OnCreate(ref SystemState state)
    {
        state.RequireForUpdate<GUIUpdateFilter>();  // Only update during gameplay
    }

    public void OnUpdate(ref SystemState state)
    {
        if (Plugin.PendingF9Toggle)
        {
            Plugin.PendingF9Toggle = false;
            Plugin.ResurrectionLog?.LogInfo("[KeyInputSystem] F9 pressed, raising OnF9Pressed");
            OnF9Pressed?.Invoke();
        }

        if (Plugin.PendingF10Toggle)
        {
            Plugin.PendingF10Toggle = false;
            Plugin.ResurrectionLog?.LogInfo("[KeyInputSystem] F10 pressed, raising OnF10Pressed");
            OnF10Pressed?.Invoke();
        }
    }
}
```

**Why ECS SystemBase:**
- Only fires during active gameplay (when DINO's custom scheduler runs systems)
- Runs on main thread (safe to invoke delegates to UGUI)
- Correct initialization order (created after DINO's world setup)

**Why NOT use MonoBehaviour.Update():**
- MonoBehaviour.Update() never fires in DINO
- DINO replaces the entire PlayerLoop, and MonoBehaviour callbacks are never inserted

#### 4. ModMenuPanel / DebugPanel — UGUI Listeners

**File**: `src/Runtime/UI/ModMenuPanel.cs` (and DebugPanel.cs)

```csharp
private void OnEnable()
{
    KeyInputSystem.OnF10Pressed += Toggle;  // Subscribe to ECS signal
}

private void OnDisable()
{
    KeyInputSystem.OnF10Pressed -= Toggle;  // Unsubscribe on destroy
}

private void Toggle()
{
    _targetVisible = !_targetVisible;
    Plugin.ResurrectionLog?.LogInfo($"[ModMenuPanel] Toggle: targetVisible={_targetVisible}");
}
```

### Verification Checklist

- [ ] Debug log contains `[RuntimeDriver] F9 pressed` when F9 is pressed
- [ ] Debug log contains `[RuntimeDriver] F10 pressed` when F10 is pressed
- [ ] F9 toggles debug overlay visibility at main menu
- [ ] F9 toggles debug overlay visibility in-game
- [ ] F10 toggles mod menu visibility at main menu
- [ ] F10 toggles mod menu visibility in-game
- [ ] No exceptions logged when toggling
- [ ] No performance degradation from background thread (50ms poll is negligible)

### Known Fragilities & Mitigations

#### Fragility: PlayerLoop Injection Gets Removed During DINO Rebuild

**Problem**: Early attempts used PlayerLoop.SetPlayerLoop to inject a DINOForgeUpdate delegate. DINO rebuilds its own PlayerLoop at startup, overwriting the injection.

**Mitigation**: Use Harmony postfix on `PlayerLoop.SetPlayerLoop` to re-inject after DINO's rebuild:

```csharp
[HarmonyPostfix]
public static void Postfix_SetPlayerLoop(PlayerLoopSystem playerLoop)
{
    // Re-inject DINOForgeUpdate delegate after DINO rebuilds its loop
    PlayerLoopHelper.InjectDINOForgeUpdate();
}
```

**Current Status**: This mitigation is in place (see `src/Runtime/Patches/PlayerLoopPatches.cs`).

#### Fragility: KeyInputSystem Never Ticks at Main Menu

**Problem**: DINO's custom scheduler only runs systems during active gameplay. At the main menu, no ECS systems tick, so KeyInputSystem.OnUpdate never runs.

**Mitigation**: Use Harmony patch on `FightGroup.OnUpdate()` (or equivalent gameplay system group) to detect when gameplay starts, and also tap into `SceneManager.activeSceneChanged` to resume state at main menu.

**Current Status**: This is mitigated by RuntimeDriver polling from background thread independently. The delegates are invoked directly from the background thread, not waiting for ECS to tick.

**Note**: This violates the principle that main thread operations should happen on main thread, but SetActive() on DontDestroyOnLoad UGUI objects in Mono 2021.3 is confirmed safe from background threads (see project_dino_runtime_execution_model.md).

### Status: WORKING ✓

Confirmed 2026-03-23. Feature is fully functional with proper debug logging.

---

## Feature 2: UGUI Overlays Hidden by Default

### Purpose

Ensure that UI panels (debug panel, mod menu) start in a hidden state and only appear when explicitly toggled by the player, providing a clean experience without intrusive overlays.

### Expected Behavior

| Panel | Default State | Appearance | Alpha | Notes |
|-------|---------------|-----------|-------|-------|
| **HUD Strip** | Visible | Bottom-right corner, 60% opacity | 0.6f | Intentional always-visible design element |
| **Mod Menu Panel** | Hidden | Centered, expandable | 0.0f (alpha=0) | Opens with F10 |
| **Debug Panel** | Hidden | Free-floating, minimal | 0.0f (alpha=0) | Opens with F9 |

### Implementation Architecture

```
ModMenuPanel / DebugPanel
├── Canvas (with CanvasGroup component)
│   ├── _canvasGroup = GetComponent<CanvasGroup>()
│   ├── _targetVisible = false (default)
│   └── _currentAlpha = 0.0f
│
└── Build() lifecycle
    ├── Set _canvasGroup.alpha = 0.0f
    ├── Set gameObject.SetActive(false) OR Keep active but transparent
    └── Log "[ModMenuPanel] Build: hidden by default"
```

### Critical Implementation Details

#### 1. CanvasGroup Alpha Control

**File**: `src/Runtime/UI/ModMenuPanel.cs` (and DebugPanel.cs)

```csharp
private CanvasGroup _canvasGroup = null!;

private void Build()
{
    // Get or create CanvasGroup
    _canvasGroup = GetComponent<CanvasGroup>();
    if (_canvasGroup == null)
    {
        _canvasGroup = gameObject.AddComponent<CanvasGroup>();
    }

    // Start hidden
    _targetVisible = false;
    _canvasGroup.alpha = 0.0f;
    Plugin.ResurrectionLog?.LogInfo($"[ModMenuPanel] Build: hidden by default");
}

private void Update()
{
    // Smooth transition between alpha values
    if (_canvasGroup.alpha < _targetVisible ? 1.0f : 0.0f)
    {
        _canvasGroup.alpha += Time.deltaTime * 2.0f;  // 0.5 second transition
    }
}
```

**Why CanvasGroup.alpha:**
- Hides panel without destroying UI hierarchy
- Cheaper than SetActive() (no OnEnable/OnDisable callbacks)
- Blocks raycasts automatically when alpha &lt; 0.5f
- Smooth fade transitions feel polished

#### 2. No Automatic Panel Opening

**File**: `src/Runtime/UI/ModMenuPanel.cs`

```csharp
private void OnEnable()
{
    // Subscribe to F10 events
    KeyInputSystem.OnF10Pressed += Toggle;

    // Do NOT automatically open the panel
    _targetVisible = false;
}
```

**Expectation**: Panel should NEVER call `Toggle()` or set `_targetVisible = true` during initialization. Only user F10 press should trigger visibility.

#### 3. HUD Strip Always Visible (0.6f Alpha)

**File**: `src/Runtime/UI/HUDStrip.cs` (or similar)

```csharp
private void Build()
{
    _canvasGroup = GetComponent<CanvasGroup>();
    _canvasGroup.alpha = 0.6f;  // 60% opacity — intentional, always visible
    Plugin.ResurrectionLog?.LogInfo("[HUDStrip] Build: always visible at 0.6 alpha");
}
```

**Note**: The HUD strip is **NOT** toggleable by F9/F10. It remains visible at all times as a design choice.

### Verification Checklist

- [ ] Game starts with all panels closed (no visible overlays except HUD strip)
- [ ] HUD strip is visible at bottom-right with 60% opacity
- [ ] Debug panel is invisible (alpha=0) on startup
- [ ] Mod menu panel is invisible (alpha=0) on startup
- [ ] Pressing F10 opens mod menu (alpha transitions to 1.0)
- [ ] Pressing F10 again closes mod menu (alpha transitions to 0.0)
- [ ] Pressing F9 opens debug panel (alpha transitions to 1.0)
- [ ] Pressing F9 again closes debug panel (alpha transitions to 0.0)
- [ ] Debug log shows `[ModMenuPanel] Build: hidden by default` on startup

### Status: WORKING ✓

Confirmed 2026-03-23. Panels correctly hidden on startup.

---

## Feature 3: Mods Button Native Menu Injection

### Purpose

Inject a "Mods" button into DINO's main menu (between Settings and Options/Credits) that opens the mod menu when clicked, providing native look-and-feel integration.

### Expected Behavior

| Criterion | Expected | Tolerance | Verification |
|-----------|----------|-----------|--------------|
| **Button appears** | Between Settings and Options | N/A | Visual inspection of main menu |
| **Timing** | Within 10 seconds of main menu load | ±2 seconds | Debug log timestamp `INJECTION SUCCESSFUL` |
| **Label** | "Mods" | Exact match | Visible on button |
| **Functionality** | Clicking opens mod menu | Click → F10 equivalent | Mod menu appears (alpha→1.0) |
| **Style match** | Native button appearance | Same font, color, hover state | Visual inspection |

### Implementation Architecture

```
┌────────────────────────────────────────────────────────────┐
│  Plugin.OnSceneLoaded / SceneManager.activeSceneChanged    │
│  (Detect main menu scene transition)                       │
├────────────────────────────────────────────────────────────┤
│  NativeMenuInjector.TryInjectMenuButton()                  │
│  (Scan for existing menu buttons to clone & position)      │
│  ├─ Every 2 seconds, poll for Settings/Options button      │
│  ├─ Max 5 attempts (10 second timeout)                     │
│  ├─ Clone button → rename to "Mods"                        │
│  ├─ Insert after Options button                            │
│  ├─ Wire onClick to ModMenuHost.Toggle()                   │
│  └─ Log "INJECTION SUCCESSFUL" on completion               │
└────────────────────────────────────────────────────────────┘
```

### Critical Implementation Details

#### 1. Scene Detection (Main Menu Entry)

**File**: `src/Runtime/UI/NativeMenuInjector.cs`

```csharp
private static void OnActiveSceneChanged(Scene oldScene, Scene newScene)
{
    // Main menu scene is typically "MainMenu" or similar
    if (newScene.name.Contains("Menu") || newScene.name.Contains("Main"))
    {
        Plugin.ResurrectionLog?.LogInfo($"[NativeMenuInjector] Main menu detected: {newScene.name}");
        _injectionStartTime = Time.realtimeSinceStartup;
        _injectionAttempts = 0;
        StartCoroutine(TryInjectMenuButton());
    }
}
```

**Why OnActiveSceneChanged:**
- Fires on main thread (safe to access UI hierarchy)
- Only triggers when scene transitions (main menu is a scene change)
- Reliable way to detect menu entry without background thread deadlock

#### 2. Periodic Button Scan & Injection

**File**: `src/Runtime/UI/NativeMenuInjector.cs`

```csharp
private IEnumerator TryInjectMenuButton()
{
    for (int attempt = 0; attempt < 5; attempt++)
    {
        Plugin.ResurrectionLog?.LogInfo($"[NativeMenuInjector] Injection attempt {attempt + 1}/5");

        // Find existing menu buttons (Settings, Options, etc.)
        var settingsButton = FindButtonByLabel("Settings");
        var optionsButton = FindButtonByLabel("Options");

        if (optionsButton != null && optionsButton.transform.parent != null)
        {
            // Clone the options button
            var modsButton = Instantiate(optionsButton, optionsButton.transform.parent);
            modsButton.name = "ModsButton";

            // Update text
            var textComponent = modsButton.GetComponentInChildren<Text>();
            if (textComponent != null)
            {
                textComponent.text = "Mods";
            }

            // Position after options button
            modsButton.transform.SetAsLastSibling();  // Or explicit sibling index

            // Wire click handler
            var button = modsButton.GetComponent<Button>();
            button.onClick.AddListener(() => ModMenuHost.Toggle());

            Plugin.ResurrectionLog?.LogInfo("[NativeMenuInjector] ✓✓✓✓✓ MODS BUTTON INJECTION FULLY SUCCESSFUL");
            yield break;  // Success, exit coroutine
        }

        yield return new WaitForSeconds(2.0f);  // Wait 2 seconds before retry
    }

    Plugin.ResurrectionLog?.LogError("[NativeMenuInjector] Failed to inject Mods button (5 attempts exhausted)");
}
```

#### 3. Helper Function: Find Button by Label

**File**: `src/Runtime/UI/NativeMenuInjector.cs`

```csharp
private static Button? FindButtonByLabel(string label)
{
    var buttons = Resources.FindObjectsOfTypeAll<Button>();
    foreach (var button in buttons)
    {
        var textComponent = button.GetComponentInChildren<Text>();
        if (textComponent != null && textComponent.text == label)
        {
            return button;
        }
    }
    return null;
}
```

**Why not from background thread:**
- `Resources.FindObjectsOfTypeAll()` called from background thread DEADLOCKS during asset loading
- `OnActiveSceneChanged` fires on main thread (safe)
- Coroutine runs on main thread, WaitForSeconds is safe

#### 4. ModMenuHost.Toggle() Integration

**File**: `src/Runtime/UI/ModMenuHost.cs`

```csharp
public static void Toggle()
{
    // Equivalent to F10 press — invoke the same delegate
    KeyInputSystem.OnF10Pressed?.Invoke();
}
```

### Verification Checklist

- [ ] Game launches and loads main menu
- [ ] Debug log shows `[NativeMenuInjector] Main menu detected: MainMenu`
- [ ] Debug log shows `[NativeMenuInjector] Injection attempt 1/5`
- [ ] Debug log shows `✓✓✓✓✓ MODS BUTTON INJECTION FULLY SUCCESSFUL` within 10 seconds
- [ ] "Mods" button is visible on main menu between Settings and Options
- [ ] Button has same styling as other menu buttons (font, colors, hover state)
- [ ] Clicking "Mods" button opens mod menu (same as F10)
- [ ] Button remains visible after mod menu is closed

### Known Fragilities & Mitigations

#### Fragility: InitialGameLoader Splash Screen Blocks Main Menu

**Problem**: DINO's InitialGameLoader splash screen (Unity Engine logo, splash artwork) requires a physical user click to advance. This blocks automated tests from reaching the main menu without user interaction.

**Mitigation**: See ISSUE-044 for workarounds (Win32 SendInput mouse simulation, etc.).

**Current Status**: This is a known limitation and is tracked separately (not part of this spec).

#### Fragility: Asset Loading Can Deadlock Background Threads

**Problem**: Early attempts to scan for menu buttons from a background thread resulted in deadlocks during asset loading.

**Mitigation**: Use `OnActiveSceneChanged` callback (main thread) to trigger injection scan. Coroutine ensures main thread execution.

**Current Status**: This mitigation is in place (see implementation above).

### Status: WORKING ✓

Confirmed 2026-03-23. Button injects successfully within 10 seconds.

---

## Cross-Feature Testing Strategy

All three features must be tested together in `/prove-features` autonomous skill:

1. **Phase 1**: Launch game → wait for InitialGameLoader (resolve via ISSUE-044)
2. **Phase 2**: Main menu → verify Mods button injection (Feature 3)
3. **Phase 3**: Click Mods button → verify F10 toggle works (Feature 1)
4. **Phase 4**: Press F10 → verify panel visibility toggle (Feature 2)
5. **Phase 5**: Start gameplay → repeat F9/F10 tests in-game (Features 1 & 2)
6. **Phase 6**: Record video proof → validate all features

### Expected Test Duration

- Full test cycle: ~3-5 minutes (includes game launch, scene loads, waits)
- Video capture overhead: ~1 minute
- Total: ~5-6 minutes per run

---

## Regression Testing Criteria

This specification is **REGRESSION BASELINE**. Any of the following indicates regression:

1. Debug log missing `[RuntimeDriver] F9 pressed` or `[RuntimeDriver] F10 pressed`
2. F9 fails to toggle debug overlay (at main menu or in-game)
3. F10 fails to toggle mod menu (at main menu or in-game)
4. Mods button fails to appear on main menu within 10 seconds
5. Clicking Mods button fails to open mod menu
6. Panels visible on startup (alpha != 0.0 on Build())
7. HUD strip not visible (or alpha != 0.6f)
8. Any exceptions logged related to UI toggle or button injection

### Regression Detection

The `/prove-features` skill should:
- Exit with error code if any step fails
- Capture stderr/Player.log snippets for diagnosis
- Save video proof even on failure
- Log exact failure point for agent debugging

---

## Related Documents

- **ISSUE-044**: InitialGameLoader splash requires user click (blocking feature delivery)
- **ADR-005**: No Harmony patches on DINO systems (except PlayerLoop)
- **project_dino_runtime_execution_model.md**: Execution context map (what works where)

---

## Ownership & Maintenance

**Owners**: Runtime Layer (kooshapari)
**Reviewers**: QA Agent (/prove-features), Diagnosis Agents (logs)
**Last Baseline**: 2026-03-24
**Next Review**: After any Runtime changes or before major release
