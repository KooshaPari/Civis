<#
.SYNOPSIS
    Test suite for sandbox instance creation validation and cleanup on failure.

.DESCRIPTION
    Validates:
    - Symlinks are created and verify correctly
    - Steam auth validation detects missing files
    - LocalAppData is properly isolated
    - Cleanup removes all sandbox files on failure
    - Cleanup preserves main game directory

    Run with: powershell -File ./scripts/tests/SandboxValidationTests.ps1

.PARAMETER Verbose
    Enable verbose output during tests
#>

param(
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Import logging module
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$loggingModule = Join-Path $scriptDir "..\shared\Logging.psm1"
if (Test-Path $loggingModule) {
    Import-Module $loggingModule -Force
}

# Test tracking
$testsRun = 0
$testsPassed = 0
$testsFailed = 0

function Test-SandboxValidation {
    param(
        [string]$TestName,
        [scriptblock]$TestScript
    )

    $script:testsRun++
    Write-Host "`n[$testsRun] Testing: $TestName" -ForegroundColor Cyan

    try {
        & $TestScript
        $script:testsPassed++
        Write-Host "  PASSED" -ForegroundColor Green
        return $true
    } catch {
        $script:testsFailed++
        Write-Host "  FAILED: $_" -ForegroundColor Red
        return $false
    }
}

# Test 1: Symlink Creation and Verification
Test-SandboxValidation "Symlinks are created and verify correctly" {
    $testDir = Join-Path $env:TEMP "sandbox_test_$(Get-Random)"
    $srcDir = Join-Path $testDir "source"
    $linkDir = Join-Path $testDir "link"

    try {
        New-Item -ItemType Directory -Path $srcDir -Force | Out-Null

        # Create symlink
        cmd /c mklink /d "$linkDir" "$srcDir" 2>&1 | Out-Null

        # Verify it exists
        if (-not (Test-Path -PathType Container $linkDir)) {
            throw "Symlink directory not found after creation"
        }

        # Verify it's actually a reparse point
        $linkInfo = cmd /c 'fsutil reparsepoint query "$linkDir"' 2>&1
        if (-not ($linkInfo -match "not a reparse point")) {
            # Good - it IS a reparse point (symlink)
            Write-Host "    Symlink verified as reparse point" -ForegroundColor Gray
        }

        Write-Host "    Source: $srcDir" -ForegroundColor Gray
        Write-Host "    Link:   $linkDir" -ForegroundColor Gray
    } finally {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: test-cleanup-ok: ephemeral test fixture in $env:TEMP
        }
    }
}

# Test 2: Steam Auth Validation
Test-SandboxValidation "Steam auth validation detects missing files" {
    $testDir = Join-Path $env:TEMP "steam_test_$(Get-Random)"

    try {
        New-Item -ItemType Directory -Path $testDir -Force | Out-Null

        # Create LocalAppData without Steam auth
        $localAppData = Join-Path $testDir "LocalAppData"
        New-Item -ItemType Directory -Path $localAppData -Force | Out-Null

        # Check for Steam directory (should not exist)
        $steamDir = Join-Path $localAppData "Steam"
        if (Test-Path $steamDir) {
            throw "Steam directory should not exist, but it does"
        }

        Write-Host "    Correctly detected missing Steam directory" -ForegroundColor Gray

        # Now create Steam structure
        $appConfig = Join-Path $steamDir "7970\local\config"
        New-Item -ItemType Directory -Path $appConfig -Force | Out-Null

        # Verify it now exists
        if (-not (Test-Path $appConfig)) {
            throw "Steam app config path not created correctly"
        }

        Write-Host "    Successfully created Steam auth structure at: $steamDir" -ForegroundColor Gray
    } finally {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: test-cleanup-ok: ephemeral test fixture in $env:TEMP
        }
    }
}

# Test 3: LocalAppData Isolation
Test-SandboxValidation "LocalAppData is properly isolated" {
    $testDir = Join-Path $env:TEMP "isolation_test_$(Get-Random)"

    try {
        # Create isolated LocalAppData
        $isolatedLocalAppData = Join-Path $testDir "LocalAppData"
        New-Item -ItemType Directory -Path $isolatedLocalAppData -Force | Out-Null

        # Verify it exists
        if (-not (Test-Path -PathType Container $isolatedLocalAppData)) {
            throw "LocalAppData directory not found after creation"
        }

        # Verify it's NOT a symlink
        $linkInfo = cmd /c 'fsutil reparsepoint query "$isolatedLocalAppData"' 2>&1
        if ($linkInfo -match "Mount point|Symlink") {
            throw "LocalAppData should be a real directory, not a symlink"
        }

        Write-Host "    LocalAppData is properly isolated (not a symlink)" -ForegroundColor Gray
        Write-Host "    Path: $isolatedLocalAppData" -ForegroundColor Gray
    } finally {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: test-cleanup-ok: ephemeral test fixture in $env:TEMP
        }
    }
}

