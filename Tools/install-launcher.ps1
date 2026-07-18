<#
.SYNOPSIS
    Install or remove the Civis Start Menu launcher for the current user.
#>
[CmdletBinding()]
param(
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$programs = [Environment]::GetFolderPath('Programs')
$shortcutDir = Join-Path $programs 'Civis'
$shortcutPath = Join-Path $shortcutDir 'Civis.lnk'

if ($Uninstall) {
    Remove-Item -LiteralPath $shortcutPath -Force -ErrorAction SilentlyContinue
    if ((Test-Path -LiteralPath $shortcutDir) -and
        -not (Get-ChildItem -LiteralPath $shortcutDir -Force)) {
        Remove-Item -LiteralPath $shortcutDir -Force
    }
    Write-Host "[install-launcher] Removed $shortcutPath" -ForegroundColor Green
    exit 0
}

$launcher = Join-Path $PSScriptRoot 'launch-civis.ps1'
if (-not (Test-Path -LiteralPath $launcher)) {
    throw "Missing launcher: $launcher"
}

New-Item -ItemType Directory -Force -Path $shortcutDir | Out-Null
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = (Get-Command powershell.exe).Source
$shortcut.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$launcher`""
$shortcut.WorkingDirectory = $repoRoot
$shortcut.Description = 'Build and launch Civis'

$iconCandidates = @(
    (Join-Path $repoRoot 'target\release\civ-standalone.exe'),
    'G:\civis-target-gate\release\civ-standalone.exe'
)
$icon = $iconCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if ($icon) {
    $shortcut.IconLocation = "$icon,0"
}

$shortcut.Save()
Write-Host "[install-launcher] Installed $shortcutPath" -ForegroundColor Green
Write-Host "[install-launcher] The first launch builds Civis if no release binary exists."
