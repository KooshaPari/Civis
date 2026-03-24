# SPEC-duplicate-instance-bypass: Win32 Mutex Detection & Release Mechanism

**Status**: Draft
**Last Updated**: 2026-03-24
**Related Issues**: ISSUE-042, ADR-013
**Implementation Track**: PowerShell utilities + launcher integration

## Overview

This specification defines the technical mechanism for detecting and releasing Win32 named mutexes that prevent DINO from launching multiple instances.

## Goals

1. **Enable repeated game launches** — Support quick dev iteration and automated testing
2. **Minimal dependencies** — Use only Win32 APIs (P/Invoke) and PowerShell
3. **Graceful fallback** — If P/Invoke approach fails, fall back to increased sleep
4. **Platform-agnostic** — Should work on Windows 10, 11, and future versions
5. **Testable** — Each component can be tested independently

## Constraints

- Must run on Windows 11 Pro (primary dev environment)
- Must not require external tools (no `Handle.exe` or `Sysinternals` in production)
- Must work with Steam running and not running
- Must not interfere with normal game operation after launch
- Must not require admin privileges (if possible)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  launch-game.ps1 (PowerShell Command)                   │
├─────────────────────────────────────────────────────────┤
│  1. Kill existing game process                          │
│  2. Call Release-GameMutex (Phase 1 approach)          │
│  3. Fallback: Extended sleep (Phase 2 approach)        │
│  4. Launch game exe with -bepinex flag                 │
│  5. Poll debug log for "Awake completed"               │
└─────────────────────────────────────────────────────────┘
         │                           │
         ▼                           ▼
