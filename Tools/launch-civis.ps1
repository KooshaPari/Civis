<#
.SYNOPSIS
    One-click Civis launcher for shortcuts and Explorer.

.DESCRIPTION
    Resolves the repository relative to this script, then delegates build,
    asset-root setup, logging, and detached launch to Tools/play.ps1.
#>
[CmdletBinding()]
param(
    [ValidateSet('release', 'debug')]
    [string]$Profile = 'release',

    [switch]$DebugLog
)

$ErrorActionPreference = 'Stop'
$play = Join-Path $PSScriptRoot 'play.ps1'
if (-not (Test-Path -LiteralPath $play)) {
    throw "Civis launcher is incomplete: missing $play"
}

$playArgs = @{
    Profile = $Profile
    NoTail  = $true
}
if ($DebugLog) {
    $playArgs.LogLevel = 'info,civ_bevy_ref=debug,wgpu=warn'
}

& $play @playArgs
exit $LASTEXITCODE
