# iter-149d BISECT: PipeKeepAlive RULED OUT as the engine-UI gray-freeze wedge

**Date:** 2026-05-29 (UTC 2026-05-30 ~03:22)
**Branch:** `fix/engine-ui-injection-race-20260529`
**Bisect commit:** `f5c1b454`
**Verdict:** PipeKeepAlive is **NOT** the wedge. The gray-freeze recurs at the identical teardown point with PipeKeepAlive disabled.

## Hypothesis tested

`PipeKeepAliveLoop` (Plugin.cs ~L397-433, started by `StartPipeKeepAliveThread()` at the
`StartResurrectionWatcher` call site ~L318) polls `SharedBridgeServer.EnsureServerAlive()` every
1s on a background thread. `EnsureServerAlive` (GameBridgeServer.cs L235-264) performs a pipe
`Stop()->Start()` whenever the bridge server thread is dead — which is ALWAYS true right after
`RuntimeDriver.OnDestroy` calls `RequestShutdown()`. The hypothesis: during teardown, OnDestroy
disposes the pipe handle (iter-144 fix) to unwedge the accept thread, but PipeKeepAlive instantly
re-creates a `NamedPipeServerStream` + re-arms `BeginWaitForConnection`, re-establishing the exact
kernel `ConnectNamedPipe` wait that `RequestShutdown` just tore down — becoming the new
un-interruptible waiter that wedges `mono_jit_cleanup` during `World.Dispose`.

## Change applied (Plugin.cs, StartResurrectionWatcher call site ~L316-338)

Gated `StartPipeKeepAliveThread()` behind `const bool EnablePipeKeepAlive = false`. With the gate
off, the loop never starts and nothing on a background thread re-arms the pipe during teardown.
The `ResurrectionFallbackLoop` was already verified to NOT call `EnsureServerAlive` (Blocker-1 fix,
6416eadb). The PlayerLoop (Plugin.cs L960) and `KeyInputSystem.OnUpdate` (KeyInputSystem.cs L330)
EnsureServerAlive callers run on the Unity main thread, so they cannot race World.Dispose on the
same thread (and ECS system groups do not fire during the menu/teardown window).

## Build / deploy proof

- `dotnet build ...DINOForge.Runtime.csproj -c Release -p:TargetFramework=netstandard2.0` -> exit 0 (114 warnings, 0 errors)
- Commit `f5c1b454`
- Deploy hashes (SHA256, first 16):
  - PRIOR deployed: `33A98D0EC737AE59`
  - BUILT:          `0CEC46501EFDD78C`
  - DEPLOYED:       `0CEC46501EFDD78C`  (== BUILT, != PRIOR — verified live)

## Live verification (decisive)

`dinoforge_debug.log` confirmed the new DLL ran: `"PipeKeepAlive thread DISABLED (iter-149d bisect...)"`
logged on both InitialGameLoader cycles; **zero** PipeKeepAlive loop activity
(`loop entered` / `EnsureServerAlive` / `thread exiting`) for the run.

Relaunch 2 (authoritative, 20:22:29 local):
- Process: `Responding=True`, title `Diplomacy is Not an Option`, CPU climbing 29s -> 49s -> 58s.
- `dinoforge_debug.log` FROZE at **20:22:43.156** at the line
  `[RuntimeDriver] OnDestroy: returning to Unity (resurrection flags set, fallback thread will revive).`
  Stuck at 39175 lines; mtime did not advance for >75s.
- `BepInEx/LogOutput.log` last line `[Plugin] BepInEx plugin object OnDestroy (persistent root still alive).`
  mtime frozen 20:22:41 — both logs froze at the same teardown instant.
- Last `ResurrectionFallback heartbeat` was #8 @ 20:22:41 — the surviving managed background
  heartbeat ALSO stopped, consistent with a native `mono_jit_cleanup -> WaitForMultipleObjectsEx`
  block that halts even managed threads.
- Screenshot: `docs/screenshots/engine-ui-iter149d-20260529.png` — game frozen on the "407 Door /
  Unity / fmod" studio splash; never reached MainMenu. No MODS button.

### THE ANSWER (verbatim deliverable)

- Does the log keep writing past `OnDestroy: returning to Unity`? **NO** — froze at that exact line
  (20:22:43.156), identical to the original bug.
- `ENGINE-UI READY modsButton=True`? **Never emitted.** No MainMenu `sceneLoaded`, no
  `MainThreadReviveIfNeeded` revive, no `ENGINE-UI READY` line.

## Verdict

**Engine UI FIXED? NO.** PipeKeepAlive is **RULED OUT**. The un-interruptible waiter that wedges
`mono_jit_cleanup` during `World.Dispose` is something OTHER than the PipeKeepAlive re-arm. Disabling
the 1s background pipe re-arm did not change the freeze point at all.

The parallel WinDbg agent's MDMP stack is now the authoritative next step — it will show the exact
native wait the main thread is parked on during World.Dispose. Candidates not yet excluded:
- A different synchronous pipe/handle op still reachable on the teardown path (e.g. `_session?.Dispose()`
  in `Stop()`, or a pipe read/write in flight when RequestShutdown fires).
- An FMOD / audio or Addressables/asset-load native wait entangled with World.Dispose during the
  InitialGameLoader->MainMenu transition (the freeze is on the studio splash, before MainMenu).
- A Burst/job-system or ECS World teardown join that blocks in native code regardless of the pipe.

The bisect commit `f5c1b454` is retained on the branch as a reverted-by-default safe change (the
gate is a `const`, trivially flipped back). No PR opened (verdict is "still frozen").

## Note on relaunch 1

Relaunch 1 (20:20:36) ran two InitialGameLoader cycles and the process exited (NOT a hang — log went
to 39105 lines then process gone), almost certainly because the parallel WinDbg agent attached and
terminated it mid-dump. Relaunch 2 (with debuggers pre-killed) is the authoritative observation and
clearly reproduced the wedge.