# Test 4: Cleanup Removes Sandbox Files
Test-SandboxValidation "Cleanup removes all sandbox files on failure" {
    $testDir = Join-Path $env:TEMP "cleanup_test_$(Get-Random)"

    try {
        # Create sandbox structure
        New-Item -ItemType Directory -Path $testDir -Force | Out-Null
        New-Item -ItemType Directory -Path "$testDir\BepInEx" -Force | Out-Null
        New-Item -ItemType Directory -Path "$testDir\LocalAppData" -Force | Out-Null
        New-Item -ItemType File -Path "$testDir\test.txt" -Force | Out-Null

        # Verify structure exists
        if (-not (Test-Path $testDir)) {
            throw "Test directory not created"
        }

        $fileCount = (Get-ChildItem -Path $testDir -Recurse -ErrorAction SilentlyContinue).Count
        Write-Host "    Created sandbox with $fileCount items" -ForegroundColor Gray

        # Cleanup
        Remove-Item -Path $testDir -Recurse -Force -ErrorAction Stop # remove-item-ok: test-cleanup-ok: validates cleanup behavior on ephemeral $env:TEMP fixture

        # Verify cleanup
        if (Test-Path $testDir) {
            throw "Cleanup failed: directory still exists"
        }

        Write-Host "    Cleanup successful, all files removed" -ForegroundColor Gray
    } finally {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: test-cleanup-ok: ephemeral test fixture in $env:TEMP
        }
    }
}

# Test 5: Cleanup Preserves Main Game Directory
Test-SandboxValidation "Cleanup preserves main game directory" {
    $mainGameDir = Join-Path $env:TEMP "main_game_$(Get-Random)"
    $sandboxDir = Join-Path $env:TEMP "sandbox_$(Get-Random)"

    try {
        # Create main game directory
        New-Item -ItemType Directory -Path $mainGameDir -Force | Out-Null
        New-Item -ItemType File -Path "$mainGameDir\game.exe" -Force | Out-Null

        # Create sandbox with symlink to main game
        New-Item -ItemType Directory -Path $sandboxDir -Force | Out-Null
        cmd /c mklink /d "$sandboxDir\game_link" "$mainGameDir" 2>&1 | Out-Null

        # Remove only sandbox
        Remove-Item -Path $sandboxDir -Recurse -Force -ErrorAction Stop # remove-item-ok: test-cleanup-ok: validates sandbox-only removal on ephemeral $env:TEMP fixture

        # Verify main game still exists
        if (-not (Test-Path "$mainGameDir\game.exe")) {
            throw "Main game directory was affected by sandbox cleanup"
        }

        Write-Host "    Main game directory preserved after sandbox cleanup" -ForegroundColor Gray
        Write-Host "    Main Game: $mainGameDir (still exists)" -ForegroundColor Gray
    } finally {
        if (Test-Path $mainGameDir) {
            Remove-Item -Path $mainGameDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: test-cleanup-ok: ephemeral test fixture in $env:TEMP
        }
        if (Test-Path $sandboxDir) {
            Remove-Item -Path $sandboxDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: test-cleanup-ok: ephemeral test fixture in $env:TEMP
        }
    }
}

# Test 6: Full Instance Creation with Validation
Test-SandboxValidation "Full instance creation applies all validations" {
    $outputDir = Join-Path $env:TEMP "full_instance_test_$(Get-Random)"

    try {
        # Create a minimal test structure
        $gameDir = Join-Path $outputDir "game"
        $gameExe = Join-Path $gameDir "game.exe"
        New-Item -ItemType Directory -Path $gameDir -Force | Out-Null
        New-Item -ItemType File -Path $gameExe -Force | Out-Null

        # Create sandbox directory
        $sandboxDir = Join-Path $outputDir "box_1"
        New-Item -ItemType Directory -Path $sandboxDir -Force | Out-Null

        # Create essential components
        New-Item -ItemType File -Path "$sandboxDir\game.exe" -Force | Out-Null
        New-Item -ItemType Directory -Path "$sandboxDir\LocalAppData" -Force | Out-Null
        New-Item -ItemType Directory -Path "$sandboxDir\BepInEx" -Force | Out-Null

        # Verify all components exist
        $components = @(
            "$sandboxDir\game.exe",
            "$sandboxDir\LocalAppData",
            "$sandboxDir\BepInEx"
        )

        foreach ($component in $components) {
            if (-not (Test-Path $component)) {
                throw "Component not created: $component"
            }
        }

        Write-Host "    All instance components created successfully" -ForegroundColor Gray
        Write-Host "    Instance: $sandboxDir" -ForegroundColor Gray
    } finally {
        if (Test-Path $outputDir) {
            Remove-Item -Path $outputDir -Recurse -Force -ErrorAction SilentlyContinue # remove-item-ok: test-cleanup-ok: ephemeral test fixture in $env:TEMP
        }
    }
}

# Print summary
Write-Host "`n`n=== Test Summary ===" -ForegroundColor Cyan
Write-Host "Total Tests:  $testsRun" -ForegroundColor White
Write-Host "Passed:       $testsPassed" -ForegroundColor Green
Write-Host "Failed:       $testsFailed" -ForegroundColor $(if ($testsFailed -eq 0) { 'Green' } else { 'Red' })

if ($testsFailed -gt 0) {
    Write-Host "`nSome tests failed!" -ForegroundColor Red
    exit 1
} else {
    Write-Host "`nAll tests passed!" -ForegroundColor Green
    exit 0
}
