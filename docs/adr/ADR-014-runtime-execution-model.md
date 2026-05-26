# ADR-014: DINO Runtime Execution Model Discovery

## Status: Accepted

## Date: 2026-03-24

## Deciders

kooshapari

---

## Context

DINOForge is a BepInEx plugin that runs inside Diplomacy is Not an Option (DINO), a Unity 2021.3.45f2 game using Unity DOTS (ECS) with the Mono scripting backend. Standard BepInEx plugin authoring assumes that `MonoBehaviour.Update()` and Unity's PlayerLoop will drive per-frame callbacks. DINO violates this assumption entirely.

### What was discovered

DINO replaces Unity's PlayerLoop with a custom ECS scheduler driven by `GameStateSystems.GameStateUpdateHandler` and `[RuntimeInitializeOnLoadMethod]`. As a result, the following **never fire** inside DINO (confirmed by zero log output after `Awake()`):

| Mechanism | Why it fails |
|---|---|
| `MonoBehaviour.Update()` | DINO's scheduler replaces the Unity PlayerLoop; the default Update phase is not called |
| `MonoBehaviour.OnGUI()` | Suppressed — no IMGUI pass in DINO's ECS renderer path |
| `PlayerLoop.Update` subsystem callbacks (naive injection) | Injected at boot, but DINO later calls `PlayerLoop.SetPlayerLoop()` which overwrites the loop, evicting all injected entries |
| `Application.onBeforeRender` | Fires 3 times at boot then stops permanently |
| `Camera.onPreCull` | No cameras exist in DINO's ECS renderer path after boot |
| `World.Update()` Harmony patch (abstract) | The abstract base is never called directly — DINO uses concrete subclasses |
| `ComponentSystemGroup.Update()` Harmony patch (abstract) | Same; abstract base not dispatched directly |
| `Time.get_deltaTime` Harmony patch | C++ icall; cannot be intercepted by managed Harmony |
| `KeyInputSystem.OnUpdate()` via ECS | System is created and added to `InitializationSystemGroup`, but DINO's custom scheduler does not tick this group in its standard loop |
| `SceneManager.sceneLoaded` callbacks | Fires during scene teardown when the Mono delegate machinery is in a corrupt state; even a no-op callback can trigger crashes in other listeners |

The following **do fire** (confirmed working):

| Mechanism | When |
|---|---|
| `MonoBehaviour.Awake()` / `OnEnable()` | Synchronously from `AddComponent()` — one-time at boot |
| `MonoBehaviour.OnDestroy()` | At frame 0, as DINO sweeps its scene hierarchy |
| `SystemBase.OnCreate()` | Synchronously during world creation |
| Win32 background thread (`GetAsyncKeyState`, `Thread.Sleep`) | Indefinitely; DINO does not interfere with native threads |
| `FightGroup.OnUpdate()` Harmony postfix | Every gameplay frame; DINO's scheduler calls this concrete managed type |
| `GameplayInitializationSystemsGroup.OnUpdate()` Harmony postfix | Every gameplay frame (earlier in frame than FightGroup) |
| `PathFindingGroup.OnUpdate()` Harmony postfix | Every gameplay frame |
| `PlayerLoop.SetPlayerLoop()` Harmony postfix | Every time DINO rebuilds its player loop (used to re-inject our entries) |

### Why the BepInEx-managed plugin object dies

DINO's scene loader destroys all non-essential `MonoBehaviour` objects at frame 0, including the BepInEx-managed `Plugin` gameObject, even when `DontDestroyOnLoad` is set. A separate `DINOForge_Root` `GameObject` created with `HideFlags.HideAndDontSave | DontDestroyOnLoad` survives this sweep.

### Why `Resources.FindObjectsOfTypeAll` is banned from background threads

Calling `Resources.FindObjectsOfTypeAll&lt;Canvas&gt;()` from a background thread deadlocks during Unity's asset-loading phase (`InitialGameLoader` scene). This is a Unity internal lock conflict and cannot be worked around without blocking the game indefinitely.

---

## Decision

### 1. Abandon `MonoBehaviour.Update()` for per-frame work

