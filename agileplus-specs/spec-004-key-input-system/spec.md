# SPEC-004: Key Input System

**Status**: Implemented — Active
**Date**: 2026-03-24
**Author**: DINOForge Agents
**Related ADR**: [ADR-014: DINO Runtime Execution Model Discovery](../adr/ADR-014-runtime-execution-model.md)

---

## Overview

The Key Input System provides reliable F9 and F10 hotkey detection inside Diplomacy is Not an Option (DINO), a Unity 2021.3 DOTS game that suppresses standard `MonoBehaviour.Update()` callbacks. F9 toggles the debug overlay; F10 toggles the mod menu. The system must survive DINO's MonoBehaviour sweep at frame 0 and function across all game phases: boot, main menu, gameplay, and scene transitions.

Because DINO replaces Unity's PlayerLoop with a custom ECS scheduler, no single execution context is guaranteed to fire consistently across all phases. The Key Input System therefore uses a **layered redundancy model**: multiple independent detection paths run in parallel, and the first path to detect a keypress dispatches the action. All paths converge on the same two static delegates: `KeyInputSystem.OnF9Pressed` and `KeyInputSystem.OnF10Pressed`.

---

## Requirements

### Functional Requirements

| ID | Requirement |
|---|---|
| KIS-F1 | Pressing F9 at any time during gameplay or the main menu must toggle the debug overlay within 100 ms of the physical keypress. |
| KIS-F2 | Pressing F10 at any time during gameplay or the main menu must toggle the mod menu within 100 ms of the physical keypress. |
| KIS-F3 | F9/F10 detection must survive DINO's frame-0 MonoBehaviour destruction of the BepInEx plugin object. |
| KIS-F4 | F9/F10 detection must survive scene transitions without requiring user action. |
| KIS-F5 | The system must detect at most one keypress event per physical key-down transition (no repeat events while key held). |
| KIS-F6 | If `DINOForge_Root` (`PersistentRoot`) is destroyed, the system must automatically recreate it (resurrection) before the next user-visible frame. |
| KIS-F7 | Key events must be dispatched to `KeyInputSystem.OnF9Pressed` and `KeyInputSystem.OnF10Pressed` respectively; no direct coupling to UI layer. |
| KIS-F8 | Detection must function during the DINO main menu (before any gameplay ECS worlds are created). |

### Non-Functional Requirements

| ID | Requirement |
|---|---|
| KIS-NF1 | Latency from physical keypress to action dispatch must not exceed 100 ms under normal conditions. The Win32 polling path introduces up to 50 ms latency; PlayerLoop and FightGroup paths introduce at most 1 frame (~16 ms at 60 FPS). |
| KIS-NF2 | The background thread must not call any Unity API that requires the main thread, and must not block the game's render loop. |
| KIS-NF3 | Log writes must not block the calling thread for more than 100 ms (enforced via `Monitor.TryEnter`). |
| KIS-NF4 | Resurrection must be capped at 3 consecutive attempts to prevent infinite loops on fatal errors. |
| KIS-NF5 | The system must write heartbeat entries to `dinoforge_debug.log` at most every 600 frames (~10 s at 60 FPS) to allow liveness verification without excessive disk I/O. |
| KIS-NF6 | The system must function correctly on Windows only (Win32 `GetAsyncKeyState` is platform-specific). |

---

## Architecture

### Component Map

