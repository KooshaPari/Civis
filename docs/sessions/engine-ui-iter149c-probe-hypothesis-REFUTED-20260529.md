# iter-149c — "diagnostic probes cause the World.Dispose wedge" hypothesis: REFUTED (2026-05-29)

Branch: `fix/engine-ui-injection-race-20260529`  Commit: `e8e87a24`
Built/deployed DLL hash: `33A98D0EC737AE59` (prior deployed `112443AE984CEAAD`)

## Hypothesis tested
The iter-144 H7/H8/H9 DIAGNOSTIC Harmony probes (Prefix/Postfix on
`Resources.UnloadUnusedAssets`, `AssetBundle.Unload`, `AssetBundle.LoadFromFile`,
`SceneManager.UnloadSceneAsync`, `Unity.Entities.World.Dispose`) were suspected of CAUSING
the InitialGameLoader->MainMenu wedge. Each prefix runs `new StackTrace()` + synchronous
BepInEx logging inside the native dispose/unload path; the theory was this contends the
BepInEx log lock and wedges the managed thread mid-`World.Dispose()`.

## Change made
`src/Runtime/Plugin.cs` — gated all 5 diagnostic probe `.Apply()` calls behind
`const bool EnableDisposeProbes = false`. `DestroyGuardPatch` (protects DINOForge_Root) and
`ModsButtonTextPatch` (engine-UI label) remain active. Probe files kept intact.

## Verification the probes were actually OFF
- `LogOutput.log`: `Harmony initialized and patches applied (disposeProbes=False).`
- Zero `"probe active"` lines (H7/H8/H9 never registered).
- Zero `WorldDispose|ResourcesUnload|AssetBundleUnload|AssetBundleLoad|SceneUnload` ENTER lines
  in either log across both relaunch cycles.

## Live result — TWO deterministic relaunch cycles
Both cycles froze at the IDENTICAL line, same as every prior run WITH the probes:
```
[RuntimeDriver] OnDestroy: returning to Unity (resurrection flags set, fallback thread will revive).
```
- Cycle 1: debug.log mtime frozen at 03:08:45, stale 113s+ when checked. Process PID alive,
  Responding=True, CPU=132s, 140 threads, WS=4085MB.
- Cycle 2: debug.log frozen at 03:11:22, stale 86s+. Responding=True, CPU=127s, 141 threads.
- `LogOutput.log` (independent BepInEx writer) also frozen — full managed-plugin thread wedge.
- Screenshot `docs/screenshots/engine-ui-iter149c-20260529.png`: MainMenu renders FULLY
  (NOT gray) with all vanilla buttons — but **NO MODS button**. Render thread alive; only the
  Mono-managed plugin world is wedged.

## VERDICT: HYPOTHESIS REFUTED
Disabling the diagnostic probes did NOT change the freeze point or unwedge the runtime. The
probes are NOT the cause. The wedge is in the World.Dispose / post-OnDestroy teardown path
itself (recurrence of the iter-144 `mono_jit_cleanup -> WaitForMultipleObjectsEx` native
gray-freeze), independent of our Harmony instrumentation.

Engine UI FIXED? **NO.** modsButton still False. No `ENGINE-UI READY` line ever emitted
(managed runtime wedges before MainMenu activation, so no revive hook can fire). PR NOT opened.

## Recommended next step (native diagnosis required)
Process stays alive + Responding at the frozen state, so it is dumpable. Capture a full-process
MDMP at the freeze and walk the managed-plugin thread stacks (as in the iter-144 WinDbg
investigation). With the probes now ruled out, the remaining suspects are in DINOForge's own
OnDestroy/resurrection teardown path or a Unity/Mono internal during the 45K-entity Default
World dispose — NOT a Harmony guard. The probe-gating change is a net positive (removes 5
StackTrace-in-hotpath probes from the teardown) and is worth keeping regardless.
