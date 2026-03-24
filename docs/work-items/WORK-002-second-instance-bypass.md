# WORK-002: Second Instance Mutex/Lockfile Bypass

**Priority**: High
**Status**: Under Investigation
**Created**: 2026-03-24
**ADR**: ADR-014 (Second Instance Detection and Bypass)
**Milestone**: M5 — Dev Tooling Hardening
**Assignee**: DINOForge agents

---

## Problem

When DINOForge test workflows (e.g., `/game-test`, `/test-swap`, `/pack-deploy`) attempt to relaunch Diplomacy is Not an Option, the game presents a fatal error dialog:

```
Another instance is already running. Only one instance of the game is allowed.
```

The game process exits immediately. BepInEx never loads. The plugin never bootstraps. The test workflow fails.

### Observed Behavior

1. `/launch-game` skill calls `Stop-Process` to kill any running game instance.
2. After a 3-second sleep, the skill launches the game `.exe` directly.
3. In some cases the game starts successfully. In other cases the "another instance" error appears immediately.
4. The failure is non-deterministic — it depends on how quickly the OS kernel releases the previous process's mutex handles after the process exits.

### Impact

- Automated test loops cannot reliably run more than one game launch cycle without manual intervention.
- This breaks the agent-testing mandate: agents must never require the user to restart the game manually.
- Affects any command that re-deploys packs or hot-reloads plugins and needs to restart the game for verification.

### Root Cause Hypothesis

DINO uses a Win32 named mutex (created via `CreateMutexW`) to enforce single-instance behavior. When the previous game process is force-killed:

1. The OS eventually marks the mutex as **abandoned** — meaning the owning thread exited without calling `ReleaseMutex`.
2. On Windows, an abandoned mutex is still considered "held" from the perspective of `CreateMutex` with `bInitialOwner=true` — a second call to `CreateMutex` for the same name returns `ERROR_ALREADY_EXISTS`.
3. The game interprets `ERROR_ALREADY_EXISTS` as "another instance is running" and exits.
4. The kernel eventually cleans up abandoned mutex handles, but the timing is unpredictable — it may take seconds or may require a system call that triggers garbage collection of kernel objects.

The exact mutex name is unknown at this time. It must be discovered via investigation.

---

## Investigation Plan

All investigation steps must be completed and results recorded in this document before any implementation work begins.

### Step 1 — Identify the Mutex Name

- [ ] Install Sysinternals `Handle.exe` or `Process Explorer` on the test machine.
- [ ] Launch the game (first instance — this should succeed).
- [ ] While the game is running, enumerate all Mutant (mutex) handles held by the game process:
  ```powershell
  # Using Handle.exe (Sysinternals):
  .\handle.exe -p "Diplomacy is Not an Option" -a | Select-String "Mutant"
  ```
  Or in Process Explorer: lower pane → handles → filter Type = Mutant.
- [ ] Record all mutex names found.
- [ ] Identify the single-instance mutex by:
  - Name pattern matching game title, developer name, or "single instance" keywords
  - Eliminating well-known system mutexes (CLR, .NET runtime, Unity engine)
- [ ] Confirm mutex name by testing: open it from a second process, release it, verify game launches.

**Expected candidates** (unconfirmed — must be verified):
- `DiploNotAnOption_SingleInstance`
- `Global\DiploNotAnOption_SingleInstance`
- `Diplomacy is Not an Option`
- `Local\DiploNotAnOption`
- A GUID-based name (e.g., `{A1B2C3D4-...}`)

**Record result here**: `_mutex name: (TBD)_`

---

### Step 2 — Check for a Lockfile in AppData or Game Directory

- [ ] Snapshot the AppData directory before launching the game:
  ```powershell
  $before = Get-ChildItem -Recurse `
    "C:\Users\koosh\AppData\LocalLow\Door 407\Diplomacy is Not an Option\" |
    Select-Object FullName, LastWriteTime, Length
  ```
- [ ] Launch the game and wait 5 seconds.
- [ ] Snapshot again and diff:
  ```powershell
  $after = Get-ChildItem -Recurse `
    "C:\Users\koosh\AppData\LocalLow\Door 407\Diplomacy is Not an Option\" |
    Select-Object FullName, LastWriteTime, Length
  Compare-Object $before $after -Property FullName
  ```