┌──────────────────────┐  ┌──────────────────────┐
│ Release-GameMutex    │  │ Get-GameMutexInfo    │
│ (Attempts Release)   │  │ (Discovery & Enum)   │
├──────────────────────┤  ├──────────────────────┤
│ • OpenMutex P/Invoke │  │ • Enumerate handles  │
│ • ReleaseMutex      │  │ • Identify mutex     │
│ • Error recovery     │  │ • Return name/handle │
└──────────────────────┘  └──────────────────────┘
```

## Component Specifications

### 1. Get-GameMutexInfo (Discovery Function)

**Purpose**: Identify the DINO game mutex name and current owner process

**Input**: None (or optional mutex name pattern)

**Output**: `[PSCustomObject]` with properties:
- `MutexName` (string) — Full name of the mutex, e.g., `Global\Valve_SteamMutex_4199_1234`
- `Exists` (bool) — Whether the mutex currently exists
- `OwnerProcess` (int) — PID of process holding mutex (if exists)
- `OwnerName` (string) — Name of process holding mutex (e.g., `Diplomacy is Not an Option`)

**Algorithm**:

```powershell
function Get-GameMutexInfo {
    [CmdletBinding()]
    param(
        [string]$MutexPattern = "Valve_SteamMutex_4199"  # DINO app ID
    )

    # Step 1: Enumerate all named mutexes via Win32 API
    # (Use kernel handle enumeration or WMI)

    # Step 2: Filter for pattern (e.g., "Valve_SteamMutex_4199*")

    # Step 3: For each match, attempt to open and get owner info
    # via OpenMutex + QueryMutexInformation

    # Step 4: Return object with details

    return [PSCustomObject]@{
        MutexName = $mutexName
        Exists = $true
        OwnerProcess = $ownerPid
        OwnerName = $ownerName
    }
}
```

**Implementation Notes**:
- Enumerating mutexes is non-trivial in PowerShell (not exposed in standard .NET)
- Fallback: Use `Get-Process` + check if "Diplomacy is Not an Option" is running, then assume mutex exists
- Alternative: Use Sysinternals Handle.exe (slower, but works as PoC)

### 2. Release-GameMutex (Mutex Release Function)

**Purpose**: Attempt to release DINO game mutex from kernel

**Input**:
- `$MutexName` (string) — Name of mutex to release (from Get-GameMutexInfo)
- `$TimeoutMs` (int) — How long to wait for release (default 5000ms)

**Output**: `[PSCustomObject]` with properties:
- `Success` (bool) — Whether release succeeded
- `Message` (string) — Details (e.g., "Mutex released" or "Kernel rejected release")
- `Elapsed` (TimeSpan) — How long operation took

**Algorithm**:

```powershell
function Release-GameMutex {
    [CmdletBinding()]
    param(
        [string]$MutexName,
        [int]$TimeoutMs = 5000
    )

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

    try {
        # Step 1: Define Win32 P/Invoke signatures
        $OpenMutexSig = @"
[DllImport("kernel32.dll", SetLastError = true)]
public static extern IntPtr OpenMutex(
    uint desiredAccess,
    bool inheritHandle,
    string name
);
"@

        $ReleaseMutexSig = @"
[DllImport("kernel32.dll", SetLastError = true)]
public static extern bool ReleaseMutex(IntPtr handle);
"@

        $CloseSig = @"
[DllImport("kernel32.dll", SetLastError = true)]
public static extern bool CloseHandle(IntPtr handle);
"@

        # Step 2: Compile Win32 class
        $Kernel32 = Add-Type -MemberDefinition @($OpenMutexSig, $ReleaseMutexSig, $CloseSig) `
            -Name "Kernel32" -Namespace "Win32" -PassThru

        # Step 3: Attempt to open mutex
        # SYNCHRONIZE (0x00100000) = right to acquire/release mutex
        $handle = $Kernel32::OpenMutex(0x00100000, $false, $MutexName)

        if ($handle -eq [IntPtr]::Zero) {
            return [PSCustomObject]@{
                Success = $false
                Message = "Mutex not found or cannot open (may not exist yet)"
                Elapsed = $stopwatch.Elapsed
            }
        }

        # Step 4: Release mutex
        $result = $Kernel32::ReleaseMutex($handle)

        # Step 5: Close handle
        $Kernel32::CloseHandle($handle) | Out-Null

        $stopwatch.Stop()

        if ($result) {
            return [PSCustomObject]@{
                Success = $true
                Message = "Mutex released successfully"
                Elapsed = $stopwatch.Elapsed
            }
        } else {
            $lastError = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
            return [PSCustomObject]@{
                Success = $false
                Message = "Kernel rejected release (error: $lastError). Mutex may have different owner."
                Elapsed = $stopwatch.Elapsed
            }
        }
    }
    catch {
        $stopwatch.Stop()
        return [PSCustomObject]@{
            Success = $false
            Message = "Exception during release: $_"
            Elapsed = $stopwatch.Elapsed
        }
    }
}
```

**Implementation Notes**:
- `OpenMutex` with `SYNCHRONIZE` access (0x00100000) may fail if kernel enforces ownership
- If ownership check prevents release, `result` will be `$false` and `GetLastWin32Error()` will be `ERROR_ACCESS_DENIED` (5)
- In that case, Phase 1 fails gracefully and we fall back to Phase 2 (increased sleep)
- Timeout allows caller to bail out if P/Invoke hangs

### 3. launch-game.ps1 (Orchestrator)

**Purpose**: Main launcher that kills game, releases mutex, and launches fresh instance

**Integration Points**:
- Calls `Get-GameMutexInfo` to identify mutex
- Calls `Release-GameMutex` to release it
- Falls back to sleep if P/Invoke fails
- Polls debug log for successful launch

**Pseudocode**:

```powershell
function Invoke-LaunchGame {
    param(
        [string]$GamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
        [int]$MaxRetries = 3,
        [switch]$WaitForWorld
    )

    # Phase 0: Kill existing process
    Write-Host "Stopping existing game instance..."
    Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
    Stop-Process -Name "UnityCrashHandler64" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1

    # Phase 1: Attempt P/Invoke mutex release
    Write-Host "Attempting mutex release..."
    $mutexInfo = Get-GameMutexInfo -MutexPattern "Valve_SteamMutex_4199"

    if ($mutexInfo.Exists) {
        $releaseResult = Release-GameMutex -MutexName $mutexInfo.MutexName
        if ($releaseResult.Success) {
            Write-Host "✓ Mutex released in $($releaseResult.Elapsed.TotalMilliseconds)ms"
        } else {
            Write-Host "⚠ Mutex release failed: $($releaseResult.Message)"
            Write-Host "  Falling back to extended sleep..."
            Start-Sleep -Seconds 10  # Phase 2 fallback
        }
    } else {
        Write-Host "ℹ Mutex not found (game not running)"
    }

    # Phase 2: Additional safety sleep
    Start-Sleep -Seconds 2

    # Phase 3: Clear debug log
    Clear-Content "$GamePath\BepInEx\dinoforge_debug.log" -ErrorAction SilentlyContinue

    # Phase 4: Launch game
    Write-Host "Launching game..."
    Start-Process `
        -FilePath "$GamePath\Diplomacy is Not an Option.exe" `
        -WorkingDirectory $GamePath `
        -ArgumentList "-bepinex"

    # Phase 5: Poll for successful launch
    Write-Host "Waiting for DINOForge initialization..."
    $timeout = 30
    $elapsed = 0
    $logFile = "$GamePath\BepInEx\dinoforge_debug.log"

    while ($elapsed -lt $timeout) {
        Start-Sleep -Seconds 2
        $elapsed += 2

        if (Test-Path $logFile) {
            $log = Get-Content $logFile -ErrorAction SilentlyContinue
            if ($log -match "Awake completed") {
                Write-Host "✓ DINOForge loaded successfully"
                return $true
            }
        }
    }

    Write-Host "✗ Timeout waiting for initialization"
    return $false
}
```

**Command-line integration** (in `.claude/commands/launch-game.md`):

```powershell
# Load helper functions
. "./src/Tools/Cli/launch-game-helpers.ps1"

# Invoke main launcher
$success = Invoke-LaunchGame @args
exit ($success ? 0 : 1)
```

## Error Handling

| Scenario | Handling |
|----------|----------|
| Mutex not found | Log info, proceed with launch (mutex may not exist yet) |
| OpenMutex fails | Log warning, fall back to Phase 2 (sleep) |
| ReleaseMutex returns false | Log warning (kernel likely rejected), fall back to sleep |
| P/Invoke compilation fails | Log error, fall back to sleep |
| Game launch fails (exe not found) | Log error, exit with code 1 |
| Debug log never appears | Log warning, exit with code 1 (game didn't initialize) |

## Testing Strategy

### Unit Tests (PowerShell)

Create `tests/unit/Test-LaunchGame.ps1`:

```powershell
Describe "Get-GameMutexInfo" {
    It "Returns object with expected properties" {
        $result = Get-GameMutexInfo
        $result.MutexName | Should -BeOfType [string]
        $result.Exists | Should -BeOfType [bool]
    }

    It "Returns 'Exists' = true when game is running" {
        # Start game manually first
        $result = Get-GameMutexInfo
        $result.Exists | Should -Be $true
    }

    It "Returns 'Exists' = false when game is not running" {
        Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 3
        $result = Get-GameMutexInfo
        $result.Exists | Should -Be $false
    }
}

Describe "Release-GameMutex" {
    It "Returns object with Success property" {
        # Assumes game is running
        $result = Release-GameMutex -MutexName "Global\Valve_SteamMutex_4199_12345"
        $result.Success | Should -BeOfType [bool]
    }

    It "Reports success when mutex exists and can be released" {
        # Expected to fail if kernel enforces ownership, but we can test the API
        $result = Release-GameMutex -MutexName "Global\Valve_SteamMutex_4199_12345"
        $result.Message | Should -Not -BeNullOrEmpty
    }
}

Describe "Invoke-LaunchGame" {
    It "Launches game and returns true when successful" {
        $result = Invoke-LaunchGame
        $result | Should -Be $true
    }

    It "Succeeds twice in succession with 5-second gap" {
        # Key regression test for mutex issue
        $result1 = Invoke-LaunchGame
        $result1 | Should -Be $true

        Start-Sleep -Seconds 5

        $result2 = Invoke-LaunchGame
        $result2 | Should -Be $true
    }
}
```

### Integration Tests (C#)

Add to `src/Tests/LauncherIntegrationTests.cs`:

```csharp
[TestFixture]
public class LauncherIntegrationTests
{
    [Test]
    [Explicit("Requires game to be installed and available")]
    public async Task LaunchGame_WithExistingInstance_Succeeds()
    {
        // Start first instance
        var result1 = await _launcher.LaunchAsync();
        Assert.That(result1.Success, Is.True);
        Assert.That(result1.ProcessId, Is.GreaterThan(0));

        // Wait 3 seconds
        await Task.Delay(TimeSpan.FromSeconds(3));

        // Start second instance (should succeed despite first being fresh)
        var result2 = await _launcher.LaunchAsync();
        Assert.That(result2.Success, Is.True);
        Assert.That(result2.ProcessId, Is.GreaterThan(0));
        Assert.That(result2.ProcessId, Is.Not.EqualTo(result1.ProcessId));

        // Cleanup
        _launcher.KillGameProcess();
    }

    [Test]
    [Explicit("Requires game to be installed")]
    public async Task LaunchGame_Logs_AweCompleted()
    {
        var result = await _launcher.LaunchAsync();
        Assert.That(result.Success, Is.True);

        // Poll debug log
        var log = await Task.Delay(TimeSpan.FromSeconds(2))
            .ContinueWith(_ => File.ReadAllText(DebugLogPath));

        Assert.That(log, Does.Contain("Awake completed"));
    }
}
```

## Acceptance Criteria

1. **Can identify DINO mutex** — `Get-GameMutexInfo` successfully returns mutex name when game is running
2. **P/Invoke compiles** — `Release-GameMutex` compiles without errors
3. **Graceful fallback** — If P/Invoke fails, no exception is thrown; falls back to sleep
4. **Launch succeeds twice** — Running `Invoke-LaunchGame` twice with 5-second gap succeeds both times
5. **No side effects** — Game runs normally after launch; no crashes or missing features
6. **Documentation complete** — Updated `.claude/commands/launch-game.md` with new behavior
7. **Tests pass** — All unit and integration tests pass on Windows 11 Pro

## Phase Rollout

### Phase 1 (Investigation — 1-2 hours)

- [ ] Use Sysinternals Handle.exe to identify mutex name
- [ ] Document exact mutex name (e.g., `Global\Valve_SteamMutex_4199_...`)
- [ ] Test P/Invoke `OpenMutex` + `ReleaseMutex` in standalone PowerShell script
- [ ] Determine if kernel allows non-owning process to release (likely: no)
- [ ] Measure actual mutex persistence time after process kill

### Phase 2 (Implementation — 1 hour)

- [ ] Implement `Get-GameMutexInfo` with P/Invoke enumeration (or fallback to process check)
- [ ] Implement `Release-GameMutex` with error handling
- [ ] Implement `Invoke-LaunchGame` orchestrator
- [ ] Integrate into `.claude/commands/launch-game.md`

### Phase 3 (Testing — 1-2 hours)

- [ ] Write and run unit tests for each function
- [ ] Manual test: consecutive launches with 5-second gap
- [ ] Manual test: game runs normally after launch
- [ ] Integration test via C# launcher

### Phase 4 (Documentation — 30 minutes)

- [ ] Update `.claude/commands/launch-game.md`
- [ ] Add troubleshooting section to `docs/troubleshooting.md`
- [ ] Update `CLAUDE.md` with launcher notes

### Phase 5 (Release — 15 minutes)

- [ ] Final testing on Windows 11 Pro
- [ ] Commit and push
- [ ] Close ISSUE-042

## References

- **ADR-013** — Architectural decision
- **ISSUE-042** — Original problem report
- **Windows Mutex Docs**: https://docs.microsoft.com/en-us/windows/win32/sync/named-mutexes
- **P/Invoke patterns**: https://github.com/powershell/psl-teamcity
- **Sysinternals Handle**: https://docs.microsoft.com/en-us/sysinternals/downloads/handle