`MonoBehaviour.Update()` is unreliable in DINO. Any feature requiring per-frame execution must use one of the verified mechanisms below.

### 2. Win32 background thread for F9/F10 key detection

A dedicated background thread polls `GetAsyncKeyState(VK_F9)` and `GetAsyncKeyState(VK_F10)` every 50 ms (20 Hz). On a falling-edge transition, the thread directly invokes `KeyInputSystem.OnF9Pressed` and `KeyInputSystem.OnF10Pressed` static delegates.

**Why this is safe:** Unity 2021.3 on Mono does not enforce main-thread checks on `SetActive()` calls for `HideAndDontSave / DontDestroyOnLoad` objects. Invoking a static C# delegate is inherently thread-safe. The delegates themselves call Unity UI code on whatever thread they run on; the RuntimeDriver handlers are designed to be idempotent and do not depend on main-thread-only Unity APIs.

**Why not Unity `Input` APIs from this thread:** `Input.GetKeyDown()` is a Unity main-thread API and will produce undefined behavior or throw when called from a background thread.

### 3. PlayerLoop injection as primary fallback

At boot (`Awake()`), and again after every `PlayerLoop.SetPlayerLoop()` call (intercepted via a Harmony postfix), a `DINOForgeUpdate` entry is inserted into Unity's `Update` subsystem. This entry calls `DINOForgePlayerLoopUpdate()` which:

- Checks `Input.GetKeyDown(F9/F10)` and dispatches to `KeyInputSystem`
- Checks `NeedsResurrection` and calls `TryResurrect`
- Runs `OnBeforeRenderWorldScan` to detect new ECS worlds

**Why Harmony re-injection is needed:** DINO rebuilds the PlayerLoop during world teardown and creation. A naive one-time injection at boot is evicted when DINO calls `SetPlayerLoop`. The Harmony postfix on `SetPlayerLoop` ensures our entry is re-appended after every such rebuild. A reentrancy guard (`_reinjecting` / `_injectingNow`) prevents the postfix from recursing when our own re-injection calls `SetPlayerLoop`.

### 4. FightGroup Harmony patch for gameplay-phase detection

A Harmony postfix on `Systems.ComponentSystemGroups.FightGroup.OnUpdate()` (located in `DNO.Main.dll`) provides a guaranteed per-frame main-thread execution context during active gameplay. This is the most reliable mechanism for resurrection and key detection while the user is playing.

**Why FightGroup specifically:**
- `FightGroup` is a concrete managed `ComponentSystemGroup` subclass — Harmony can intercept it.
- It runs inside `SimulationSystemGroup` every gameplay frame.
- It does not run on the main menu, which is acceptable since UI overlays are only useful during gameplay.
- `GameplayInitializationSystemsGroup` is patched first in the priority list as it fires earlier in the frame; `FightGroup` is the primary fallback if the initialization group is not found.

**Why the abstract base class cannot be patched:** Harmony patches apply to individual method slots on types. `ComponentSystemGroup.OnUpdate()` is abstract; patching the abstract declaration does not intercept overrides in concrete subclasses. Each concrete DINO group must be patched individually.

### 5. `NeedsResurrection` flag + `TryResurrect` for MonoBehaviour lifecycle recovery

When `RuntimeDriver.OnDestroy()` fires (frame 0, main thread), it sets `Plugin.NeedsResurrection = true`. The resurrection is then performed by whichever execution context fires next (PlayerLoop, FightGroup postfix, KeyInputSystem `OnUpdate`, or the background watcher triggering a main-thread-guarded call). `TryResurrect` is guarded:

- Main-thread check: if called from a background thread, sets `NeedsResurrection = true` and returns.
- Max-attempt cap (3): prevents infinite resurrection loops if the underlying cause is fatal.
- Idempotency: checks `PersistentRoot != null` before creating a new object.

---

## Consequences

### Positive

- F9/F10 key input works at any point in the game lifecycle, including the main menu and during scene transitions.
- Resurrection of `DINOForge_Root` is automatic and requires no user action.
- The system degrades gracefully: each mechanism is independent; failure of one does not disable the others.
- Observability is high: every execution path writes timestamped heartbeat entries to `dinoforge_debug.log`.

