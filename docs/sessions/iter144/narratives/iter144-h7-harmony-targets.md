# Gray-Freeze H7 — Candidate Harmony Patch Targets (iter-144)

## Log evidence

From `dinoforge_debug.log` (window 21:42:24.964 → 21:42:25.916):

```
21:42:24.964  [Plugin] OnActiveSceneChanged: old='' new='InitialGameLoader'
21:42:24      [NativeMenuInjector] InitialGameLoader auto-advance: SceneManager.LoadScene(1)
21:42:25.222  ResurrectionFallback heartbeat #12 (still healthy)
21:42:25.379  VanillaCatalog.Build: GetAllEntities threw ArgumentNullException
21:42:25.394  [GameBridgeServer] Started on pipe: dinoforge-game-bridge
21:42:25.446  [Plugin] OnActiveSceneChanged: old='' new='MainMenu'
21:42:25.765  [KeyInputSystem] KeyInputSystem.OnDestroy: World='Default World' IsCreated=True
21:42:25.766  [AssetSwapSystem] AssetSwapSystem.OnDestroy - bundles unloaded
21:42:25.913  [RuntimeDriver] OnDestroy: NeedsResurrection set; awaiting scene transition.
21:42:25.916  [RuntimeDriver] OnDestroy.worker: ShutdownNonBridge completed.
<<< 2s later: Mono wedges, gray-freeze; no further log lines >>>
```

LogOutput.log shows ContentLoader registering 33 YAML files just before the wedge — i.e. `ModPlatform.LoadPacks` was mid-flight on the main thread when InitialGameLoader→MainMenu fired.

## DINO method candidates

### 1. `UnityEngine.Resources.UnloadUnusedAssets()` — HIGH
- Auto-invoked by Unity after `SceneManager.LoadScene` (single mode) finishes its async pump. Symptom matches: 2s gap between scene-change event and freeze = exactly the GC + asset-graph traversal pass.
- DINO has 6 GB of Addressables; UnloadUnusedAssets walks the full handle graph. Combined with our `Resources.FindObjectsOfTypeAll<Canvas>` (NativeMenuInjector.cs:312) marking everything reachable, this becomes a long pinning sweep. With ECS systems mid-OnDestroy + ModPlatform mid-pack-load, the Mono GC scan races a half-disposed world.
- Patch: **Prefix** returning `false` (no-op AsyncOperation) OR **Postfix** logging entry/exit to confirm wedge boundary.
- Risk: skipping it leaks memory but won't break gameplay (Unity calls it again on next scene load).

### 2. `UnityEngine.SceneManagement.SceneManager.LoadScene(int)` (or `LoadSceneAsync`) — HIGH
- Source: ours at `src/Runtime/UI/NativeMenuInjector.cs:375`. DINO also calls it during MainMenu→Game transitions.
- Patch: **Prefix** that ensures `_s_sceneTransitionGuard` is checked + that the ECS world is *not* mid-destroy before delegating. Better: skip the call entirely and use `SceneManager.LoadSceneAsync(1, LoadSceneMode.Single)` with a completion callback that DEFERS Resources.UnloadUnusedAssets via `allowSceneActivation = false` until our root is detached.
- Risk: misfire prevents reaching MainMenu — but we already have the auto-advance guard.

### 3. `Unity.Entities.World.DestroyAllSystemsAndLogException` / `World.Dispose` — MED
- The 21:42:25.765 `KeyInputSystem.OnDestroy` log is DINO tearing down the Default World during scene transition. Our systems' OnDestroy fires here too. If a system's Dispose hits a native container that was already freed by DINO, Mono wedges.
- Patch: **Prefix** on `World.Dispose` to detach DINOForge-owned systems first (FactionSystem, KeyInputSystem, AssetSwapSystem, etc.) via `World.RemoveSystem(...)` before DINO's tear-down proceeds.
- Risk: orphans the systems; resurrection re-creates them.

### 4. `Components.RawComponents.*` static initializers / `Systems.ContentResetSystem.OnUpdate` — LOW
- No log evidence of a ContentReset system firing. Speculative.

## Recommended next experiment

Patch **`Resources.UnloadUnusedAssets`** with a Postfix-only logger first to *confirm* the wedge boundary, NOT to skip the call:

```csharp
var m = typeof(Resources).GetMethod(
    nameof(Resources.UnloadUnusedAssets),
    BindingFlags.Public | BindingFlags.Static,
    null, System.Type.EmptyTypes, null);
var prefix  = new HarmonyMethod(typeof(UnloadGuardPatch).GetMethod(nameof(UnloadPrefix),  BindingFlags.Static|BindingFlags.NonPublic));
var postfix = new HarmonyMethod(typeof(UnloadGuardPatch).GetMethod(nameof(UnloadPostfix), BindingFlags.Static|BindingFlags.NonPublic));
harmony.Patch(m, prefix: prefix, postfix: postfix);
// UnloadPrefix:  WriteDebug("[UnloadGuard] >>> UnloadUnusedAssets ENTER"); return true;
// UnloadPostfix: WriteDebug("[UnloadGuard] <<< UnloadUnusedAssets EXIT (op != null)");
```

If the next freeze log shows `ENTER` without `EXIT`, H7 is confirmed: the wedge is inside Unity's native asset unload. Then promote to a **Prefix that returns `false`** for the first call after a scene transition where ECS world is mid-destroy.

## Risk surface
- Skipping `UnloadUnusedAssets` will accumulate memory each scene change (acceptable for a mod-dev runtime; DINO itself calls it on every subsequent transition).
- Patching `SceneManager.LoadScene` affects ALL scene transitions including MainMenu→Game, MainMenu→Saves — risk of breaking save loading. Mitigate by gating on `_s_sceneTransitionGuard` + scene name.
- Patching `World.Dispose` is the riskiest — DINO's ECS systems may rely on tear-down ordering. Test in MainMenu→Game first, not InitialGameLoader→MainMenu.

## Source references
- `src/Runtime/Bridge/DestroyGuardPatch.cs:1-143` — existing pattern for Apply()
- `src/Runtime/Plugin.cs:128-131` — Harmony bootstrap site
- `src/Runtime/UI/NativeMenuInjector.cs:375` — our SceneManager.LoadScene(1) trigger
- `src/Runtime/Bridge/GameBridgeServer.cs:1136-1138` — DINO scene-load via bridge
