#Requires -Version 5.1
<#
.SYNOPSIS
  Exit 0 if a supported UE install is found; exit 2 if not.
  Prints detected version and path. Sets script-scope $script:DetectedUeVersion for callers.
#>
[CmdletBinding()]
param(
    [string[]] $PreferredVersions = @('5.4', '5.7', '5.6', '5.5')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-InstalledUeFolders {
    $found = [System.Collections.Generic.List[object]]::new()
    $roots = @(
        (Join-Path ${env:ProgramFiles} 'Epic Games'),
        (Join-Path ${env:ProgramFiles(x86)} 'Epic Games')
    )
    foreach ($root in $roots) {
        if (-not $root -or -not (Test-Path -LiteralPath $root)) { continue }
        Get-ChildItem -LiteralPath $root -Directory -Filter 'UE_*' -ErrorAction SilentlyContinue | ForEach-Object {
            $ver = $_.Name -replace '^UE_', ''
            $ubt = Join-Path $_.FullName 'Engine\Binaries\DotNET\UnrealBuildTool\UnrealBuildTool.exe'
            $buildBat = Join-Path $_.FullName 'Engine\Build\BatchFiles\Build.bat'
            if ((Test-Path -LiteralPath $ubt) -or (Test-Path -LiteralPath $buildBat)) {
                $found.Add([pscustomobject]@{ Version = $ver; Root = $_.FullName })
            }
        }
    }
    if ($env:UE_ROOT -and (Test-Path -LiteralPath $env:UE_ROOT)) {
        $name = Split-Path -Leaf $env:UE_ROOT
        $ver = if ($name -match '^UE_(.+)$') { $Matches[1] } else { 'custom' }
        $found.Add([pscustomobject]@{ Version = $ver; Root = (Resolve-Path -LiteralPath $env:UE_ROOT).Path })
    }
    return $found
}

function Select-UeInstall {
    param([string[]] $Order)
    $installed = Get-InstalledUeFolders
    foreach ($want in $Order) {
        $hit = $installed | Where-Object { $_.Version -eq $want } | Select-Object -First 1
        if ($hit) { return $hit }
    }
    return $installed | Select-Object -First 1
}

$all = @(Get-InstalledUeFolders)
if ($all.Count -gt 0) {
    Write-Host 'Installed UE engines:' -ForegroundColor DarkGray
    foreach ($e in ($all | Sort-Object Version)) {
        Write-Host "  UE_$($e.Version) -> $($e.Root)"
    }
}

$pick = Select-UeInstall -Order $PreferredVersions
if (-not $pick) {
    Write-Host "No UE install found (wanted: $($PreferredVersions -join ', ')). Set UE_ROOT or finish Epic Launcher install."
    exit 2
}

$uprojectVer = '5.7'
$uprojectPath = Join-Path (Resolve-Path (Join-Path $PSScriptRoot '..')).Path 'CivShow.uproject'
if (Test-Path -LiteralPath $uprojectPath) {
    $uproj = Get-Content -LiteralPath $uprojectPath -Raw | ConvertFrom-Json
    if ($uproj.EngineAssociation) { $uprojectVer = [string]$uproj.EngineAssociation }
}

if ($pick.Version -ne $uprojectVer) {
    Write-Host "Note: CivShow.uproject expects $uprojectVer but building with UE_$($pick.Version) at:" -ForegroundColor Yellow
}
else {
    Write-Host "UE $($pick.Version) matches uproject ($uprojectVer) at:" -ForegroundColor Green
}
Write-Host $pick.Root

$ubt = Join-Path $pick.Root 'Engine\Binaries\DotNET\UnrealBuildTool\UnrealBuildTool.exe'
if (Test-Path -LiteralPath $ubt) {
    Write-Host "UBT: $ubt"
}

# Export for build.ps1 dot-sourcing
$script:DetectedUeVersion = $pick.Version
$script:DetectedUeRoot = $pick.Root
exit 0