### Negative

- The Win32 background thread invokes C# delegates cross-thread; handlers must be written to tolerate cross-thread invocation or marshal to the main thread explicitly.
- The Harmony postfix on `PlayerLoop.SetPlayerLoop` adds overhead to every player loop rebuild (typically infrequent, ~seconds apart).
- The FightGroup patch only fires during gameplay, not on the main menu. Resurrection during main-menu-only sessions depends on the PlayerLoop injection path.
- All resurrection paths using Unity APIs are guarded by a main-thread ID check captured at `Awake()`. If `_mainThreadId` is not set (edge case: `Awake` never called), the guard is bypassed.

### Neutral

- `SceneManager.sceneLoaded` and `activeSceneChanged` callbacks are explicitly not used, diverging from standard BepInEx plugin practice.
- `Application.onBeforeRender` is registered as a belt-and-suspenders callback but is expected to be unreliable after boot.

---

## Implementation Notes

### Execution context priority (highest confidence to lowest)

1. `FightGroup.OnUpdate()` Harmony postfix — gameplay only, highest reliability during play
2. `PlayerLoop.Update` injected entry (`DINOForgeUpdate`) — re-injected after every loop rebuild
3. `KeyInputSystem.OnUpdate()` ECS path — only fires if DINO's scheduler happens to tick `InitializationSystemGroup`
4. `Camera.onPreCull` — registered as extra belt-and-suspenders; may not fire
5. Win32 background thread — runs always; cannot call Unity APIs directly
6. `Application.onBeforeRender` — fires at boot only; unreliable beyond frame 3

### Key constants

```csharp
private const int VK_F9  = 0x78;  // Win32 virtual key code for F9
private const int VK_F10 = 0x79;  // Win32 virtual key code for F10
private const int MaxResurrectionAttempts = 3;
private const int BackgroundThreadPollIntervalMs = 50; // 20 Hz
private const int HeartbeatInterval = 600; // frames (~10s at 60 FPS)
```

### Thread safety contract

| Field | Writer thread | Reader threads |
|---|---|---|
| `Plugin.NeedsResurrection` | Background watcher, OnDestroy | PlayerLoop, FightGroup postfix, KeyInputSystem |
| `Plugin.PersistentRoot` | Main thread (TryResurrect) | Background watcher (via `ReferenceEquals` only) |
| `Plugin.PendingF9Toggle` | Background watcher | (consumed in RuntimeDriver.Update if active) |
| `KeyInputSystem.OnF9Pressed` | Plugin.Awake (main thread) | Background watcher, PlayerLoop |

`NeedsResurrection` and `PendingF9Toggle` / `PendingF10Toggle` are declared `volatile`. `PersistentRoot` writes are main-thread-only; background reads use `System.Object.ReferenceEquals` (never Unity's `==` operator, which calls `UnityEngine.Object.op_Equality` and is not thread-safe).

### Debug log location

```
<BepInEx root>/dinoforge_debug.log
```

Log writes are protected by `Monitor.TryEnter` with a 100 ms timeout to prevent deadlocks between the main thread and the background watcher thread.

---

## References

- `src/Runtime/Plugin.cs` — Bootstrap, resurrection, PlayerLoop injection, FightGroup patch, Win32 thread
- `src/Runtime/Bridge/KeyInputSystem.cs` — ECS system, PlayerLoop injection (secondary), F9/F10 dispatch
- `docs/adr/ADR-005-ecs-integration-strategy.md` — ECS bridge strategy context
- `docs/adr/ADR-009-runtime-orchestration.md` — ModPlatform lifecycle coordination
- `C:\Users\koosh\.claude\projects\C--Users-koosh-Dino\memory\project_dino_runtime_execution_model.md` — Confirmed runtime facts (2026-03-21)
- Unity Entities 0.51 / DOTS documentation: `SystemBase`, `InitializationSystemGroup`, `PlayerLoop`
- BepInEx 5.4.23.5 documentation: plugin lifecycle, `BaseUnityPlugin`
- HarmonyX documentation: `Harmony.Patch`, `HarmonyMethod`, postfix delegates
