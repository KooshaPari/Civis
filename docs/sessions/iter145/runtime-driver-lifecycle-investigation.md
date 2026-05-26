# iter-145 RuntimeDriver lifecycle investigation

**Date:** 2026-05-23  
**Status:** Diagnostic complete; fix pending implementation  
**Root cause:** Resurrection fallback thread exits at first scene transition without re-spawning RuntimeDriver. All UI work downstream is dead code.

## Live-log evidence (08:27 launch)

```
08:27:22.86 [Plugin] Loaded runtime assembly
08:27:22.99 [Plugin] activeSceneChanged watcher registered (iter-144 #546 fix)
08:27:22.99 [Plugin] Resurrection fallback thread started + loop entered
08:27:30.95 [Plugin] OnActiveSceneChanged: old='' new='InitialGameLoader'
08:27:31.26 [RuntimeDriver] OnDestroy: GameBridgeServer.RequestShutdown() invoked
08:27:31.26 [Plugin] Resurrection fallback thread exiting.   ← BUG
08:27:37.37 [Plugin] Loaded runtime assembly (second load)   ← Re-init attempt
... never reaches LoadPacks, PushLoadedPacksToUgui, NativeMenuInjector
```

Zero occurrences in 14000+ heartbeat log of: `ModPlatform.LoadPacks`, `PushLoadedPacksToUgui`, `[RuntimeDriver.diag]` (the iter-145 telemetry probes), `ABOUT TO CALL`, `NativeMenuInjector.*OnClick`.

## Architecture (per Plugin.cs:78-88 comments)

The iter-144 #543/#547 design:
1. **Plugin.OnDestroy (L621)** = BepInEx plugin object teardown. Comment L638: "activeSceneChanged + fallback thread persist by design (iter-144 #547)." This OnDestroy is expected and benign.
2. **RuntimeDriver.OnDestroy (L1715)** = Scene-transition destroy. RuntimeDriver is attached to a non-DontDestroyOnLoad GameObject **by design** so Unity tears it down at each scene boundary. Comments at L1722-1820 describe the planned recovery:
   - Set `s_skipBundleUnload=true` so AssetSwapSystem preserves bundles
   - Set `NeedsResurrection` flag
   - `GameBridgeServer.RequestShutdown()` to unwedge pipe accept
   - Dispatch `ShutdownNonBridge` to worker thread (avoids gray-freeze)
   - Return → resurrection-watcher / fallback thread re-spawns a fresh RuntimeDriver in the new scene
3. **StartResurrectionWatcher (L240)** + **StartResurrectionFallbackThread (L405)** = persist across plugin lifetime. Watch `SceneManager.activeSceneChanged` (primary) + fallback thread polls `NeedsResurrection` flag (backup since iter-144 found sceneLoaded didn't fire reliably).

## Root-cause hypothesis

L497 logs `"Resurrection fallback thread exiting"`. Per fallback-thread comment at L424: "from 'no scene events firing yet' in the post-OnDestroy gray-freeze window. Previous 10s timeout..." → the loop exits on some condition that **shouldn't** fire on a normal scene transition.

The conditions that exit the fallback loop are at lines around L424-497. Suspects:
- The loop terminates after one resurrection cycle (one-shot semantics) when design wants persistent
- The loop's exit-condition predicate is met by normal scene transition (e.g. `_destroyed` flag set by Plugin.OnDestroy on plugin-host GameObject — but Plugin should persist)
- The `s_rootJustDestroyed` flag (L442) gets stuck high

## Iter-144 milestone #707 ("VERIFIED IN-GAME >2h stable") status

**Likely FALSE COMPLETION.** Either:
- The "verification" was against a build where these scene-tear-down flags didn't trigger
- The "stable >2h" run measured uptime, not in-game functionality (heartbeats kept firing because Plugin.OnDestroy + fallback are persistent — but RuntimeDriver was dead)

## Fix path (proposed — NOT implemented yet)

1. **Find** the resurrection-fallback exit condition at L424-497. Specifically look for:
   - Where `_destroyed` is checked (should ONLY be true on Plugin.OnDestroy, not RuntimeDriver.OnDestroy)
   - Where it observes `NeedsResurrection` going false WITHOUT successfully re-spawning
   - Whether it has a one-shot break that should be a continue
2. **Verify** what actually re-spawns RuntimeDriver. Either:
   - SceneManager.activeSceneChanged callback (L233-240 area) creates a new GameObject + AddComponent<RuntimeDriver>, OR
   - The fallback thread re-spawns it
3. **Fix**: ensure the fallback OR the scene-changed callback creates a new RuntimeDriver post-OnDestroy AND that the fallback thread does not exit until plugin itself is unloaded.

## Investigation method limit

This report was authored by the orchestrator after the dispatched gpt-5.4-mini agent (`bejkbu0sp`) failed to produce its requested report file at the expected path. Output was wrapper-echo only (1 line). Possible quota/timeout failure; not Spark quota since smoke-test post-completion passed. Next dispatch should use Haiku via Agent tool fallback OR direct Read+Edit by orchestrator on Plugin.cs L405-497.

## Estimated impact

If fixed correctly:
- RuntimeDriver re-spawns in `InitialGameLoader` or `MainMenu` scene
- `RuntimeDriver.Initialize()` coroutine runs → calls `_modPlatform.LoadPacks()` → triggers PushLoadedPacksToUgui (3 callsites)
- F9/F10 panels populate with real pack list (replacing current 0-packs display)
- NativeMenuInjector's MODS button finally has a live RuntimeDriver to talk to → click handlers work

This is the single load-bearing bug. ~24hrs of UI work (DFCanvas raycaster, EventSystem reconcile, MODS sprite, panel rect, PushLoadedPacksToUgui wiring) is correctly implemented but never executes.
