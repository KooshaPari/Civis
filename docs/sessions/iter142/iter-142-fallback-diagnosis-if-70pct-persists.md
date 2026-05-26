# Iter-142 Fallback Diagnosis Plan
**If 70% hang persists after HandleConnect deploy**, follow this decision tree to isolate the culprit.

---

## Symptom
- Loading bar freezes at ~70%
- Text: "Loading is finished. Launching..."
- Chicken skeleton visible (scene loaded, ECS spawned)
- Game appears frozen but processes still running

---

## Top 3 Candidates (Ranked by Likelihood)

### 1. **EcsTypeDiscovery.DiscoverAndLog() BLOCKING** (HIGH)
**Why**: Runs synchronously in Plugin.Awake(), scans all assemblies for component types before HotReload or GameBridgeServer start.

**Evidence**: 
- `Plugin.cs:97` calls `EcsTypeDiscovery.DiscoverAndLog()` — blocks Awake() until complete
- Reflection over 45K+ types takes 2-5s on first call
- If an assembly fails to enumerate (security exception, corrupted metadata), reflection loop could timeout

**How to diagnose**:
1. Check `BepInEx\dinoforge_debug.log` for timestamp gaps:
   ```
   [Plugin] ECS type discovery complete — check dinoforge_debug.log for details
   ```
   If this line appears **>8 seconds** after startup, discovery is slow.

2. Look for reflection exceptions:
   ```
   ECS type discovery assembly check failed: ...
   ```

3. **If present**: Discovery is the culprit. Mitigation: wrap reflection in timeout or defer to background thread.

---

### 2. **AssetSwapSystem MinFrameDelay = 600 frames blocking scene transition** (MED)
**Why**: AssetSwapSystem waits 600 frames (~10s @ 60fps) before applying swaps. If scene transition code waits for swap completion, loading stalls.

**Evidence**:
- `AssetSwapSystem.cs:74` sets `const int MinFrameDelay = 600`
- System updates in `PresentationSystemGroup` — runs EVERY frame
- If any downstream code in "Loading is finished" stage polls `AssetSwapRegistry.GetPending().Count` and waits for it to be 0, game hangs

**How to diagnose**:
1. Check `BepInEx\dinoforge_debug.log` for:
   ```
   AssetSwapSystem: processing N pending swap(s)
   ```
   If this appears **after** "Loading is finished", swaps are being applied post-transition (not blocking).

2. Search for calls to `AssetSwapRegistry.GetPending()` in game code (unlikely, but check mod pack contents).

3. **If AssetSwap logs are MISSING**: Swaps are not being registered/applied — not the issue.

---

### 3. **GameBridgeServer.StartServer() hanging on NamedPipeServerStream initialization** (MED)
**Why**: ModPlatform initializes GameBridgeServer in `BeforeWorldReady` hook (`Plugin.cs:268-292`). If the named pipe server cannot bind (port conflict, permission issue), server thread may deadlock/spin.

**Evidence**:
- `ModPlatform.cs:278` creates `new GameBridgeServer(this)` 
- GameBridgeServer constructor starts a background thread
- If the thread enters an exception loop (e.g., NamedPipeServerStream.WaitForConnection hanging), main thread may be blocked by exception handling

**How to diagnose**:
1. Check for these log lines in `BepInEx\LogOutput.log` (NOT dinoforge_debug.log):
   ```
   [ModPlatform] GameBridgeServer started (new singleton)
   [ModPlatform] Failed to start GameBridgeServer: ...
   ```

2. Look for repeated errors about pipe server:
   ```
   Cannot open named pipe / Access denied / Already in use
   ```

3. **If GameBridgeServer log is missing entirely**: Server initialization never completed — this is the culprit.

---

## Quick Decision Tree

```
IF "ECS type discovery complete" appears >8s after startup
  → Discovery is slow/timing out. Next fix: async reflection or cached assembly snapshot.

ELSE IF "AssetSwapSystem: processing" appears AFTER "Loading is finished"
  → Swaps are applied post-transition. Not blocking. Check vanilla game.

ELSE IF "GameBridgeServer started" is missing from LogOutput.log
  → Server initialization hangs. Check named pipe permissions or retry logic.

ELSE IF all three appear with normal timings
  → Issue is NOT in Plugin.Awake() or early ECS. Suspect:
     - RuntimeDriver background polling loop (Plugin.cs:693+)
     - ModPlatform.LoadPacks() blocking on schema validation
     - F9/F10 key polling interference

ELSE IF logs show no errors but loading bar still stalled at 70%
  → Unity scene transition hook is blocked. Investigate DINO's NativeMenuInjector
     or SceneManager callbacks in game code (outside DINOForge scope).
```

---

## Key Log Locations

| Log | Command | Contains |
|-----|---------|----------|
| `dinoforge_debug.log` | `tail -f BepInEx/dinoforge_debug.log` | ECS discovery, AssetSwap, HotReload, Pack load |
| `LogOutput.log` | `tail -f BepInEx/LogOutput.log` | Plugin lifecycle, GameBridgeServer, exceptions |

---

## Next Steps if 70% Persists

1. **Verify HandleConnect deployed**: `dotnet build` → binary match at `BepInEx/plugins/DINOForge.Runtime.dll` 
2. **Collect fresh logs** from a clean launch (delete old logs first)
3. **Check timestamps** in `dinoforge_debug.log` for gaps >2s
4. **If all logs appear normal**: Issue is in vanilla game or DINO's scene loader (not DINOForge plugin scope)
