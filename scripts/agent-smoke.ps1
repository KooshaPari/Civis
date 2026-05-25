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
    [switch] $IncludeBevy
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

Push-Location $RepoRoot
try {
    Write-Host '==> civ-server WS smoke (health, snapshot, spawn pin)' -ForegroundColor Cyan
    & cargo test -p civ-server --quiet --test ws_smoke
    if ($LASTEXITCODE -ne 0) { exit 1 }

    Write-Host '==> civ-watch API smoke' -ForegroundColor Cyan
    & cargo test -p civ-watch --quiet
    if ($LASTEXITCODE -ne 0) { exit 1 }

    if ($IncludeBevy) {
        Write-Host '==> civ-bevy-ref lib tests' -ForegroundColor Cyan
        & cargo test -p civ-bevy-ref --quiet
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }

    if (-not $SkipUnreal) {
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
        }
        else {
            $verify = Join-Path $unrealScripts 'verify-unreal-ready.ps1'
            if (Test-Path -LiteralPath $verify) {
                Write-Host '==> Unreal offline preflight' -ForegroundColor Cyan
                & powershell -NoProfile -ExecutionPolicy Bypass -File $verify
                if ($LASTEXITCODE -ne 0) { exit 1 }
            }
        }
    }

    Write-Host '==> agent-smoke passed' -ForegroundColor Green
    exit 0
}
finally {
    Pop-Location
}
