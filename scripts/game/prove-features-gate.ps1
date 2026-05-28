#!/usr/bin/env pwsh
<#
.SYNOPSIS
    SPEC-007 regression gate: runtime features baseline (F9/F10, overlays, Mods button).

.DESCRIPTION
    -ValidateOnly / -DryCheck: CI-safe checks without a running game (unit/characterization
      tests + GameLaunch project compile). Always exits 0 when structural checks pass.
    Full mode (no -ValidateOnly): requires DINO_GAME_PATH or docs/proof-of-features VLM proof.
    Use on self-hosted runners or workflow_dispatch, not default ubuntu CI.

.PARAMETER ValidateOnly
    Run offline regression checks only; skip VLM proof and live game requirements.

.PARAMETER DryCheck
    Alias for -ValidateOnly.

.PARAMETER SkipProveFeatures
    Skip proof generation; only validate existing artifacts (delegates to .claude gate).

.PARAMETER RequireGame
    Fail if DINO_GAME_PATH is unset (for manual / self-hosted full runs).

.EXAMPLE
    pwsh scripts/game/prove-features-gate.ps1 -ValidateOnly
#>

[CmdletBinding()]
param(
    [Alias('DryCheck')]
    [switch]$ValidateOnly,

    [switch]$SkipProveFeatures,

    [switch]$RequireGame
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $repoRoot

$proofDir = Join-Path $repoRoot 'docs/proof-of-features'
$gateResultPath = Join-Path $proofDir 'gate_result.json'
$claudeGate = Join-Path $repoRoot '.claude/commands/prove-features-gate.ps1'

function Write-Gate([string]$Message, [string]$Level = 'Info') {
    $color = switch ($Level) {
        'Error' { 'Red' }
        'Success' { 'Green' }
        'Warn' { 'Yellow' }
        default { 'Cyan' }
    }
    $ts = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm:ss')
    Write-Host "${ts}Z [SPEC-007] $Message" -ForegroundColor $color
}

function Write-GateResult([hashtable]$Result) {
    if (-not (Test-Path $proofDir)) {
        New-Item -ItemType Directory -Force -Path $proofDir | Out-Null
    }
    $Result | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $gateResultPath -Encoding utf8
}

$script:GameProcessBaseName = 'Diplomacy is Not an Option'
$script:GameExecutableFileName = 'Diplomacy is Not an Option.exe'

function Test-GameAttachOnlyMode {
    $value = $env:DINO_GAME_ALREADY_RUNNING
    return (-not [string]::IsNullOrWhiteSpace($value)) -and
        ($value -eq '1' -or $value -ieq 'true')
}

function Stop-StrayGameLaunchProcesses {
    Write-Gate 'Stopping stray game processes (pre/post flight)' 'Warn'
    foreach ($procName in @($script:GameProcessBaseName, 'UnityCrashHandler64')) {
        Get-Process -Name $procName -ErrorAction SilentlyContinue |
            Stop-Process -Force -ErrorAction SilentlyContinue
    }
    # Game Launch Protocol: wait 3s and verify no processes remain
    Start-Sleep -Seconds 3
    $remaining = @()
    foreach ($procName in @($script:GameProcessBaseName, 'UnityCrashHandler64')) {
        $remaining += @(Get-Process -Name $procName -ErrorAction SilentlyContinue)
    }
    if ($remaining.Count -gt 0) {
        Write-Gate "Warning: $($remaining.Count) game-related process(es) still running after cleanup" 'Warn'
    }
}

function Test-GameLaunchAllSkipped([string]$TestOutput) {
    if ([string]::IsNullOrWhiteSpace($TestOutput)) {
        return $false
    }
    if ($TestOutput -notmatch 'Skipped:\s+(\d+),\s+Total:\s+(\d+)') {
        return $false
    }
    $skipped = [int]$Matches[1]
    $total = [int]$Matches[2]
    if ($total -le 0 -or $skipped -ne $total) {
        return $false
    }
    if ($TestOutput -match 'Passed:\s+(\d+)') {
        return [int]$Matches[1] -eq 0
    }
    return $true
}

function Resolve-DinoGameExePath {
    $path = $env:DINO_GAME_PATH
    if ([string]::IsNullOrWhiteSpace($path)) {
        return $null
    }
    if (Test-Path -LiteralPath $path -PathType Leaf) {
        if ([string]::Equals([IO.Path]::GetFileName($path), $script:GameExecutableFileName, [StringComparison]::OrdinalIgnoreCase)) {
            return $path
        }
        return $null
    }
    if (Test-Path -LiteralPath $path -PathType Container) {
        $exe = Join-Path $path $script:GameExecutableFileName
        if (Test-Path -LiteralPath $exe) {
            return $exe
        }
    }
    return $null
}

function Test-GameAvailable {
    return $null -ne (Resolve-DinoGameExePath)
}

function Get-GameInstallRoot {
    $exe = Resolve-DinoGameExePath
    if ($exe) {
        return Split-Path -Parent $exe
    }
    if (-not [string]::IsNullOrWhiteSpace($env:DINO_GAME_PATH)) {
        return $env:DINO_GAME_PATH
    }
    # Windows dev default only — ubuntu CI has no G: drive; avoid Join-Path on missing drive.
    if ($IsWindows -or ($env:OS -match 'Windows')) {
        return 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
    }
    return $null
}

function Test-GameInstalledForTests {
    $root = Get-GameInstallRoot
    if ([string]::IsNullOrWhiteSpace($root)) {
        return $false
    }
    $managedDir = Join-Path $root 'Diplomacy is Not an Option_Data\Managed'
    return Test-Path -LiteralPath (Join-Path $managedDir 'UnityEngine.dll')
}

function Test-KeyInputSystemTestsCompiled([string]$TestsDllPath) {
    if (-not (Test-Path -LiteralPath $TestsDllPath)) {
        return $false
    }
    try {
        $asm = [System.Reflection.Assembly]::LoadFrom($TestsDllPath)
        return $null -ne $asm.GetType('DINOForge.Tests.KeyInputSystemTests', $false)
    }
    catch {
        return $false
    }
}

function Get-OfflineSpec007TestFilter([string]$TestsDllPath) {
    $parts = @(
        'FullyQualifiedName~NativeMenuInjectorCharacterizationTests',
        'FullyQualifiedName~ModMenuTests'
    )
    if ((Test-GameInstalledForTests) -or (Test-KeyInputSystemTestsCompiled $TestsDllPath)) {
        $parts += 'FullyQualifiedName~KeyInputSystemTests'
    }
    return ($parts -join '|')
}

function Invoke-ValidateOnlyGate {
    Write-Gate 'Validate-only mode (no game required)'

    $requiredPaths = @(
        'docs/specs/SPEC-007-runtime-features-baseline.md',
        'src/Tests/GameLaunch/GameLaunchOverlayTests.cs',
        'src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs',
        'src/Tests/GameLaunch/GameLaunchUiTests.cs',
        'src/Tests/NativeMenuInjectorCharacterizationTests.cs',
        'src/Tests/ModMenuTests.cs'
    )

    $missing = @($requiredPaths | Where-Object { -not (Test-Path -LiteralPath $_) })
    if ($missing.Count -gt 0) {
        Write-Gate "Missing SPEC-007 regression assets: $($missing -join ', ')" 'Error'
        Write-GateResult @{
            timestamp = (Get-Date).ToUniversalTime().ToString('o')
            status    = 'FAILED'
            mode      = 'validate-only'
            reason    = 'Missing required test/spec files'
            missing   = $missing
        }
        exit 1
    }

    Write-Gate 'Building GameLaunch test project (compile-only gate)'
    dotnet build 'src/Tests/GameLaunch/DINOForge.Tests.GameLaunch.csproj' `
        -c Release --verbosity minimal
    if ($LASTEXITCODE -ne 0) {
        Write-GateResult @{
            timestamp = (Get-Date).ToUniversalTime().ToString('o')
            status    = 'FAILED'
            mode      = 'validate-only'
            reason    = 'GameLaunch test project build failed'
        }
        exit $LASTEXITCODE
    }

    $testsDll = Join-Path $repoRoot 'src/Tests/bin/Release/net8.0/DINOForge.Tests.dll'
    if (Test-Path -LiteralPath $testsDll) {
        $offlineFilter = Get-OfflineSpec007TestFilter $testsDll
        Write-Gate "Running offline SPEC-007 unit/characterization tests (--no-build); filter: $offlineFilter"
        dotnet test 'src/Tests/DINOForge.Tests.csproj' `
            -c Release `
            --no-build `
            --filter $offlineFilter `
            --verbosity minimal
        if ($LASTEXITCODE -ne 0) {
            Write-GateResult @{
                timestamp = (Get-Date).ToUniversalTime().ToString('o')
                status    = 'FAILED'
                mode      = 'validate-only'
                reason    = 'Offline SPEC-007 dotnet tests failed'
            }
            exit $LASTEXITCODE
        }
    }
    else {
        Write-Gate 'DINOForge.Tests.dll not built (Runtime/Unity absent) — structural checks only' 'Warn'
        $injectorSource = Join-Path $repoRoot 'src/Runtime/UI/NativeMenuInjector.cs'
        if (-not (Test-Path -LiteralPath $injectorSource)) {
            Write-Gate "Expected source file missing: $injectorSource" 'Error'
            exit 1
        }
        $injectorText = Get-Content -LiteralPath $injectorSource -Raw
        if ($injectorText -notmatch 'DINOForge_ModsButton') {
            Write-Gate 'NativeMenuInjector.cs missing DINOForge_ModsButton guard' 'Error'
            exit 1
        }
    }

    $gameNote = if (Test-GameAvailable) {
        'DINO_GAME_PATH set — run without -ValidateOnly on self-hosted for live GameLaunch proof.'
    } else {
        'DINO_GAME_PATH not set — live GameLaunch tests skipped (expected on ubuntu CI).'
    }
    Write-Gate $gameNote 'Warn'

    Write-GateResult @{
        timestamp           = (Get-Date).ToUniversalTime().ToString('o')
        status              = 'PASSED'
        mode                = 'validate-only'
        reason              = 'Offline SPEC-007 regression checks passed'
        game_path_available = (Test-GameAvailable)
        live_game_required  = $false
    }
    Write-Gate 'Validate-only gate PASSED' 'Success'
    exit 0
}

function Invoke-FullGate {
    if ($RequireGame -and -not (Test-GameAvailable)) {
        Write-Gate 'RequireGame set but DINO_GAME_PATH is missing or invalid' 'Error'
        Write-GateResult @{
            timestamp = (Get-Date).ToUniversalTime().ToString('o')
            status    = 'FAILED'
            mode      = 'full'
            reason    = 'DINO_GAME_PATH required but not available'
        }
        exit 1
    }

    if (Test-GameAvailable) {
        $resolvedExe = Resolve-DinoGameExePath
        if ($resolvedExe) {
            $env:DINO_GAME_PATH = $resolvedExe
            Write-Gate "Using game executable: $resolvedExe"
        }

        if (-not (Test-GameAttachOnlyMode)) {
            Stop-StrayGameLaunchProcesses
        }
        else {
            Write-Gate 'Attach-only mode (DINO_GAME_ALREADY_RUNNING) — skipping pre-flight process cleanup' 'Warn'
        }

        $gameExit = 0
        $gameTestOutput = ''
        try {
            Write-Gate 'Running GameLaunch E2E tests (DINO_GAME_PATH detected)'
            $gameTestOutput = dotnet test 'src/Tests/GameLaunch/DINOForge.Tests.GameLaunch.csproj' `
                -c Release `
                --filter 'Category=GameLaunch' `
                --verbosity minimal 2>&1 | Out-String
            $gameExit = $LASTEXITCODE
            if ($gameTestOutput) {
                Write-Host $gameTestOutput
            }
            if ($gameExit -eq 0 -and (Test-GameLaunchAllSkipped $gameTestOutput)) {
                Write-Gate 'GameLaunch tests all skipped (fixture not initialized) — treating as failure' 'Error'
                $gameExit = 1
            }
        }
        finally {
            if (-not (Test-GameAttachOnlyMode)) {
                Stop-StrayGameLaunchProcesses
            }
        }

        if ($gameExit -ne 0) {
            Write-GateResult @{
                timestamp = (Get-Date).ToUniversalTime().ToString('o')
                status    = 'FAILED'
                mode      = 'full'
                reason    = 'GameLaunch tests failed'
            }
            exit $gameExit
        }

        Write-GateResult @{
            timestamp = (Get-Date).ToUniversalTime().ToString('o')
            status    = 'PASSED'
            mode      = 'full'
            reason    = 'GameLaunch SPEC-007 tests passed'
        }
        Write-Gate 'Full game gate PASSED' 'Success'
        exit 0
    }

    if (-not (Test-Path -LiteralPath $claudeGate)) {
        Write-Gate "No game and no VLM gate at $claudeGate — use -ValidateOnly in CI" 'Error'
        Write-GateResult @{
            timestamp = (Get-Date).ToUniversalTime().ToString('o')
            status    = 'SKIPPED'
            mode      = 'full'
            reason    = 'No DINO_GAME_PATH and no VLM proof gate script'
        }
        exit 1
    }

    Write-Gate 'No game — validating VLM proof artifacts via .claude/commands/prove-features-gate.ps1'
    $gateArgs = @()
    if ($SkipProveFeatures) { $gateArgs += '-SkipProveFeatures' }
    & $claudeGate @gateArgs
    exit $LASTEXITCODE
}

if ($ValidateOnly) {
    Invoke-ValidateOnlyGate
}

Invoke-FullGate