```
Plugin (BepInEx BaseUnityPlugin)
  ├── Awake()
  │     ├── Creates DINOForge_Root (HideAndDontSave + DontDestroyOnLoad)
  │     ├── Attaches RuntimeDriver MonoBehaviour
  │     ├── Registers Application.onBeforeRender → OnBeforeRenderWorldScan
  │     ├── Registers Camera.onPreCull → OnCameraPreCull
  │     ├── Calls InjectPlayerLoopUpdate() → inserts DINOForgeUpdate into PlayerLoop.Update
  │     ├── Calls PatchPlayerLoopRejection() → Harmony postfix on PlayerLoop.SetPlayerLoop
  │     ├── Calls PatchDinoSystemGroups() → Harmony postfix on FightGroup.OnUpdate
  │     └── Calls StartResurrectionWatcher() → background thread
  │
  ├── StartResurrectionWatcher() [background thread — DINOForge-KeyInputWatcher]
  │     ├── Polls GetAsyncKeyState(VK_F9/VK_F10) every 50 ms
  │     ├── On falling-edge: sets PendingF9/F10Toggle, invokes KeyInputSystem.OnF9/F10Pressed
  │     └── Checks NeedsResurrection flag → calls TryResurrect (guarded: main-thread only)
  │
  ├── DINOForgePlayerLoopUpdate() [PlayerLoop.Update subsystem — main thread]
  │     ├── Input.GetKeyDown(F9/F10) → KeyInputSystem.OnF9/F10Pressed
  │     ├── NeedsResurrection check → TryResurrect
  │     └── Calls OnBeforeRenderWorldScan
  │
  ├── OnDinoSystemGroupUpdate() [Harmony postfix on FightGroup.OnUpdate — main thread]
  │     ├── Input.GetKeyDown(F9/F10) → KeyInputSystem.OnF9/F10Pressed
  │     └── NeedsResurrection check → TryResurrect
  │
  └── OnCameraPreCull() [Camera.onPreCull — main thread, if cameras exist]
        ├── Input.GetKeyDown(F9/F10) → KeyInputSystem.OnF9/F10Pressed
        └── NeedsResurrection check → TryResurrect

KeyInputSystem (ECS SystemBase — InitializationSystemGroup)
  ├── OnCreate()
  │     ├── Calls PatchPlayerLoopSetPlayerLoop() → Harmony postfix (dedup guard)
  │     └── Calls InjectIntoPlayerLoop() → inserts DINOForgeKeyLoop into PlayerLoop.Update
  └── OnUpdate() [ECS tick — only if DINO scheduler ticks InitializationSystemGroup]
        ├── Resurrection check → inline new GameObject creation
        ├── Input.GetKeyDown(F9) → OnF9Pressed
        └── Input.GetKeyDown(F10) → OnF10Pressed

RuntimeDriver (MonoBehaviour on DINOForge_Root)
  ├── Initialized via RuntimeDriver.Initialize(log, config, dump, dumpPath)
  ├── KeyInputSystem.OnF9Pressed = () => _debugOverlay.Toggle()
  ├── KeyInputSystem.OnF10Pressed = () => _modMenu.Toggle()
  └── OnDestroy() → Plugin.NeedsResurrection = true
```

### Detection Path Priority

The following paths are listed from highest per-frame confidence to lowest. All paths dispatch to the same delegates.

| Priority | Path | Execution Context | Fires When | Max Latency |
|---|---|---|---|---|
| 1 | FightGroup Harmony postfix | Main thread, per-frame | During active gameplay | 1 frame |
| 2 | PlayerLoop.Update injection (`DINOForgeUpdate`) | Main thread, per-frame | When DINO's PlayerLoop is running | 1 frame |
| 3 | KeyInputSystem.OnUpdate() ECS | Main thread, per-frame | Only if DINO ticks InitializationSystemGroup | 1 frame |
| 4 | Camera.onPreCull callback | Main thread, per-frame | When a camera exists and renders | 1 frame |
| 5 | Win32 background thread | Background thread, 20 Hz | Always — independent of DINO scheduler | 50 ms |
| 6 | Application.onBeforeRender | Main thread | Boot only (3 calls, then stops) | N/A |

### Delegate Wiring

`KeyInputSystem` exposes two static nullable delegates:

```csharp
public static System.Action? OnF9Pressed;
public static System.Action? OnF10Pressed;
```

`RuntimeDriver.Initialize()` assigns these during boot:

```csharp
KeyInputSystem.OnF9Pressed  = () => _debugOverlay?.Toggle();
KeyInputSystem.OnF10Pressed = () => _modMenu?.Toggle();
```

All detection paths call `OnF9Pressed?.Invoke()` or `OnF10Pressed?.Invoke()`. If no handler is assigned (e.g., pre-resurrection), the invocation is a no-op.

---

## Implementation

### Path 1: Win32 Background Thread (Plugin.StartResurrectionWatcher)

**File**: `src/Runtime/Plugin.cs` — `StartResurrectionWatcher()`

A `Thread` named `DINOForge-KeyInputWatcher` is launched from `Awake()` as a background thread (`IsBackground = true`). It polls every 50 ms:

```csharp
bool f9Down  = (GetAsyncKeyState(VK_F9)  & 0x8000) != 0;
bool f10Down = (GetAsyncKeyState(VK_F10) & 0x8000) != 0;
```

