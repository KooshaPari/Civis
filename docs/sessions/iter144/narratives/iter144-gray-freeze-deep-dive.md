# DINO Gray-Freeze Root Cause Deep-Dive (iter-144)

## What user observes

Game launches, window paints gray, never progresses past gray screen. Tier 1-4 probes pass (window, log mtime, debug log init, named pipe). Tier 5 (30s health loop) reports unresponsive after t+13s, BepInEx log mtime frozen.

## Log trace at hang point

### `dinoforge_debug.log` (last ~10 relevant lines, run 2026-05-20T09:14):
```
T09:14:27.9610Z  VanillaCatalog.Build starting scan
T09:14:27.9715Z  [GameBridgeServer] Started on pipe: dinoforge-game-bridge
T09:14:27.9756Z  [GameBridgeServer] Waiting for connection...
T09:14:28.1178Z  [KeyInputSystem] KeyInputSystem.OnDestroy
T09:14:28.1193Z  [AssetSwapSystem] AssetSwapSystem.OnDestroy - bundles unloaded
T09:14:28.2690Z  [RuntimeDriver] OnDestroy: background poll stopped, main-thread pump idle until resurrection. BridgeServerThreadAlive=True. NeedsResurrection set; awaiting scene transition.
T09:14:28.2974Z  PackUnitSpawner.Initialize: Registry initialized
T09:14:28.2974Z  AerialSpawnSystem.Initialize: Registry set
<<< silence — log truncated at "Registered stats from starwar..." >>>
```

### `LogOutput.log` (last 5):
```
[Info] [ContentLoader] Registered weapons from blasters.yaml
[Info] [ContentLoader] Registered doctrines from cis_doctrines.yaml
[Info] [ContentLoader] Registered doctrines from republic_doctrines.yaml
[Info] [ContentLoader] Registered stats from starwar...  (truncated mid-line)
```

CRITICAL NEW EVIDENCE (LogOutput.log earlier): `[Warning] [ModPlatform] VanillaCatalog build failed: System.ArgumentNullException: Value cannot be null. Parameter name: destination` thrown from `EntityManager.GetAllEntities` at `VanillaCatalog.cs:96`.

## Hypotheses ranked by evidence

### H1: VanillaCatalog scans world DURING DINO's world-teardown race (HIGH)
**Evidence**: 
- `VanillaCatalog.Build starting scan` at T+27.961 (`VanillaCatalog.cs:94`), then 157ms later `KeyInputSystem.OnDestroy` fires (T+28.117). The em.GetAllEntities at `VanillaCatalog.cs:96` throws `ArgumentNullException` because the underlying `EntityDataAccess` chunk store is being torn down — `MemSet(destination=null, ...)` confirms the destination buffer is null, meaning the EntityManager's allocator state is invalid.
- The scan blocks ModPlatform.OnWorldReady (ModPlatform.cs:247) wrapped in try/catch (line 244-256), so the exception is caught and logged — but the ModPlatform thread continues into ContentLoader registrations, which then race against `Partial shutdown (keeping bridge)`.
- Pattern matches Pattern #233/#530 family: world is in TFM-of-runtime equivalent unstable state.

**Counter-evidence**: catch block at ModPlatform.cs:253 swallows the exception, so the hang itself can't be a thrown exception — must be downstream blocking.

**Confidence**: HIGH for the symptom; the AmbiguousMatch-style scan-on-dying-world is the kickoff.

### H2: ContentLoader pack registration races RuntimeDriver.OnDestroy (HIGH)
**Evidence**: 
- T+28.297 PackUnitSpawner.Initialize fires AFTER RuntimeDriver.OnDestroy at T+28.269 (`Plugin.cs:1184`). 
- ContentLoader keeps logging "Registered ..." (LogOutput.log tail) WHILE Partial shutdown runs. The log literally cuts mid-line at "Registered stats from starwar..." — disk write was in progress when something blocked the main thread.
- ContentLoader runs on the same thread as the scene transition that destroyed RuntimeDriver. If a registration calls back into a now-disposed registry/world reference, it blocks indefinitely.

