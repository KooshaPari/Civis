#Requires -Version 5.1
<#
.SYNOPSIS
  Exit 0 if UE 5.4 root is discoverable; exit 2 if not.

  Used by wait-for-ue.ps1 and agents to probe install without building.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$UeVersion = '5.4'
$UeFolder = "UE_$UeVersion"

function Get-EpicLauncherUeRoots {
    $paths = [System.Collections.Generic.List[string]]::new()
    $manifest = Join-Path $env:PROGRAMDATA 'Epic\UnrealEngineLauncher\LauncherInstalled.dat'
    if (-not (Test-Path -LiteralPath $manifest)) {
        return $paths
    }
    try {
        $json = Get-Content -LiteralPath $manifest -Raw -Encoding UTF8 | ConvertFrom-Json
        foreach ($entry in @($json.InstallationList)) {
            $loc = [string]$entry.InstallLocation
            if ([string]::IsNullOrWhiteSpace($loc)) { continue }
            $parent = Split-Path -Parent $loc.TrimEnd('\', '/')
            $candidate = Join-Path $parent $UeFolder
            if (Test-Path -LiteralPath $candidate) {
                $paths.Add((Resolve-Path -LiteralPath $candidate).Path)
            }
        }
    }
    catch {
        # ignore parse errors
    }
    return $paths
}

function Get-UeRoot {
    if ($env:UE_ROOT -and (Test-Path -LiteralPath $env:UE_ROOT)) {
        return (Resolve-Path -LiteralPath $env:UE_ROOT).Path
    }
    $candidates = @(
        (Join-Path ${env:ProgramFiles} "Epic Games\$UeFolder"),
        (Join-Path ${env:ProgramFiles(x86)} "Epic Games\$UeFolder"),
        (Join-Path $env:LOCALAPPDATA "EpicGamesLauncher\Engine\$UeFolder")
    )
    foreach ($path in $candidates) {
        if ($path -and (Test-Path -LiteralPath $path)) {
            return (Resolve-Path -LiteralPath $path).Path
        }
    }
    foreach ($path in (Get-EpicLauncherUeRoots)) {
        if (Test-Path -LiteralPath $path) {
            return $path
        }
    }
    return $null
}

$root = Get-UeRoot
if ($root) {
    Write-Host "UE ${UeVersion} at $root"
    $ubt = Join-Path $root 'Engine\Binaries\DotNET\UnrealBuildTool\UnrealBuildTool.exe'
    if (Test-Path -LiteralPath $ubt) {
        Write-Host "UBT: $ubt"
    }
    exit 0
}

Write-Host "UE $UeFolder not found. Set UE_ROOT or finish Epic Launcher install."
exit 2