Edge detection uses `f9WasDown` / `f10WasDown` local booleans to fire exactly once per key-down transition. On a rising edge, the thread sets `PendingF9Toggle = true` and directly invokes `KeyInputSystem.OnF9Pressed?.Invoke()`.

The thread never calls any Unity API. It reads `Plugin.PersistentRoot` using `System.Object.ReferenceEquals` (not Unity's `==` operator). If `NeedsResurrection` is set, it calls `Plugin.TryResurrect()`, which internally checks the thread ID and returns early if not on the main thread, setting `NeedsResurrection = true` for a later main-thread path to handle.

**Win32 import**:

```csharp
[DllImport("user32.dll")]
private static extern short GetAsyncKeyState(int vKey);

private const int VK_F9  = 0x78;
private const int VK_F10 = 0x79;
```

### Path 2: PlayerLoop Injection (Plugin.InjectPlayerLoopUpdate)

**File**: `src/Runtime/Plugin.cs` — `InjectPlayerLoopUpdate()`, `DINOForgePlayerLoopUpdate()`

At `Awake()`, `InjectPlayerLoopUpdate()` appends a `PlayerLoopSystem` entry typed `DINOForgeUpdate` to the `UnityEngine.PlayerLoop.Update` subsystem. The `updateDelegate` is `DINOForgePlayerLoopUpdate`, which calls `Input.GetKeyDown(F9/F10)` and dispatches to `KeyInputSystem`.

DINO rebuilds the PlayerLoop during world setup. To survive this, a second Harmony postfix (`dinoforge.plugin.playerloop`) is placed on `PlayerLoop.SetPlayerLoop`. After every `SetPlayerLoop` call, `OnPlayerLoopSet` is invoked, which calls `InjectPlayerLoopUpdate()` again. A `_reinjecting` boolean prevents recursion:

```
PlayerLoop.SetPlayerLoop() called by DINO
  → Harmony postfix fires
  → OnPlayerLoopSet() checks _reinjecting, sets true
  → InjectPlayerLoopUpdate()
    → reads current loop, appends DINOForgeUpdate, calls SetPlayerLoop()
    → Harmony postfix fires again — _reinjecting is true, returns immediately
  → _reinjecting = false
```

### Path 3: KeyInputSystem ECS System (KeyInputSystem.OnUpdate)

**File**: `src/Runtime/Bridge/KeyInputSystem.cs`

`KeyInputSystem` is a `SystemBase` decorated with `[AlwaysUpdateSystem]` and `[UpdateInGroup(typeof(InitializationSystemGroup))]`. It is created and enrolled in `InitializationSystemGroup` by `Plugin.ScanAndRegisterKeyInputSystem()` for every ECS world detected via `World.All`.

`OnUpdate()` calls `Input.GetKeyDown(F9/F10)` and dispatches to the static delegates. It also handles resurrection inline — if `Plugin.NeedsResurrection` is set and `Plugin.PersistentRoot` is null, it creates a new `DINOForge_Root` GameObject and attaches a `RuntimeDriver` directly on the main thread.

`KeyInputSystem.OnCreate()` independently applies the same PlayerLoop injection as `Plugin.InjectPlayerLoopUpdate()`, targeting a separate marker type `DINOForgeKeyLoop`. This provides a second re-injection point independent of Plugin's Harmony patch.

**Resurrection in OnUpdate (inline)**:

```csharp
if (Plugin.NeedsResurrection && Plugin.PersistentRoot == null)
{
    Plugin.NeedsResurrection = false;
    GameObject root = new GameObject("DINOForge_Root");
    root.hideFlags = HideFlags.HideAndDontSave;
    Object.DontDestroyOnLoad(root);
    Plugin.PersistentRoot = root;
    RuntimeDriver driver = root.AddComponent<RuntimeDriver>();
    driver.Initialize(Plugin.ResurrectionLog, Plugin.ResurrectionConfig,
                      Plugin.ResurrectionDump, Plugin.ResurrectionDumpPath);
}
```

### Path 4: FightGroup Harmony Postfix (Plugin.PatchDinoSystemGroups)

**File**: `src/Runtime/Plugin.cs` — `PatchDinoSystemGroups()`, `OnDinoSystemGroupUpdate()`

`PatchDinoSystemGroups()` searches `AppDomain.CurrentDomain.GetAssemblies()` for `DNO.Main`, then iterates a priority list of concrete system group type names:

```csharp
string[] groupTypeNames = new[]
{
    "Systems.ComponentSystemGroups.FightGroup",
    "Systems.ComponentSystemGroups.GameplayInitializationSystemsGroup",
    "Systems.ComponentSystemGroups.PathFindingGroup",
    "Systems.ComponentSystemGroups.ResourceDeliveryGroup",
};
```

The first type found in `DNO.Main` that exposes an `OnUpdate()` method (or inherits one from `ComponentSystemGroup`) is patched with a Harmony postfix pointing to `OnDinoSystemGroupUpdate`. Only one patch is applied (`break` after first success). A `_dinoSystemGroupHarmonyPatched` guard prevents double-patching.

`OnDinoSystemGroupUpdate()` is protected by a `_inDinoSystemGroupUpdate` reentrancy flag (the postfix itself may trigger code that calls another DINO system group method).

### Resurrection Flow

```
RuntimeDriver.OnDestroy()
  → Plugin.NeedsResurrection = true

Next execution context that fires on main thread:
  PlayerLoop tick / FightGroup tick / KeyInputSystem.OnUpdate / Camera.onPreCull
  → checks NeedsResurrection && PersistentRoot == null
  → calls Plugin.TryResurrect(trigger, context)
    → checks main thread ID — if background thread: sets NeedsResurrection = true, returns
    → checks _resurrectionAttempts < MaxResurrectionAttempts (3)
    → creates new DINOForge_Root (HideAndDontSave + DontDestroyOnLoad)
    → AddComponent<RuntimeDriver>().Initialize(...)
    → re-registers Application.onBeforeRender and Camera.onPreCull
    → clears _seenWorldKeys (so KeyInputSystem is re-enrolled in new ECS worlds)
    → sets _resurrectionAttempts = 0, NeedsResurrection = false
```

If resurrection fails (exception), `_resurrectionAttempts` is not reset. After 3 consecutive failures the system logs a single "giving up" message and stops attempting to prevent log spam.

### Harmony Instance IDs

| Instance ID | Patch | Applied in |
|---|---|---|
| `dinoforge.keyinput.playerloop` | `PlayerLoop.SetPlayerLoop` postfix (re-inject `DINOForgeKeyLoop`) | `KeyInputSystem.OnCreate()` |
| `dinoforge.plugin.playerloop` | `PlayerLoop.SetPlayerLoop` postfix (re-inject `DINOForgeUpdate`) | `Plugin.PatchPlayerLoopRejection()` |
| `dinoforge.plugin.dinosysgroup` | `FightGroup.OnUpdate` postfix | `Plugin.PatchDinoSystemGroups()` |
| `dinoforge.plugin.GUID` | Base Harmony instance (no patches applied per ADR-005) | `Plugin.Awake()` |

### Observability

Every execution path writes to `&lt;BepInEx root&gt;/dinoforge_debug.log` via `Plugin.WriteDebug()` or `KeyInputSystem.WriteDebug()`. Log entries include a timestamp, class name, and message. Heartbeat entries are written at the first call and every 600 subsequent calls per path:

```
[2026-03-24 12:00:00.000] [KeyInputSystem] OnUpdate frame=1 PersistentRoot=alive
[2026-03-24 12:01:40.000] [KeyInputSystem] PlayerLoop tick #600
[2026-03-24 12:00:00.050] [Plugin] Win32: F9 pressed — invoking OnF9Pressed directly
```

Log writes use `Monitor.TryEnter(_debugLogLock, 100)` to prevent deadlocks between the main thread and the background watcher. If the lock cannot be acquired within 100 ms, the log entry is silently dropped.

---

## Test Plan

### Unit Tests

| ID | Test | Expected Result |
|---|---|---|
| KIS-T1 | Call `KeyInputSystem.OnF9Pressed?.Invoke()` with a mock handler assigned | Mock handler called exactly once |
| KIS-T2 | Call `KeyInputSystem.OnF10Pressed?.Invoke()` with a mock handler assigned | Mock handler called exactly once |
| KIS-T3 | Call `KeyInputSystem.OnF9Pressed?.Invoke()` with no handler assigned | No exception |
| KIS-T4 | Assign a handler to `OnF9Pressed`, then assign null, then invoke | No exception (null-conditional delegate invoke) |
| KIS-T5 | `Plugin.TryResurrect` called with background thread ID when `_mainThreadId` is set to a different value | Returns without creating new GameObject; `NeedsResurrection = true` |
| KIS-T6 | `Plugin.TryResurrect` called 4 times with `PersistentRoot = null` each time | Executes max 3 times; 4th call is a no-op with no exception |
| KIS-T7 | Win32 edge detection: set `f9WasDown = true`, then `f9Down = true` | No invocation (key already held) |
| KIS-T8 | Win32 edge detection: set `f9WasDown = false`, then `f9Down = true` | Invocation fires exactly once |

### Integration Tests

| ID | Test | Expected Result |
|---|---|---|
| KIS-IT1 | Simulate `RuntimeDriver.OnDestroy()` by setting `NeedsResurrection = true`, then call `PlayerLoopTick()` | `PersistentRoot` is non-null after tick; `NeedsResurrection = false` |
| KIS-IT2 | Simulate `PlayerLoop.SetPlayerLoop()` call after injection | `DINOForgeUpdate` entry is present in new PlayerLoop subsystem |
| KIS-IT3 | `KeyInputSystem.OnCreate()` called in a fresh ECS World | `PlayerLoop.Update` subsystem contains `DINOForgeKeyLoop` entry |
| KIS-IT4 | Simulate DINO calling `SetPlayerLoop` to evict injected entries | Re-injection Harmony postfix restores the entry |

### Manual / Smoke Tests

| ID | Test | Expected Result |
|---|---|---|
| KIS-M1 | Launch DINO with DINOForge installed; press F9 after 10 seconds | Debug overlay toggles; log contains `[KeyInputSystem] F9 pressed` or `[Plugin] Win32: F9 pressed` |
| KIS-M2 | Launch DINO; press F10 | Mod menu toggles |
| KIS-M3 | Launch DINO; wait 90+ seconds for main menu to appear; press F9 | Overlay toggles (verifies main-menu phase detection) |
| KIS-M4 | Start a game match; press F9 | Overlay toggles (verifies gameplay-phase FightGroup path) |
| KIS-M5 | Wait 60 seconds after launch; verify `dinoforge_debug.log` contains heartbeat entries from at least one path | Log contains `Heartbeat`, `PlayerLoop tick`, or `OnUpdate frame` entries |

---

## Open Issues

| ID | Issue | Priority |
|---|---|---|
| KIS-OI1 | Win32 background thread invokes C# delegates cross-thread. Unity `SetActive()` on `DontDestroyOnLoad` objects is thread-safe in Mono 2021.3, but this is undocumented Unity behavior and could break in future Unity versions. | Medium |
| KIS-OI2 | `KeyInputSystem.OnUpdate()` (ECS path) has not been confirmed to fire in the DINO main menu phase. The ECS world for the menu may not tick `InitializationSystemGroup`. Priority 3 may effectively only provide redundancy during gameplay. | Low |
| KIS-OI3 | `Camera.onPreCull` has not been confirmed to fire in any DINO phase after boot. It remains registered as belt-and-suspenders but may be effectively dead code. Should be verified with a targeted log experiment. | Low |
| KIS-OI4 | `Application.onBeforeRender` fires 3 times at boot and stops. The registration is retained for correctness but contributes no ongoing functionality. Consider removing after further confirmation to reduce overhead. | Low |
| KIS-OI5 | The reentrancy flag `_inDinoSystemGroupUpdate` on `OnDinoSystemGroupUpdate` uses a static non-volatile boolean. Under pathological conditions (unlikely for a postfix that runs on a single ECS scheduler thread), this could malfunction. | Low |
| KIS-OI6 | `MaxResurrectionAttempts = 3` is hardcoded. If DINO undergoes rapid scene cycling (e.g., loading screens), 3 attempts may be exhausted before the game stabilizes, leaving the overlay non-functional. Consider exposing this as a BepInEx config entry. | Low |

---

## Status

**Implementation**: Complete as of commit `4da07eb` (2026-03-24).
**Verified paths**: Win32 background thread, PlayerLoop injection, FightGroup Harmony postfix, KeyInputSystem.OnUpdate (ECS).
**Unverified paths**: Camera.onPreCull (no DINO cameras confirmed), Application.onBeforeRender (fires 3x at boot only).
**Test coverage**: Unit test stubs exist in `src/Tests/`; integration tests for PlayerLoop injection pending.
