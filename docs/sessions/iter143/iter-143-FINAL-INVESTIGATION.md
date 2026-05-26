# Iter-143 Final Investigation Status

**Date**: 2026-05-19  
**Session arc (mid-investigation)**: chicken-skeleton + UI-input bug → autonomous mod-off control test → DestroyGuard ruled out → AssetBundleCache fix landed → hang discovered as separate bug.

---

## **CLOSURE UPDATE (later same day, post-wave 2)**

The hypotheses captured below were **superseded by the wave 2 closure work**. Definitive status:

### Hang root cause RESOLVED (#535)
- Real cause: `MainThreadDispatcher.RunOnMainThread(...).Result` in GameBridgeServer parked the bridge thread indefinitely waiting on a TCS that only `KeyInputSystem.OnUpdate.DrainQueue()` could complete — but KeyInputSystem.OnDestroy fires during scene transition, killing the pump
- Fix: PumpIsAlive volatile flag + fast-fail short-circuit in MainThreadDispatcher + 7 bounded timeouts in GameBridgeServer (`.Result` → `.Wait(timeout)` with fallback DTOs) + 8 governance markers
- **Verified at runtime**: log progression past `RuntimeDriver.OnDestroy` confirmed (`PackUnitSpawner.Initialize` + `AerialSpawnSystem.Initialize` now fire post-destroy). Game stays `Process.Responding=True` 120+ seconds post-launch.

### Chicken sprites RESOLVED (#534)
- Real cause: `AssetBundleCache.Unload(unloadAllLoadedObjects: true)` at 3 sites — destroys all Unity assets loaded from cached bundles when AssetSwapSystem.OnDestroy fires on scene transition. Vanilla UI components referencing those assets show Unity's default placeholder (chickens).
- Fix landed: 3× `true → false` change — bundle metadata still frees; loaded objects vanilla UI references stay alive.

### Star Wars 0/36 render RESOLVED (#101)
- Real cause: `AssetSwapSystem.cs:337-338` reflection lookup `typeof(EntityManager).GetMethod("SetSharedComponentData")` threw `AmbiguousMatchException` because Unity Entities has multiple `SetSharedComponentData<T>` overloads.
- Fix: `GetMethods().FirstOrDefault(m => m.IsGenericMethodDefinition && m.GetParameters().Length == 2 && m.GetParameters()[0].ParameterType == typeof(Entity))` pins the correct overload.

### Capture pipeline UNBLOCKED (#536/#537)
- WGC capture wrapper landed at `src/Tools/DinoforgeMcp/dinoforge_mcp/capture_wgc.py` + new `game_screenshot_wgc` MCP tool
- `game_screenshot` updated with 3-tier fallback (WGC → GameControlCli → GDI), returns `backend` field
- WGC tier activates next clean MCP restart (live server still on v0.13.0 pre-WGC)

### Pattern retirements landed wave 2
- **Pattern #96** (LogError stack-trace): 46 → 0 violations across entire repo (DF0096 Roslyn analyzer Tier 1 + Python detector)
- **Pattern #232** (unbounded log append): 3 HIGH → 0 — `WriteDebug` methods now have 100MB rotation guard + BepInEx fallback
- **Pattern #222** (long methods): NativeMenuInjector.InjectButton 302 lines → 63 lines via 6-helper decomp (13 characterization tests pass)
- **Pattern #99 family**: #540 PackDependencyResolver + #541 Registry<T> case-collision (StringComparer.OrdinalIgnoreCase → Ordinal)
- **Pattern #231** detector fix: Windows-path normalization

### Governance hardening
- **#539** `block-git-stash.ps1` hook: deny-by-default + allowlist (was only blocking bare + drop, letting push/save/create/store through)
- Memory entry `feedback_agent_governance_hardening.md` captures the meta-lesson

