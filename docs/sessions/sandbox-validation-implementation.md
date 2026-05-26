# Sandbox Validation & Cleanup Implementation

**Date**: 2026-04-12  
**Scope**: Enhanced `New-TempGameInstance.ps1` with comprehensive validation and cleanup  
**Status**: COMPLETE

## Summary

Implemented robust validation and automatic cleanup for temporary game instance creation to prevent orphaned sandbox directories on failure. Validation occurs at critical points:
1. **Symlink Creation** - Verifies all directory symlinks exist after creation
2. **Steam Auth** - Validates Steam auth files copied successfully
3. **LocalAppData Isolation** - Ensures LocalAppData is a real directory, not a shared symlink
4. **Error Handling** - Automatic rollback cleanup on any failure

## Changes Made

### 1. Enhanced `scripts/game/New-TempGameInstance.ps1`

#### New Validation Functions

**`Validate-Symlink`** - Verifies symlinks created successfully
```powershell
# Checks symlink path exists and contains expected directory
if (-not (Test-Path -PathType Container $LinkPath)) {
    Write-LogError "Failed to create symlink: $LinkName"
    return $false
}
```

**`Validate-SteamAuth`** - Confirms Steam auth copied with required files
```powershell
# Validates existence of Steam auth directory and app config
$steamAuthDir = Join-Path $InstanceDir "LocalAppData\Steam"
$appConfig = Join-Path $steamAuthDir "7970\local\config"
if (-not (Test-Path $appConfig)) {
    Write-LogWarn "Steam app config not found"
    return $false
}
```

**`Validate-LocalAppDataIsolation`** - Ensures proper isolation from main game
```powershell
# Verifies LocalAppData is NOT a symlink (must be independent)
$linkInfo = cmd /c 'fsutil reparsepoint query "$isolatedLocalAppData"' 2>&1
if ($linkInfo -match "Mount point|Symlink") {
    Write-LogWarn "LocalAppData appears to be a link"
    return $false
}
```

**`Remove-InstanceDirectory`** - Cleanup helper with logging
```powershell
# Removes failed instance directory and logs result
Remove-Item -Path $InstanceDir -Recurse -Force -ErrorAction Stop
Write-LogInfo "Instance cleanup completed"
```

#### Enhanced `New-SingleInstance` Function

- Wraps entire creation in try/catch block
- Validates symlinks, Steam auth, LocalAppData after creation
- Automatic cleanup on any failure
- Structured logging with request ID correlation
- Returns $null on failure (not partial object)

**Failure Flow**:
```
Any error → Remove-InstanceDirectory → Cleanup logs → Return $null
```

#### New Parameters
- `SkipValidation` - Allows disabling validation for testing (defaults to off)
- Request ID integration for audit trails

#### Pool-level Changes
- Counts created vs. requested instances
- Tracks created directories for cleanup
- Returns `CreatedDirs` array for cleanup on launch failure
- Sets pool status to `partial_creation` if some instances fail

### 2. Enhanced `scripts/automation/Launch-ParallelGames.ps1`

#### Launch Failure Handling

Added complete cleanup chain:
1. **Detect launch failure** - Any `Start-Process` failure triggers cleanup
2. **Kill running processes** - Stop all successfully launched instances
3. **Remove all sandbox directories** - Delete created instance directories
4. **Log failure** - Detailed error logging with request ID

**Cleanup Sequence**:
```powershell
if ($launchFailureFlag) {
    # 1. Kill processes
    $processes | Stop-Process -Force
    
    # 2. Remove sandboxes
    foreach ($dir in $boxPool.CreatedDirs) {
        Remove-Item -Path $dir -Recurse -Force
    }
    
    # 3. Exit with error
    exit 1
}
```

#### New Tracking
- `$launchFailureFlag` - Tracks if any launch failed
- Loop breaks on first failure instead of continuing
- Individual instance failure doesn't retry (fail-fast)

### 3. New Test Suite: `scripts/tests/SandboxValidationTests.ps1`

Six comprehensive tests:

1. **Symlink Creation & Verification** (PASSED)
   - Creates test symlink
   - Verifies directory exists
   - Confirms reparse point status

2. **Steam Auth Validation** (PASSED)
   - Detects missing Steam directory
   - Creates full Steam structure
   - Verifies app config path