- [ ] Also check `$env:TEMP` and `$env:LOCALAPPDATA` for new DINO-related files.
- [ ] Check known Unity lock locations:
  - `C:\Users\koosh\AppData\LocalLow\Door 407\Diplomacy is Not an Option\Unity\` — inspect all files
  - Game install root: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\` — check for `.lock` files
- [ ] If a lockfile is found, note its full path and whether it is exclusively locked (test with `[IO.File]::Open` or `handle.exe`).

**Record result here**: `_lockfile path: (TBD, or "none found")_`

---

### Step 3 — Test BepInEx Preloader Patcher Timing

- [ ] Write a minimal preloader patcher (`DINOForge.TimingProbe.dll`) that writes a timestamp to a log file in `Awake()`.
- [ ] Install to `BepInEx/patchers/`.
- [ ] Launch the game — compare the patcher log timestamp against when the "another instance" dialog appears (if it appears).
- [ ] Determine: does the patcher run **before** the mutex check fires?

**This is the feasibility gate for Option A (preloader patcher approach).**

**Record result here**: `_patcher timing: runs before / after mutex check (TBD)_`

---

### Step 4 — Measure Kernel Mutex Hold Time After Process Kill

- [ ] Launch the game (first instance).
- [ ] Force-kill the game process.
- [ ] Immediately begin polling: attempt to launch the game every 2 seconds.
- [ ] Record the time from kill to first successful launch.
- [ ] Repeat 5 times to get a distribution.

```powershell
function Measure-MutexRelease {
    $gamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
    Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
    $killTime = Get-Date
    $attempt = 0
    while ($true) {
        $attempt++
        Start-Sleep -Seconds 2
        $proc = Start-Process -FilePath $gamePath -PassThru -WindowStyle Hidden
        Start-Sleep -Seconds 3
        if ($proc.HasExited -and $proc.ExitCode -eq -1073741510) {
            # Exit code for "another instance" — try again
            continue
        }
        $elapsed = (Get-Date) - $killTime
        Write-Host "Success on attempt $attempt after $($elapsed.TotalSeconds)s"
        $proc | Stop-Process -Force
        return $elapsed.TotalSeconds
    }
}
```

**Record result here**: `_average mutex hold time after kill: (TBD) seconds_`

---

### Step 5 — Test P/Invoke Mutex Release (PowerShell)

- [ ] Write a PowerShell proof-of-concept using P/Invoke:
  ```powershell
  Add-Type -TypeDefinition @'
  using System;
  using System.Runtime.InteropServices;
  public class MutexUtils {
      [DllImport("kernel32.dll", CharSet=CharSet.Unicode)]
      public static extern IntPtr OpenMutex(uint access, bool inherit, string name);
      [DllImport("kernel32.dll")]
      public static extern bool ReleaseMutex(IntPtr hMutex);
      [DllImport("kernel32.dll")]
      public static extern uint WaitForSingleObject(IntPtr hObject, uint ms);
      [DllImport("kernel32.dll")]
      public static extern bool CloseHandle(IntPtr h);
      [DllImport("kernel32.dll")]
      public static extern uint GetLastError();
  }
  '@

  $MUTEX_ALL_ACCESS = 0x1F0001
  $WAIT_ABANDONED = 0x00000080
  $WAIT_TIMEOUT = 0x00000102

  $h = [MutexUtils]::OpenMutex($MUTEX_ALL_ACCESS, $false, "MUTEX_NAME_HERE")
  if ($h -ne [IntPtr]::Zero) {
      $waitResult = [MutexUtils]::WaitForSingleObject($h, 0)
      Write-Host "WaitResult: 0x$('{0:X}' -f $waitResult)"
      [MutexUtils]::ReleaseMutex($h)
      [MutexUtils]::CloseHandle($h)
  } else {
      Write-Host "OpenMutex failed — last error: $([MutexUtils]::GetLastError())"
  }
  ```
- [ ] Run this after killing the game — check if `OpenMutex` succeeds and `WaitForSingleObject` returns `WAIT_ABANDONED (0x80)`.
- [ ] If abandoned: attempt `ReleaseMutex`, check return value. `false` = failure (expected due to ownership semantics).
- [ ] Try `CreateMutex` instead — acquire it in our process to prevent the game's own check from failing.

**Record result here**: `_P/Invoke approach: viable / not viable (TBD)_`

---

## Implementation Plan (Once Mutex Name Is Known)

Implementation will follow one of these paths based on investigation findings.

### Path A — Extended Wait + Retry in launch-game.md (Option B, low risk, implement first)

Regardless of other findings, improve the `/launch-game` skill immediately:

1. Replace the fixed 3-second sleep with polling: wait until the game process handle is fully gone.
2. Add an additional 5-second grace period after process exit.
3. Add retry logic: if the game exits with an "another instance" exit code, wait 5 more seconds and retry (up to 3 attempts).
4. Surface the retry count and wait time in the skill's output so the user can see what is happening.

**Files to update**:
- `.claude/commands/launch-game.md` — update Step 1 with polling + retry logic
- `.claude/commands/game-test.md` — ensure it uses the updated launch-game sequence

---

### Path B — BepInEx Preloader Patcher (Option A, if doorstop timing allows)

If investigation confirms the preloader runs before the mutex check:

1. Create `src/Runtime/Preloader/SecondInstancePatcher.cs` as a new class library targeting `net472` (BepInEx patcher target framework).
2. Implement `Initialize()` with `OpenMutex` + conditional `WaitForSingleObject` + `ReleaseMutex` / `CreateMutex` as described in ADR-014.
3. Register it as a BepInEx patcher by adding the correct `[BepInProcess]` and targeting `Diplomacy is Not an Option.exe`.
4. Build to `BepInEx/patchers/DINOForge.SecondInstancePatcher.dll`.
5. Update `src/Runtime/Plugin.cs` to log a warning if the patcher did not run (detectable via a static flag set by the patcher).
6. Add to the deploy step of `/pack-deploy` and `/test-swap`.

**Files to create/update**:
- `src/Runtime/Preloader/SecondInstancePatcher.cs` (new)
- `src/Runtime/Preloader/SecondInstancePatcher.csproj` (new, or add to existing Runtime project)
- `src/DINOForge.sln` — add new project if created separately
- `.claude/commands/launch-game.md` — document that patcher handles mutex
- `docs/troubleshooting.md` — document mutex bypass behavior

---

### Path C — Lockfile Deletion (Option C, if lockfile is confirmed)

If a lockfile is found in investigation:

1. Identify the full path of the lockfile.
2. Add a pre-launch step to `/launch-game` that deletes the file:
   ```powershell
   Remove-Item "C:\path\to\lockfile" -Force -ErrorAction SilentlyContinue
   ```
3. Verify this does not corrupt save data (test with a fresh save and confirm save integrity after deletion).
4. Document the lockfile path in `docs/troubleshooting.md`.

---

## Acceptance Criteria

- [ ] A second game instance can be launched within 15 seconds of the first instance being killed, without the "another instance" error dialog appearing.
- [ ] The `/launch-game` skill succeeds reliably across 5 consecutive kill-and-relaunch cycles in automated testing.
- [ ] The `/game-test` and `/test-swap` commands complete their full test cycle without requiring manual game restart.
- [ ] No save data corruption results from the bypass mechanism.
- [ ] If the bypass mechanism is the preloader patcher (Path B), it emits a structured log line confirming it ran and which mutex (if any) it released.
- [ ] Investigation results are recorded in the "Investigation Plan" checkboxes above before any implementation is merged.

---

## Notes

### On Win32 Mutex Ownership Semantics

`ReleaseMutex` requires that the calling thread own the mutex (i.e., it was the thread that called `CreateMutex` or `WaitForSingleObject` to acquire it). Calling `ReleaseMutex` from a non-owning thread returns `false` with `GetLastError() == ERROR_NOT_OWNER (0x120)`. This is a documented Win32 constraint and cannot be bypassed without kernel-level access. The practical implication: if the previous game process is still running and owns the mutex, we cannot release it from the outside. We can only wait for the process to die (at which point the mutex becomes abandoned) and then hope the OS cleanup happens quickly enough.

### On Abandoned Mutex Behavior

When a process that owns a mutex is terminated (not gracefully exiting), the mutex enters the **abandoned** state. From MSDN: "The system does not release an abandoned mutex automatically. The owning thread can release the mutex by calling `ReleaseMutex`, but since the owning thread is terminated, this cannot happen." In practice, the mutex is released as part of process handle cleanup, which occurs when all handles to the process are closed — including the handle held by the kernel's process table. On Windows, this happens asynchronously after `TerminateProcess` returns, not immediately. The timing varies.

### On the launch-game.md Current Workaround

The existing `/launch-game` skill already bypasses one major source of the problem: it launches the `.exe` directly rather than through the Steam launcher. This avoids Steam's own mutex (`Global\Valve_SteamMutex`). The remaining issue is the game's own single-instance mutex, which is independent of Steam.

### Agent Testing Policy

Per MEMORY.md: agents must never ask the user to "launch the game yourself", "click through the dialog", or "restart manually". All testing must be automated. This work item is high priority precisely because it blocks that mandate.

### Related Commands That Depend on This Being Solved

- `.claude/commands/game-test.md`
- `.claude/commands/test-swap.md`
- `.claude/commands/pack-deploy.md`
- `.claude/commands/launch-game.md` (direct)

---

## References

- ADR-014: Second Instance Detection and Bypass
- ADR-013: Duplicate Instance Detection & Win32 Mutex Bypass (earlier related ADR)
- `.claude/commands/launch-game.md`
- `src/Runtime/Plugin.cs`
- `src/Runtime/Bridge/KeyInputSystem.cs`
- Windows `ReleaseMutex` API: https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-releasemutex
- Windows abandoned mutexes: https://docs.microsoft.com/en-us/windows/win32/sync/mutex-objects
- BepInEx preloader patchers: https://docs.bepinex.dev/articles/dev_guide/preloader_patchers.html
- Sysinternals Handle.exe: https://docs.microsoft.com/en-us/sysinternals/downloads/handle
