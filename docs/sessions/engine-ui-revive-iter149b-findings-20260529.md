# Engine-UI revive — iter-149b live findings (2026-05-29)

Branch: `fix/engine-ui-injection-race-20260529`  Base commit before this work: `6be0f5e3`
This work commit: `6416eadb`  Deployed DLL hash: `112443AE984CEAAD` (prior `7B06EB1447C326EC`)

## What was fixed (both assigned blockers — verified working in-game)

### Blocker 1 — bg fallback wedged on pipe restart (FIXED, confirmed improved)
- `ResurrectionFallbackLoop` no longer calls `SharedBridgeServer.EnsureServerAlive()` at the top of
  every iteration (that pipe `Stop()`→`Start()` blocked the heartbeat during asset load).
- Pipe keepalive moved to (a) the main-thread PlayerLoop `%60` gate and (b) a new dedicated
  `PipeKeepAliveLoop` background thread (`StartPipeKeepAliveThread`) that may block on pipe I/O
  without affecting resurrection heartbeats.
- The fallback loop's grace-window path now **only MARKs** `NeedsDeferredResurrection` (no bg-thread
  Unity ECalls) for a main-thread consumer to execute. It no longer calls `TryResurrect` on the bg
  thread.
- **Live proof of improvement:** cycle 2 emitted `ResurrectionFallback heartbeat #12 NeedsRes=True
  NeedsDefRes=True rootNull=True` AFTER `OnDestroy` — previously heartbeat #24 never appeared at all.
  The heartbeat now survives one tick past OnDestroy with the need flags correctly set.

### Blocker 2 — MainMenu emits no `activeSceneChanged` (FIXED the hook; keystone confirmed)
- Added `SceneManager.sceneLoaded` subscription, logging `name / buildIndex / mode / isLoaded` on
  every scene event, routed through a shared main-thread `MainThreadReviveIfNeeded()` that performs
  `TryResurrect` on the Unity main thread.
- **Live proof the diagnosis was right:** `OnSceneLoaded: name='InitialGameLoader' buildIndex=0
  mode=2 isLoaded=True` — **mode=2 == `LoadSceneMode.Additive`.** DINO loads scenes ADDITIVELY, which
  is exactly why `activeSceneChanged` was silent for MainMenu. `sceneLoaded` is the correct hook.
- `OnPlayerLoopSet postfix fired — DINOForgeUpdateMarker re-injected=True` confirms the PlayerLoop
  marker DOES survive re-injection through the InitialGameLoader loop rebuild.

## RESULT: MODS button still does NOT appear — a THIRD, deeper blocker is exposed

Two clean relaunch cycles, fully deterministic. Process `Responding=True`, CPU rising (main thread
alive), MainMenu renders completely (screenshot `docs/screenshots/engine-ui-FIXED-20260529.png`
shows full menu — NOT a gray freeze) — but **no MODS button**.

### Blocker 3 (NEW, keystone) — managed plugin runtime wedges at `World.Dispose()` on the InitialGameLoader→MainMenu transition

Exact freeze signature, identical across both cycles and ALL prior runs:
```
... OnActiveSceneChanged new='InitialGameLoader'
... OnSceneLoaded name='InitialGameLoader' mode=2 (Additive)
... KeyInputSystem.OnDestroy  /  AssetSwapSystem.OnDestroy   <- DINO disposes the Default World
... ResurrectionFallback heartbeat #12 NeedsRes=True NeedsDefRes=True rootNull=True  (cycle 2)
... grace deadline armed (4000ms)
... RuntimeDriver OnDestroy: GameBridgeServer.RequestShutdown() invoked
... RuntimeDriver OnDestroy: metrics snapshot written
... RuntimeDriver OnDestroy: returning to Unity (resurrection flags set...)
<<< LOG FREEZES HERE PERMANENTLY — no MainMenu scene event, no grace-elapsed MARK, no revive >>>
```

Decisive evidence this is a native managed-runtime wedge (NOT pipe, NOT scene-event, NOT DebugLog):
1. **BepInEx's own `LogOutput.log` — a fully independent writer that does NOT use our `DebugLog._lock`
   — freezes at the EXACT SAME timestamp** (`02:50:42.489` cycle 1). So it is not a DebugLog lock
   deadlock; the entire BepInEx/Mono managed-plugin thread pool is wedged.
2. The Unity render thread keeps running (CPU rising, music + menu interactive on screen), so the
   freeze is confined to the Mono-managed plugin world, not the game.
3. The freeze point is `World.Dispose()` (we patch it as `WorldDisposeGuardPatch` — H8 probe) during
   the scene transition that destroys the InitialGameLoader Default World before MainMenu activates.
4. Because the managed threads wedge BEFORE MainMenu activates, **no MainMenu `sceneLoaded` /
   `activeSceneChanged` ever fires** — so the (correct) Blocker-2 hook never gets a MainMenu event to
   act on, and the (correct) Blocker-1 grace MARK never elapses.

This is a recurrence of the iter-144 gray-freeze native deadlock (`mono_jit_cleanup` →
`WaitForMultipleObjectsEx`, WinDbg-confirmed previously). The prior fix
(`GameBridgeServer.RequestShutdown()` at the TOP of OnDestroy) IS present and DOES fire here, but it
is no longer sufficient — the wedge survives it. The remaining wedge is in the world-dispose /
post-OnDestroy teardown path, not in the pipe accept loop the prior fix addressed.

### Recommended next step (requires native diagnosis — beyond the two localized blockers)
- Attach WinDbg / capture a full-process MDMP at the frozen state (process stays alive + Responding,
  so it is dumpable) and walk the managed-plugin thread stacks to find what `World.Dispose()` (or a
  Harmony guard prefix/postfix on it, or `AssetBundle.Unload` / `Resources.UnloadUnusedAssets` H7-H9
  probes) blocks on. Candidates: a Harmony guard patch taking a lock that the disposing thread holds;
  a `Resources.*`/asset API call made from a guard during dispose; or a finalizer/GC stall.
- Until the world-dispose wedge is cleared, the engine UI cannot revive on MainMenu by ANY hook,
  because the managed runtime is frozen before MainMenu activation.

## Build / deploy facts
- `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:TargetFramework=netstandard2.0` — exit 0.
- 21 resurrection / KeyInputSystem / PlayerLoop tests pass.
- Deploy hashes: prior deployed `7B06EB14`; built (deploy) `112443AE`; deployed-now `112443AE`
  (deployed == built, differs from prior — deploy proven by hash).

## Verdict
- MODS button + F9/F10 load? **NO.** Both assigned blockers are fixed and verified
  (sceneLoaded fires, mode=Additive confirmed, marker re-injects, pipe call removed, heartbeat
  survives one tick further), but a third native blocker — the managed-plugin runtime wedging at
  `World.Dispose()` on the MainMenu transition — prevents any revive. PR NOT opened (modsButton=False).