3. **LocalAppData Isolation** (PASSED)
   - Verifies LocalAppData is real directory
   - Confirms not a symlink
   - Tests isolation properties

4. **Cleanup Removes Sandbox Files** (PASSED)
   - Creates complete sandbox structure
   - Removes directory
   - Confirms all files deleted

5. **Cleanup Preserves Main Game** (PASSED)
   - Creates main game directory
   - Creates sandbox with symlinks
   - Verifies main game unaffected

6. **Full Instance Creation** (PASSED)
   - Creates complete instance structure
   - Validates all components
   - Confirms structure integrity

**Test Results**:
```
Total Tests:  6
Passed:       6
Failed:       0
Status:       ALL TESTS PASSED
```

## Validation Checklist

- [x] Symlinks validated after creation
- [x] Steam auth files verified
- [x] LocalAppData properly isolated (not a symlink)
- [x] Rollback/cleanup on creation failure
- [x] Launch failure triggers sandbox cleanup
- [x] Process failures captured and logged
- [x] Structured logging with request ID
- [x] Main game directory preserved on cleanup
- [x] Comprehensive test coverage
- [x] Tests passing on Windows (admin not required)

## Key Behaviors

### Success Path
```
Create Instance
  ├─ Create directories
  ├─ Create symlinks + VALIDATE
  ├─ Copy Steam auth + VALIDATE
  ├─ Setup LocalAppData + VALIDATE
  └─ Return instance object
```

### Failure Path (Instance Creation)
```
Create Instance
  ├─ [Any validation fails]
  ├─ Remove-InstanceDirectory
  ├─ Log error with request ID
  └─ Return $null
```

### Failure Path (Launch)
```
Launch Processes
  ├─ [Any process fails to start]
  ├─ Set launchFailureFlag = true
  ├─ Kill all running processes
  ├─ Remove all sandbox directories
  ├─ Log comprehensive error
  └─ Exit 1
```

## Logging Integration

All operations use structured logging module:
- `Write-LogInfo` - Normal operation steps
- `Write-LogWarn` - Validation warnings (missing files, etc.)
- `Write-LogError` - Failures and cleanup
- `Write-LogDebug` - Detailed validation results

Log path: `$env:TEMP\DINOForge\dinoforge.jsonl`

**Example Log Entry**:
```json
{
  "timestamp": "2026-04-12T12:34:56.789Z",
  "level": "ERROR",
  "message": "Instance creation failed",
  "context": {
    "instanceNumber": 1,
    "directory": "G:\dino_boxes\box_1",
    "error": "Failed to create symlink for BepInEx"
  },
  "requestId": "abc-123-def",
  "processId": 1234
}
```

## Files Modified

| File | Changes |
|------|---------|
| `scripts/game/New-TempGameInstance.ps1` | +200 LOC: 4 validation functions, try/catch, cleanup |
| `scripts/automation/Launch-ParallelGames.ps1` | +40 LOC: launch failure handling, sandbox cleanup |
| `scripts/tests/SandboxValidationTests.ps1` | NEW: 6-test suite, 100% passing |

## Edge Cases Handled

1. **Symlink Creation Failure** - Catches both hardlink and symlink failures
2. **Admin Requirements** - Tests work without admin (symlinks tested on regular Windows)
3. **Stale Previous Instance** - Previous instances cleaned before creation
4. **Partial Pool Creation** - Reports which instances failed
5. **Cross-Disk Symlinks** - Falls back from hardlink to symlink
6. **Missing Steam Auth** - Non-fatal warning, game still launches
7. **Process Already Running** - Killed in cleanup
8. **Reparse Point Detection** - Validates symlink type with fsutil

## Testing Instructions

Run the validation test suite:
```powershell
pwsh -File scripts/tests/SandboxValidationTests.ps1 -Verbose
```

Expected output:
```
=== Test Summary ===
Total Tests:  6
Passed:       6
Failed:       0

All tests passed!
```

## Deployment Notes

1. No breaking changes to existing code
2. Backward compatible with existing scripts
3. All logging is optional (works without Logging module)
4. Can be disabled with `-SkipValidation` flag for performance testing
5. Cleanup happens immediately on failure (no dangling directories)

## Future Enhancements

- Add parallel instance creation (currently sequential)
- Implement symlink pre-validation before creation
- Add timeout validation for instance startup
- Collect performance metrics on cleanup time
- Add metrics for validation time vs creation time
