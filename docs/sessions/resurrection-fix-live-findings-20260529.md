# Main-thread resurrection fix — live verification findings (2026-05-29)

Branch: `fix/engine-ui-injection-race-20260529`  Commit: `6be0f5e3`
Deployed DLL: `7B06EB1447C326EC` (prior `8240D91ED2FD8559`)

## What was completed (code)
- Implemented the missing `Plugin.ConsumeResurrectionOnMainThread()` (main-thread, throttled %60,
  cap-guarded, never throws — Pattern #104/#111). Verified `ResurrectionSucceeded()` and
  `ResetGraceDeadline()` already present and correct.
- Confirmed `GameBridgeServer.EnsureServerAlive` no longer calls `TryResurrect` (bg thread only
  restarts the pipe server thread + `MarkNeedsDeferredResurrection`).
- Confirmed revive path reaches `RunMainMenuInit` via `RuntimeDriver.Initialize` on menu scenes.
- Build netstandard2.0 + net8.0 exit 0. 20 resurrection/KeyInputSystem tests pass. CHANGELOG updated.

## Live result: engine UI does NOT reliably load (MODS button absent)
Two clean relaunches, both deterministic:
- Game reaches MainMenu and renders fully (screenshot `docs/screenshots/engine-ui-resurrection-fix-20260529.png`
  shows interactive menu — NOT a gray freeze). Process Responding=True.
- BUT no MODS button, and the DINOForge debug log goes **completely silent** after
  `RuntimeDriver OnDestroy: returning to Unity` on the `InitialGameLoader` transition. No MainMenu
  `activeSceneChanged`, no `ConsumeResurrectionOnMainThread`, no `ENGINE-UI READY` line ever logged.

## Root cause of the remaining failure (precise)
1. **Background fallback thread wedges on the PIPE restart, not Unity ECalls.** Heartbeats run on a
   clean 2s cadence (#4,#8,#12,#16,#20) until the exact moment `NeedsResurrection` becomes true at
   OnDestroy; heartbeat #24 never appears. The fallback loop calls
   `SharedBridgeServer?.EnsureServerAlive()` at the TOP of every iteration. The TryResurrect deadlock
   was correctly removed, but `EnsureServerAlive` still calls `Stop()`→`Start()` to restart a dead
   pipe server thread (GameBridgeServer.cs ~257-263). That pipe teardown/recreate (NamedPipeServerStream
   + thread join) blocks the bg thread during the asset-load window — so it stops heart-beating and
   never reaches the grace-window revive. The deadlock MOVED from TryResurrect to the pipe restart on
   the same background thread.
2. **Main-thread PlayerLoop consumer does not survive the MainMenu PlayerLoop rebuild.** DINO rebuilds
   its PlayerLoop entering MainMenu; our injected `DINOForgeUpdateMarker` is dropped and the
   `OnPlayerLoopSet` Harmony postfix re-injection did not log either, so `ConsumeResurrectionOnMainThread`
   never ticks on the MainMenu scene. With both the bg path wedged and the main-thread path not ticking,
   resurrection never fires.

## Next iteration (actionable)
- Do NOT call the pipe-restarting `EnsureServerAlive()` from the fallback loop body before the revive
  check; gate pipe-restart behind a non-blocking check or move it off the fallback thread. Pipe
  Stop/Start must not block the resurrection heartbeat.
- Make the PlayerLoop re-injection robust across DINO's MainMenu PlayerLoop rebuild (verify
  `OnPlayerLoopSet` postfix fires; consider re-injecting from `activeSceneChanged` on the main thread).
- Verify the MainMenu `activeSceneChanged` actually reaches `OnActiveSceneChanged` (no log line this
  run — confirm the static subscription survived and the event is delivered for the MainMenu activation).
