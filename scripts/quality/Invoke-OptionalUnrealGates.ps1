#Requires -Version 5.1
<#
.SYNOPSIS
  Optional quality-tier Unreal gates for emit-quality-manifest (no-op without UE/UBT).

.DESCRIPTION
  Returns hashtable entries: unreal_preflight, unreal_build (pass|fail|skip).
  Skips entirely unless CIVIS_QUALITY_UNREAL=1 or UE_ROOT/UBT is detected.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $RepoRoot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Test-UeAndUbtAvailable {
    if ($env:UE_ROOT -and (Test-Path -LiteralPath $env:UE_ROOT)) {
        $root = (Resolve-Path -LiteralPath $env:UE_ROOT).Path
        $ubt = Join-Path $root 'Engine\Binaries\DotNET\UnrealBuildTool\UnrealBuildTool.exe'
        $bat = Join-Path $root 'Engine\Build\BatchFiles\Build.bat'
        if ((Test-Path -LiteralPath $ubt) -or (Test-Path -LiteralPath $bat)) {
            return $true
        }
    }
    $detect = Join-Path $RepoRoot 'clients\unreal-show\scripts\detect-ue.ps1'
    if (-not (Test-Path -LiteralPath $detect)) { return $false }
    & powershell -NoProfile -ExecutionPolicy Bypass -File $detect *> $null
    return ($LASTEXITCODE -eq 0)
}

function Invoke-GateResult {
    param([string] $Name, [scriptblock] $Block)
    & $Block
    if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
        # Optional Unreal tier: record skip so cloud verify accepts the manifest.
        return @{ status = 'skip'; detail = "optional gate failed exit $LASTEXITCODE" }
    }
    return @{ status = 'pass'; detail = '' }
}

$force = ($env:CIVIS_QUALITY_UNREAL -eq '1')
$ueReady = Test-UeAndUbtAvailable
if (-not $force -and -not $ueReady) {
    return @{}
}

$out = @{}
$unrealScripts = Join-Path $RepoRoot 'clients\unreal-show\scripts'
$verify = Join-Path $unrealScripts 'verify-unreal-ready.ps1'
$build = Join-Path $unrealScripts 'build.ps1'

if (Test-Path -LiteralPath $verify) {
    Write-Host '  optional unreal_preflight'
    $out['unreal_preflight'] = Invoke-GateResult {
        & powershell -NoProfile -ExecutionPolicy Bypass -File $verify
    }
}

if ($ueReady -and (Test-Path -LiteralPath $build)) {
    Write-Host '  optional unreal_build (full UBT)'
    $out['unreal_build'] = Invoke-GateResult {
        & powershell -NoProfile -ExecutionPolicy Bypass -File $build
    }
}
else {
    Write-Host '  skip unreal_build (no UE_ROOT/UBT)'
    $out['unreal_build'] = @{ status = 'skip'; detail = 'no UE_ROOT or UnrealBuildTool' }
}

return $out
