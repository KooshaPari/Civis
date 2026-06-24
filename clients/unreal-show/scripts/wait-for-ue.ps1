#Requires -Version 5.1
<#
.SYNOPSIS
  Poll until UE 5.4 is installed, then run build.ps1.

.PARAMETER IntervalSeconds
  Seconds between detection attempts (default 120).

.PARAMETER MaxAttempts
  Stop after this many attempts (0 = unlimited).

.EXAMPLE
  .\wait-for-ue.ps1
  .\wait-for-ue.ps1 -IntervalSeconds 60 -MaxAttempts 30
#>
[CmdletBinding()]
param(
    [int] $IntervalSeconds = 120,
    [int] $MaxAttempts = 0
)

$ErrorActionPreference = 'Stop'
$BuildScript = Join-Path $PSScriptRoot 'build.ps1'
$DetectScript = Join-Path $PSScriptRoot 'detect-ue.ps1'

if (-not (Test-Path -LiteralPath $DetectScript)) {
    Write-Error "Missing $DetectScript"
}

$attempt = 0
while ($true) {
    $attempt++
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Attempt $attempt — checking for UE 5.4..." -ForegroundColor Yellow

    & $DetectScript
    if ($LASTEXITCODE -eq 0) {
        Write-Host "UE found. Running full build..." -ForegroundColor Green
        & $BuildScript
        exit $LASTEXITCODE
    }

    if ($MaxAttempts -gt 0 -and $attempt -ge $MaxAttempts) {
        Write-Host "Giving up after $MaxAttempts attempts." -ForegroundColor Red
        exit 2
    }

    Write-Host "UE not ready (exit 2). Sleeping ${IntervalSeconds}s..." -ForegroundColor DarkGray
    Start-Sleep -Seconds $IntervalSeconds
}
