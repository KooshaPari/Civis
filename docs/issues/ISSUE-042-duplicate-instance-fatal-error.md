# ISSUE-042: Duplicate Instance Fatal Error on Game Launch

**Status**: Reported
**Priority**: High
**Component**: Launcher (`launch-game` command, test harness)
**Related ADR**: ADR-013-duplicate-instance-detection-bypass.md

## Problem Description

When launching Diplomacy is Not an Option (DINO) via the `launch-game` PowerShell command or automated test launcher, the game fails with a fatal error:

```
Another instance is already running. Only one instance of the game is allowed.
```

This prevents:
- Running automated tests that need to launch and close the game multiple times
- Quick iteration during development (kill, fix, relaunch cycle)
- CI/CD pipelines that spawn game instances for integration tests
- Parallel test execution on shared machines

### Environment

- **Windows 11 Pro** (10.0.28020)
- **DINO version**: Latest (2026-03 branch)
- **Steam client**: Running (standard configuration)
- **Game path**: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option`
- **BepInEx**: 5.4.23.5 (standard GitHub release)
- **DINOForge**: Latest main branch

### Root Cause

Diplomacy is Not an Option uses a Win32 named mutex to enforce single-instance constraints. When the game process is killed (even via `Stop-Process -Force`), the kernel may not immediately release the mutex, causing subsequent launch attempts to fail during game initialization before BepInEx even loads.

See ADR-013 for full technical analysis.

## Steps to Reproduce

1. Open PowerShell
2. Run `.claude/commands/launch-game.md` (or equivalent):
   ```powershell
   Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
   Start-Sleep -Seconds 3
   Start-Process -FilePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe" -WorkingDirectory "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option" -ArgumentList "-bepinex"
   ```
3. Game window appears briefly, then closes with "Another instance is already running" error
4. No DINOForge bootstrap logs appear in `BepInEx/dinoforge_debug.log`

**Expected behavior**: Game should launch successfully and BepInEx + DINOForge should initialize

## Root Cause Analysis

### Win32 Mutex Semantics

1. Game calls `CreateMutex("Global\DiplomaXXX", ...)` on startup
2. If mutex already exists (from previous game instance), game exits with fatal error
3. When game process dies, kernel's mutex table is not immediately flushed
4. New instance tries to create the same mutex, finds it still in kernel table, fails
5. Sleep (3 seconds) is insufficient on most systems; mutex may persist for 10-30+ seconds

### Why `Stop-Process` Alone Fails

- `Stop-Process -Force` kills the process immediately
- Kernel-level resources (mutexes, file handles) held by the process are queued for release
- Release is asynchronous and may not complete within a few seconds
- New instance attempts to acquire mutex before kernel has released it from the old process

### Why Direct Exe Launch Still Fails

- Even calling `.exe` directly (bypassing Steam launcher) doesn't bypass the game's own mutex check
- DINO's init code (static initializer or Unity player startup) enforces the constraint
- This is game-level protection, not Steam-level

## Acceptance Criteria

The `launch-game` command is considered **fixed** when:

1. **Reliability**: Launching `launch-game.ps1` twice in succession (with 5-second gap) succeeds both times without "another instance" error
2. **Speed**: Total time from `launch-game` invocation to "Awake completed" in logs is &lt; 15 seconds (including any necessary waits)
3. **Automation-safe**: The fix works when called from a PowerShell script without user intervention
4. **No side effects**: Game runs normally after launch; no crashes, missing features, or gameplay issues
5. **Documentation**: Solution documented in `CLAUDE.md` or new `launch-game.md` version
6. **Fallback**: If primary method fails, has a documented fallback (e.g., "increase sleep" or "restart Steam")

## Implementation Plan

### Phase 1: Investigation (before code)

1. **Identify mutex name**
   - Use Sysinternals `Handle.exe` to enumerate Win32 mutexes
   - Launch game and capture all mutexes created
   - Determine if name is static or per-instance
   - Document findings in ADR-013

2. **Test P/Invoke approach**
   - Write PowerShell proof-of-concept using `OpenMutex` + `ReleaseMutex`
   - Test on real DINO instance
   - Determine if kernel allows non-owning process to release mutex
   - Document success/failure in ADR-013

3. **Empirical timing measurement**
   - Kill game and measure how long mutex persists in kernel table
   - Test multiple runs to find average persistence time
   - Determine safe minimum sleep duration

### Phase 2: Implementation

**Choose approach based on Phase 1 results:**

- **If P/Invoke works**: Integrate mutex release into `launch-game.ps1` (see SPEC-duplicate-instance-bypass.md)
- **If P/Invoke fails**: Use empirical sleep duration (10-15 seconds) instead
- **As fallback**: Implement copy-exe or other workaround

### Phase 3: Testing

1. **Unit test** (PowerShell):
   - Create test script that launches game 5 times with 5-second gaps
   - Verify all 5 launches succeed
   - Run on Windows 10, Windows 11, and GitHub Actions runner (if applicable)

2. **Integration test** (C#):
   - Add `LaunchGameTests.cs` to `src/Tests/`
   - Test game startup via `launch-game` command
   - Verify BepInEx initialization via log polling
   - Test cleanup (game exit) doesn't break subsequent launch

3. **Manual verification**:
   - Test on developer machine (this one)
   - Test with Steam running and not running
   - Test after Windows restart

### Phase 4: Documentation

1. Update `.claude/commands/launch-game.md`:
   - Document mutex release mechanism
   - Add `--force` flag if applicable
   - Add troubleshooting section

2. Update `CLAUDE.md`:
   - Add note about mutex behavior
   - Add launch-game command description

3. Add to `docs/troubleshooting.md`:
   - "Game won't launch: 'another instance' error"
   - Step-by-step resolution

### Phase 5: Release

1. Test on real Windows 11 system
2. Commit to main branch with updated `launch-game.md`
3. Close this issue when all tests pass

## Estimated Effort

- Phase 1 (Investigation): 1-2 hours
- Phase 2 (Implementation): 1 hour (once investigation complete)
- Phase 3 (Testing): 1-2 hours
- Phase 4 (Documentation): 30 minutes
- Phase 5 (Release): 15 minutes

**Total**: ~4-6 hours

## Blockers

None identified. This is a pure launcher/testing issue, not a gameplay or core framework blocker.

## Related Issues

- None currently open

## Related ADRs

- **ADR-013** — Architectural decision for mutex bypass strategy
- **SPEC-duplicate-instance-bypass.md** — Technical specification and code

## Links

- **Win32 Mutex docs**: https://docs.microsoft.com/en-us/windows/win32/sync/named-mutexes
- **Sysinternals Handle**: https://docs.microsoft.com/en-us/sysinternals/downloads/handle
- **Current launch-game command**: `.claude/commands/launch-game.md`