**Counter-evidence**: "Partial shutdown complete. Bridge server still running" did print, suggesting ShutdownNonBridge completed. The hang is *after* that.

**Confidence**: HIGH

### H3: Plugin.SharedBridgeServer pipe wait blocks main-thread re-entry (MED)
**Evidence**: `[GameBridgeServer] Waiting for connection...` fired at T+27.975 and never logs accept/timeout. The named-pipe `BeginWaitForConnection` is async, but if SharedBridgeServer instance is referenced from a main-thread continuation that lost its synchronization context post-OnDestroy, the resume point dies silently.

**Counter-evidence**: RuntimeDriver.OnDestroy reports `BridgeServerThreadAlive=True` (Plugin.cs:1204), so the server's background thread itself is alive.

**Confidence**: MED — bridge thread is fine; problem is main-thread starvation.

### H4: Resurrection watcher never fires for the destroyed root (MED)
**Evidence**: `Plugin.cs:184` registers `SceneManager.sceneLoaded += OnSceneLoaded`. Per `project_dino_runtime_execution_model.md`, only `activeSceneChanged` is confirmed firing — `sceneLoaded` is unverified for DINO's custom PlayerLoop replacement. If `sceneLoaded` never fires after the InitialGameLoader→MainMenu transition that destroyed our root, `TryResurrect` never runs.

**Counter-evidence**: NativeMenuInjector Attempt#3 scanned 28 canvases including 'MainMenu' (LogOutput.log earlier), so scenes ARE transitioning and observable to other components.

**Confidence**: MED

### H5: MockSteamworksNet / Hospital unresolved-component side effect (LOW)
**Evidence**: `Unresolved component type: Components.Hospital` logged (ComponentMap.cs:196). MockSteamworksNet not seen in this log. No clear blocking path.

**Confidence**: LOW

## Recommended next step

ONE concrete experiment to falsify H1+H2: **gate VanillaCatalog.Build behind a "world is alive AND not in teardown" check, AND wrap the entire OnWorldReady body so a single failed step does not race against the scene transition.**

Falsification query: After deploying a build with `VanillaCatalog.Build` short-circuited (skip scan, log "VanillaCatalog: skipped — world unstable"), launch the game. If `dinoforge_debug.log` now shows `[Plugin] OnSceneLoaded` entries firing for MainMenu, H1+H2 are confirmed and the resurrection path is intact. If the log still silences at the same point, H4 wins.

Concrete grep to inspect first: 
```
grep -n "OnSceneLoaded\|sceneLoaded\|activeSceneChanged" src/Runtime/Plugin.cs
```
Lines 184, 188-209 — verify whether `SceneManager.activeSceneChanged` is ALSO subscribed (currently only `sceneLoaded` per line 184). The runtime-execution-model doc says `activeSceneChanged` is the proven-working hook.

## Test for fix

After any fix, `dinoforge_debug.log` past the RuntimeDriver.OnDestroy line MUST contain:
1. `[Plugin] OnSceneLoaded: scene='MainMenu' mode=Single` (or `activeSceneChanged` equivalent) — proves scene event fired.
2. `[Plugin] OnSceneLoaded: resurrection needed - NeedsRes=True rootNull=True` — proves the resurrection branch was taken.
3. `[Plugin] TryResurrect (...) PersistentRoot null ... resurrecting...` — proves new RuntimeDriver attached.
4. `[Plugin] Resurrection complete via ...` — proves Initialize() succeeded.
5. A second `VanillaCatalog: scanning N entities` where N > 0 — proves the world is now stable.

Absence of (1) means the scene event never fires (H4). Presence of (1) but absence of (3) means resurrection logic itself is broken.
