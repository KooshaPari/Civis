# ADR-013: Duplicate Instance Detection & Win32 Mutex Bypass

**Status**: Proposed
**Date**: 2026-03-24
**Deciders**: kooshapari, DINOForge agents

## Context

Diplomacy is Not an Option (DINO), like many Steam games, enforces a single-instance constraint via Win32 named mutexes. This prevents multiple game clients from running simultaneously and avoids player confusion, save file corruption, and port conflicts.

### Current Problem

When DINOForge (`launch-game` command) or test harnesses attempt to launch the game repeatedly, they encounter a fatal error:

```
Another instance is already running. Only one instance of the game is allowed.
```

This occurs because:

1. **Steam maintains a mutex** — Steam holds a named mutex while the game is running (likely `Global\Valve_SteamMutex_*` or a game-specific equivalent).
2. **Previous instances may not fully release the mutex** — Even after:
   - User manually kills the game process (`Stop-Process`)
   - OS forcibly terminates the process
   - Game crashes or exits abnormally

   The mutex handle may still be held in the kernel's mutex table, preventing a new instance from acquiring it.

3. **Direct exe launch still respects the mutex** — Even if we bypass Steam's launcher and call the `.exe` directly with `-bepinex`, the game's own init code (likely in `Diplomacy is Not an Option.exe`'s static initializer or `Unity.Player.dll`) calls `CreateMutex` and checks if it already exists. If it does, the game exits with the "another instance" error and never reaches BepInEx bootstrap.

### How DINO Detects Duplicate Instances

DINO's detection mechanism is likely one of:

- **Named mutex** — Windows API `CreateMutex(mutexName)` check in game init code or Unity player startup
- **TCP port lock** — Game tries to bind to a port; if it fails, another instance is assumed
- **File lock** — Temp file in `%LOCALAPPDATA%` or `%TEMP%` that the game exclusively locks

Most Steam games use named mutexes. The mutex is created at game startup in a global namespace accessible to all processes on the system.

### Current Workaround Limitations

The `launch-game.md` command currently:

```powershell
Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3
Start-Process -FilePath "...\Diplomacy is Not an Option.exe" ...
```

**Why this fails**: The OS may not immediately release kernel mutexes held by a killed process. Even after `Stop-Process` and a sleep, the mutex may still exist in the kernel's mutex table, and the new instance fails to acquire it.

### Investigation Needed

Before committing to a solution, we must determine:

1. **What is the exact mutex name?**
   - Is it `Global\Valve_SteamMutex_*`?
   - Is it game-specific, e.g., `Global\DiplomaIsNotAnOption_*`?
   - How to enumerate all named mutexes on the system to find it?

2. **Can we release it programmatically?**
   - Win32 API `OpenMutex` + `ReleaseMutex` in PowerShell (via P/Invoke)?
   - Does the kernel allow non-owning processes to release mutexes, or does it validate ownership?

3. **Is it a file lock instead?**
   - Where is the lock file? (`%TEMP%`, `%LOCALAPPDATA%`, `BepInEx/`, game root?)
   - Can we simply delete it?

4. **Does the mutex have a timeout or auto-cleanup?**
   - Some games release the mutex after a grace period (e.g., 10 seconds) if no heartbeat is detected
   - If so, we might just need a longer sleep

5. **Will bypassing the mutex cause gameplay issues?**
   - Could two game instances interfere with each other if both acquire the mutex?
   - Do they share configuration, save files, or other resources that would corrupt?

## Decision Options

### Option 1: Direct Win32 Mutex Release (PowerShell P/Invoke)

**How it works:**
1. Enumerate all named mutexes via Win32 API `CreateToolhelp32Snapshot` or `NtQuerySystemInformation`
2. Find the DINO/Steam mutex by name pattern
3. Open it with `OpenMutex` from a PowerShell script
4. Call `ReleaseMutex` to force-release
5. Attempt game launch

**Pros:**
- Minimal dependencies (pure Win32 P/Invoke)
- Works offline, no external tools needed
- Can be packaged in PowerShell directly

**Cons:**
- Kernel may reject non-owning process trying to release a mutex
- Enumerating named mutexes is non-trivial (not exposed in standard .NET)
- Requires understanding Windows kernel mutex ownership semantics
- May require admin privileges

### Option 2: Use Steam's `-applaunch` with Single-Instance Override Flags

**How it works:**
1. Call `steam://run/411400 -single_instance` (if flag exists) or similar
2. Steam may have a known flag to force bypass instance checking
3. If not, submit to Valve support to confirm no such flag exists

**Pros:**
- Avoids mutexes entirely if Steam has a documented override
- Delegates to Steam's own multi-instance handling

**Cons:**
- May not exist or may be undocumented
- Requires Steam to be running
- Slower than direct exe launch

### Option 3: Rename/Copy Exe to Bypass Steam Association

**How it works:**
1. Copy `Diplomacy is Not an Option.exe` to `DINO_Launcher_Temp.exe`
2. Launch the copy instead
3. Copy has a different process name, potentially different mutex name
4. On exit, delete the copy

**Pros:**
- No P/Invoke needed
- Bypasses Steam's launcher entirely

**Cons:**
- Game may detect copy via file path or signature and reject it
- Creates temp files that could accumulate
- Fragile; depends on game not validating exe integrity

### Option 4: Temporarily Kill Steam Process (Nuclear Option)

**How it works:**
1. Kill `steam.exe` (force via `taskkill /F`)
2. Wait for kernel to release all Steam-held mutexes
3. Launch game directly
4. Optionally restart Steam after game closes

**Pros:**
- Guaranteed to release any Steam mutex

**Cons:**
- Kills the Steam client, losing overlay, achievements, etc.
- Intrusive and hostile to user's session
- May interfere with other Steam games or Steam features

### Option 5: Increase Sleep Duration (Empirical Brute Force)

**How it works:**
1. `Stop-Process`
2. Sleep for 10-30 seconds instead of 3
3. Retry launch
4. If still fails, repeat with longer sleep

**Pros:**
- Zero code changes
- No P/Invoke or dependencies

**Cons:**
- Slow and unreliable
- Kernel may never release the mutex on some systems
- Doesn't scale (what if mutex is held indefinitely?)

## Recommended Decision

**Hybrid approach: Option 1 + Option 5 (with fallback to Option 3)**

### Implementation Strategy

1. **First attempt (safest):**
   - Kill game process
   - Sleep 3 seconds
   - Attempt mutex release via P/Invoke (with graceful failure if not possible)
   - Sleep 2 more seconds
   - Launch game

2. **If that fails, escalate to Option 5:**
   - Increase sleep to 10 seconds
   - Retry launch

3. **Last resort (if both fail):**
   - Warn user, then proceed with Option 3 (copy exe)

### Why This Approach

- **Prioritizes correctness** — Direct mutex release is the most robust if it works
- **Has graceful fallbacks** — Doesn't fail catastrophically if P/Invoke fails
- **Empirically proven** — Sleep-based approach is known to work on some systems
- **Minimal friction** — Doesn't kill Steam or rename exes unless absolutely necessary
- **Testable** — Each step can be verified independently

### Investigation Phase

Before writing code, conduct:

1. **Mutex name discovery** — Use Sysinternals `Handle.exe` or PowerShell WMI to enumerate mutexes and find the exact name
2. **P/Invoke proof-of-concept** — Write a small PowerShell script to test `OpenMutex` + `ReleaseMutex` on the discovered name
3. **Ownership validation** — Test whether a non-owning process can release the mutex (it likely can't; this determines if Option 1 is viable)
4. **Kernel behavior study** — Measure how long the kernel holds a mutex after the owning process dies on this system
5. **Test on real game** — Verify mutex name doesn't change between game versions or configurations

## Consequences

### If Option 1 Works (Mutex Release)

- `launch-game` becomes highly reliable
- No dependency on timing or race conditions
- Enables fully automated testing workflows
- Establishes pattern for other Win32 issues (if they arise)

### If Option 1 Doesn't Work (Kernel Ownership)

- Must fall back to Option 5 (sleep) or Option 3 (copy exe)
- Sleep adds latency to test workflows (acceptable if &lt; 15 seconds)
- Copy-exe approach is fragile but functional as last resort

### Long-Term

- Document the mutex name and workaround in `docs/troubleshooting.md`
- Add `--force-launch` flag to `launch-game` that skips mutex check entirely
- Consider wrapping this into a standalone `release-game-mutex` PowerShell function for manual use
- If Steam provides an official workaround, update this ADR and switch to it

## Links to Investigation

- Windows named mutexes: https://docs.microsoft.com/en-us/windows/win32/sync/named-mutexes
- Sysinternals Handle.exe: https://docs.microsoft.com/en-us/sysinternals/downloads/handle
- PowerShell P/Invoke patterns: https://github.com/powershell/psl-teamcity/
