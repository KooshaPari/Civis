#Requires -Version 5.1
<#
.SYNOPSIS
  Agent-facing smoke: Rust protocol tests + optional Unreal preflight or full UBT build.

.EXIT CODES
  0  All checks passed
  1  Test or preflight failure
#>
[CmdletBinding()]
param(
    [switch] $SkipUnreal,
    [switch] $FullUnreal,
    [switch] $IncludeBevy,
    [switch] $Budgeted,
    [string] $TargetDirectory,
    [long] $ExpectedGrowthBytes,
    [long] $ReserveBytes,
    [int] $Jobs,
    [string] $ReceiptDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

function Test-UnrealUbtAvailable {
    if ($env:UE_ROOT -and (Test-Path -LiteralPath $env:UE_ROOT)) {
        $root = (Resolve-Path -LiteralPath $env:UE_ROOT).Path
        $ubt = Join-Path $root 'Engine\Binaries\DotNET\UnrealBuildTool\UnrealBuildTool.exe'
        $bat = Join-Path $root 'Engine\Build\BatchFiles\Build.bat'
        return ((Test-Path -LiteralPath $ubt) -or (Test-Path -LiteralPath $bat))
    }
    $detect = Join-Path $RepoRoot 'clients\unreal-show\scripts\detect-ue.ps1'
    if (-not (Test-Path -LiteralPath $detect)) { return $false }
    & powershell -NoProfile -ExecutionPolicy Bypass -File $detect *> $null
    return ($LASTEXITCODE -eq 0)
}

if ($Budgeted) {
    foreach ($name in @('TargetDirectory','ExpectedGrowthBytes','ReserveBytes','Jobs','ReceiptDirectory')) {
        if (-not $PSBoundParameters.ContainsKey($name)) {
            throw "-Budgeted requires explicit -$name"
        }
    }
    if ($ExpectedGrowthBytes -le 0 -or $ReserveBytes -le 0 -or $Jobs -lt 1 -or $Jobs -gt 64) {
        throw '-Budgeted requires positive ExpectedGrowthBytes/ReserveBytes and Jobs in the range 1..64'
    }

    $helper = Join-Path $PSScriptRoot 'ci\invoke-budgeted-child.ps1'
    if (-not (Test-Path -LiteralPath $helper -PathType Leaf)) {
        throw "Budgeted helper missing: $helper"
    }
    $pwsh = (Get-Command pwsh -CommandType Application -ErrorAction Stop | Select-Object -First 1).Path
    $childArguments = [System.Collections.Generic.List[string]]::new()
    foreach ($argument in @('-NoProfile','-ExecutionPolicy','Bypass','-File',(Resolve-Path -LiteralPath $PSCommandPath).Path)) {
        [void]$childArguments.Add($argument)
    }
    if ($SkipUnreal) { [void]$childArguments.Add('-SkipUnreal') }
    if ($FullUnreal) { [void]$childArguments.Add('-FullUnreal') }
    if ($IncludeBevy) { [void]$childArguments.Add('-IncludeBevy') }

    $helperParameters = @{
        ChildPath = $pwsh
        ChildArgumentsJson = ($childArguments.ToArray() | ConvertTo-Json -Compress)
        TargetDirectory = $TargetDirectory
        ExpectedGrowthBytes = $ExpectedGrowthBytes
        ReserveBytes = $ReserveBytes
        Jobs = $Jobs
        ReceiptDirectory = $ReceiptDirectory
        Execute = $true
    }
    & $pwsh -NoProfile -ExecutionPolicy Bypass -File $helper @helperParameters
    exit $LASTEXITCODE
}

function Invoke-PlayableGate {
    # Terrain playability gate: WS + watch + Unreal preflight in series.
    Write-Host '==> playable gate (WS, watch, Unreal preflight)' -ForegroundColor Cyan

    # ws_smoke includes civ_pins[].job asserts (UX-01) in ws_jsonrpc_sim_snapshot_returns_snapshot_fields
    Write-Host '==> civ-server WS smoke (health, snapshot, civ_pins job, spawn palette)' -ForegroundColor Cyan
    & cargo test -p civ-server --quiet --test ws_smoke
    if ($LASTEXITCODE -ne 0) { exit 1 }

    Write-Host '==> civ-watch API smoke' -ForegroundColor Cyan
    & cargo test -p civ-watch --quiet
    if ($LASTEXITCODE -ne 0) { exit 1 }

    if ($SkipUnreal) {
        return
    }

    $unrealScripts = Join-Path $RepoRoot 'clients\unreal-show\scripts'
    if ($FullUnreal) {
        $build = Join-Path $unrealScripts 'build.ps1'
        if ((Test-UnrealUbtAvailable) -and (Test-Path -LiteralPath $build)) {
            Write-Host '==> Unreal full build (build.ps1, UBT)' -ForegroundColor Cyan
            & powershell -NoProfile -ExecutionPolicy Bypass -File $build
            if ($LASTEXITCODE -ne 0) { exit 1 }
        }
        else {
            Write-Host '==> FullUnreal: no UE_ROOT/UBT — skipping full compile' -ForegroundColor Yellow
        }
        return
    }

    $verify = Join-Path $unrealScripts 'verify-unreal-ready.ps1'
    if (Test-Path -LiteralPath $verify) {
        Write-Host '==> Unreal offline preflight' -ForegroundColor Cyan
        & powershell -NoProfile -ExecutionPolicy Bypass -File $verify
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
}

Push-Location $RepoRoot
try {
    Write-Host '==> civis-3d quick gates (catalog, scenario, mod-host, godot rust)' -ForegroundColor Cyan
    & just civis-3d-catalog-check
    if ($LASTEXITCODE -ne 0) { exit 1 }
    & just civis-3d-scenario-check
    if ($LASTEXITCODE -ne 0) { exit 1 }
    & just civis-3d-mod-check
    if ($LASTEXITCODE -ne 0) { exit 1 }
    & just godot-test
    if ($LASTEXITCODE -ne 0) { exit 1 }

    if ($IncludeBevy) {
        Write-Host '==> civ-bevy-ref lib tests' -ForegroundColor Cyan
        & cargo test -p civ-bevy-ref --quiet
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }

    Invoke-PlayableGate

    Write-Host '==> agent-smoke passed' -ForegroundColor Green
    exit 0
}
finally {
    Pop-Location
}
