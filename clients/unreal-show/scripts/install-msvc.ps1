#Requires -Version 5.1
<#
.SYNOPSIS
  Add MSVC v14.44 (UE 5.7) to an existing VS 2022 install via setup.exe.

  Run elevated if modify fails with access denied.
#>
[CmdletBinding()]
param(
    [switch] $Passive,
    [string] $InstallPath = 'C:\Program Files\Microsoft Visual Studio\2022\Community'
)

$ErrorActionPreference = 'Stop'

$running = Get-Process setup, vs_installer -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "Visual Studio Installer is already open (PID $($running.Id -join ','))." -ForegroundColor Yellow
    Write-Host "Complete or close that window, then re-run this script." -ForegroundColor Yellow
    exit 3
}

$Setup = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\setup.exe'
if (-not (Test-Path -LiteralPath $Setup)) {
    Write-Error "Visual Studio Installer not found at $Setup"
}

$components = @(
    'Microsoft.VisualStudio.Workload.VCTools',
    'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
    'Microsoft.VisualStudio.Component.VC.14.44.17.14.x86.x64',
    'Microsoft.VisualStudio.Component.Windows11SDK.22621'
)

$addArgs = ($components | ForEach-Object { '--add', $_ }) -join ' '
$quiet = if ($Passive) { '--passive' } else { '--quiet' }

Write-Host "Modifying VS at: $InstallPath" -ForegroundColor Cyan
Write-Host "Components: $($components -join ', ')" -ForegroundColor DarkGray

$argList = @(
    'modify',
    '--installPath', $InstallPath,
    '--norestart',
    '--nocache'
) + ($components | ForEach-Object { '--add'; $_ })
if ($Passive) { $argList += '--passive' } else { $argList += '--quiet' }

& $Setup @argList
$code = $LASTEXITCODE
Write-Host "setup.exe exit: $code"
if ($code -eq 0) {
    Write-Host 'MSVC install/modify finished. Re-run scripts\build.ps1' -ForegroundColor Green
}
exit $code