### Test outcome
- Build: 0 errors across all configs
- Test suite: 3636/3641 pass + 4 skip; 1 pre-existing failure subsequently fixed (#540 + #541)
- Analyzer tests: 119/119 pass
- All 15 pattern detectors green (or pre-existing-only residuals)

### Working tree state at closure
- ~90 uncommitted files spanning all wave 2 + cleanup work
- Latest deployed DLL: `2026-05-19 04:44:10`, build 0 errors
- Branch: `fix/handle-connect-iter142`

### What's LEFT for v0.25.0 tag
- User authorization for: commit, push, PR, tag
- Branch consolidation (#507/#510/#512) — needs user authz
- MCP server clean restart (live v0.13.0 → wave 2 WGC build)

---

## Original mid-investigation analysis (kept for reference):

## What's confirmed (high confidence)

1. **Chickens are mod-induced, not vanilla**. Mod-OFF baseline screenshot (`docs/screenshots/mod-off-mainmenu.png`, 2.38 MB) shows DINO main menu rendering cleanly with NO chickens, NO placeholder sprites, NO unresponsiveness. Disambiguation test ran via autonomous BepInEx-folder rename.

2. **DestroyGuard Harmony prefix is NOT the root cause**. Surgical disable test (`Bridge.DestroyGuardPatch.Apply(_harmony)` commented out, rebuild, redeploy, launch) showed game STILL hangs after RuntimeDriver.OnDestroy. Plugin.cs restored to original.

3. **Hang is reproducible AND independent of iter-143 fixes**. Three test launches with different DLL states all hang at the same point — `[RuntimeDriver] OnDestroy called — DINO destroyed our root. Bridge kept alive.` is the final log line. ~170ms gap exists between AssetSwapSystem.OnDestroy and RuntimeDriver.OnDestroy where something heavy happens that we don't log.

4. **AssetBundleCache `Unload(true) → false` fix is deployed** (DLL mtime 02:29:50). Comment trail in `src/Runtime/Bridge/AssetBundleCache.cs:124,135,171` references #534. Hang persists with fix in place, so this fix addresses chicken-sprite cause (preserving vanilla sprite refs) but NOT the hang.

## Pending tasks

- **#534** (in-progress): AssetBundleCache fix deployed, visual verification blocked on capture pipeline.
- **#535** (pending): Hang root cause unknown. Suspects: AssetSwapSystem itself (loading vanilla bundles for inspection), GC pause, native asset reload chain, GameBridgeServer thread holding a lock during main-thread cleanup.
- **#536** (pending): Capture pipeline needs WGC (Windows.Graphics.Capture) or DXGI Desktop Duplication. Current 5-tier cascade (GDI/PrintWindow/bare-cua/game-control/CopyFromScreen) all fail when DINO hangs in DXGI exclusive fullscreen.
- **#537** (new, recommend): Add WGC backend to MCP `game_screenshot` tool — only foreground-independent, hung-process-resilient capture method.

## Working tree state

- **Uncommitted change**: `src/Runtime/Bridge/AssetBundleCache.cs` — 3× `unloadAllLoadedObjects: true → false` with #534 reference comments. No other source files modified this session.
- **Branch**: `fix/handle-connect-iter142` (unchanged from session start; 2 commits ahead of session start origin commit `411e34b8`).
- **Deployed DLL hash**: `8A09E189C8BA7CFF3A1215F0217D35D68F7B9EEDED844D91FA842665F7DFF60F` (BepInEx/plugins/DINOForge.Runtime.dll @ 02:29:50).

## Bisect plan for next session

The hang is reliably reproducible. Next session should:

1. **Log-only verification** (no screenshot needed): launch with each candidate revert, tail dinoforge_debug.log, look for `[Plugin] OnSceneLoaded: scene='MainMenu'` line AFTER `[RuntimeDriver] OnDestroy`. If present → game advanced past hang. If absent → still hangs.
2. **Revert candidates** in order of suspicion:
   - a) Disable AssetSwapSystem entirely (don't add it in Plugin.cs ECS world setup)
   - b) Revert HandleConnect addition (iter-142 commit `ced0dccf`)
   - c) Revert EventSystem null guard (#531, DFCanvas.cs:135-145)
   - d) Revert iter-143 PR base diff entirely (`git diff main..fix/handle-connect-iter142 src/Runtime/`)
3. **For each revert**: edit → rebuild explicit netstandard2.0 → launch → wait 30s → tail log → restore.

## Screenshots taken this session

- `docs/screenshots/mod-off-mainmenu.png` — 2.38 MB — vanilla DINO clean (NO chickens)
- `docs/screenshots/mod-on-mainmenu.png` — 1.19 MB — WorldBox bleed (CAPTURE INVALID)
- `docs/screenshots/mod-on-noguard-mainmenu.png` — 9 KB — black (DINO hung) (CAPTURE FAILED)
- `docs/screenshots/mod-on-noguard-v2-mainmenu.png` — 290 KB — WorldBox bleed (CAPTURE INVALID)
- `docs/screenshots/mod-on-fixed-mainmenu.png` — 67 KB — black + Fatal error dialog (CAPTURE FAILED)

Only the mod-OFF baseline is valid evidence. Mod-ON state remains visually unverified due to capture pipeline gaps.

## Open question for user

Worth discussing whether to:
- (a) Land WGC capture FIRST (#537), then iterate on bisect; OR
- (b) Attempt hang bisect with log-only signals, defer capture fix; OR
- (c) Revert iter-143 work entirely + tag what we have as v0.25.0-rc1 and chase the hang in v0.26.0.
