# Concurrent Game Instance Launcher

**Status**: Implemented (2026-03-30)
**Purpose**: Launch two independent game instances for parallel testing without disk copying
**Location**: `scripts/game/New-TempGameInstance.ps1` + `scripts/game/Launch-ConcurrentInstances.ps1`

## Problem

Previously, testing required either:
1. **Full directory copy** (12GB) → 3-5 minute setup, then cleanup
2. **Sequential launches** → test one, kill it, test the other

This blocked parallel/concurrent testing workflows and consumed excessive CI disk space.

## Solution

**Lightweight symlink-based temp instance** (~100MB, <15s creation):

- Main instance: Original install at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
- Temp instance: Minimal directory with:
  - Hardlinked executable (50MB, shared inode)
  - Symlinked BepInEx, Assets, Data (read-only from main)
  - Independent LocalAppData (saves, logs, configs)

Both run simultaneously with no mutex conflicts (boot.config: `single-instance=0`).

## Architecture

```
Main Install (12GB, read-only via symlinks):
├─ Diplomacy is Not an Option.exe
├─ Diplomacy is Not an Option_Data/
├─ BepInEx/
├─ StreamingAssets/
└─ LocalAppData/ (main instance writes here)

Temp Instance (100MB ephemeral):
├─ Diplomacy is Not an Option.exe      ← hardlink (shared inode)
├─ Diplomacy is Not an Option_Data/    ← symlink → main
├─ BepInEx/                            ← symlink → main
├─ StreamingAssets/                    ← symlink → main
└─ LocalAppData/                       ← isolated copy
```

## Usage

### Basic Launch

```powershell
# Create and launch both instances
$instances = & "scripts/game/Launch-ConcurrentInstances.ps1"

# Both running. Check logs:
Write-Host "Main logs: $(Split-Path (Get-Item 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log').FullName)"
Write-Host "Temp logs: $($instances.temp.DebugLogPath)"

# When done, cleanup
Remove-Item $instances.temp.RootDir -Recurse -Force
```

### Manual Temp Instance Creation

```powershell
# Just create temp instance (don't launch yet)
$tempInstance = & "scripts/game/New-TempGameInstance.ps1" -Verbose

# Use it however you want
Write-Host $tempInstance.GameExePath
Write-Host $tempInstance.DebugLogPath

# Cleanup
Remove-Item $tempInstance.RootDir -Recurse -Force
```

## Boot Config Fix

**Critical**: Both instances use `single-instance=0` in boot.config.

This disables Unity's native single-instance enforcement (previously set to empty string, which Unity treated as truthy).

The symlink approach works because:
1. Both instances have independent `boot.config` (they're in different directories)
2. Unity reads boot.config from its current working directory
3. So even though they symlink to the same physical files, Unity sees different configs per CWD

## Performance

| Operation | Time |
|-----------|------|
| Create temp instance (symlinks) | <5 seconds |
| Launch main instance | ~8 seconds to "Awake completed" |
| Launch temp instance | ~8 seconds to "Awake completed" |
| Both ready | ~15-20 seconds total |
| Cleanup | <1 second (single dir delete) |

**Disk usage**: ~100MB vs 12GB for full copy = 120x reduction

## CI Integration

### Before (Full Copy Approach)

```yaml
- name: Copy game to test directory
  run: |
    Copy-Item "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option" `
              "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST" `
              -Recurse -Force
  # Takes 3-5 minutes, requires 12GB free space
```

### After (Symlink Approach)

```yaml
- name: Launch concurrent instances for parallel testing
  run: |
    $instances = & "scripts/game/Launch-ConcurrentInstances.ps1"
    # Use both instances simultaneously for testing
    # Takes <20 seconds, requires <200MB free space
    Remove-Item $instances.temp.RootDir -Recurse -Force
```

## Limitations & Notes

1. **Symlinks are read-only** — BepInEx plugins, assets must come from main install
   - ✓ Works great for testing mod packs (content-only changes)
   - ✗ Won't work if testing BepInEx plugin changes (need to relink)
   - Workaround: If testing Runtime changes, use full copy approach with `/launch-test-instance`

2. **Independent LocalAppData** — Saves won't sync between instances
   - By design! Prevents save corruption and testing interference

3. **Windows-specific** — Uses `mklink` for hardlinks/symlinks
   - WSL2: Works (NTFS), doesn't work on ext4 volumes
   - GitHub Actions: `windows-latest` runner only

4. **Cleanup is manual** — Script outputs cleanup command
   - Temp instance directory won't auto-clean
   - Use: `Remove-Item $instances.temp.RootDir -Recurse -Force`

## Implementation Details

### New-TempGameInstance.ps1

Creates a single temp instance:

```powershell
$tempInstance = & "scripts/game/New-TempGameInstance.ps1" `
    -TempDir "$env:TEMP\DINOForge\instances" `
    -GameExePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe" `
    -Verbose

# Returns [PSCustomObject] with:
# - InstanceId: UUID string (e.g., "abc12345")
# - RootDir: Full path to temp instance
# - GameExePath: Path to executable in temp dir
# - WorkingDirectory: Where to launch from
# - DebugLogPath: Path to dinoforge_debug.log
# - Size_MB: ~50 (just the hardlinked exe)
```

### Launch-ConcurrentInstances.ps1

Manages both main + temp:

```powershell
$instances = & "scripts/game/Launch-ConcurrentInstances.ps1" `
    -WaitForBoth $true `
    -HideTemp $false

# Returns @{main, temp} each with:
# - ProcessId
# - GameExePath
# - DebugLogPath
# - RootDir (temp only)
```

## Testing

To verify both instances work together:

```powershell
# Launch both
$instances = & "scripts/game/Launch-ConcurrentInstances.ps1"

# Wait for both to initialize
Start-Sleep -Seconds 20

# Check both are running
Get-Process | Where-Object { $_.Name -like "*Diplomacy*" }
# Should show 2 processes

# Verify both wrote logs
$mainLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
$tempLog = $instances.temp.DebugLogPath

Get-Content $mainLog -Tail 5
Get-Content $tempLog -Tail 5
# Both should contain "Awake completed"

# Verify independence (one can restart, other keeps running)
Get-Process -Name "Diplomacy is Not an Option" | Select-Object -First 1 | Stop-Process -Force
Start-Sleep -Seconds 3
Get-Process | Where-Object { $_.Name -like "*Diplomacy*" }
# Should still show 1 process (the other instance)

# Cleanup
Remove-Item $instances.temp.RootDir -Recurse -Force
```

## Future Enhancements

1. **Hidden desktop launch** — `Launch-ConcurrentInstances.ps1 -HideTemp` for fully invisible CI testing
2. **Auto-cleanup** — Temp instance cleanup on script exit (defer deletion until PS session closes)
3. **Instance pooling** — Keep N pre-created instances, reuse instead of create each run
4. **Cross-branch testing** — Create instances with different BepInEx versions (symlink to different main installs)

## References

- **Boot config fix**: `Diplomacy is Not an Option_Data\boot.config` (`single-instance=0`)
- **ADR-018**: Second Instance Detection and Bypass (documents mutex resolution)
- **Launch command**: `.claude/commands/launch-game.md`
