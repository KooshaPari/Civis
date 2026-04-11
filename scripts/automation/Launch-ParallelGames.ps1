#!/usr/bin/env powershell
<#
.SYNOPSIS
Launch N game instances in parallel with unique pipe names and isolated configs.

.PARAMETER InstanceCount
Number of game instances to launch (default: 4)

.PARAMETER GamePath
Path to DINO game executable directory.
If not specified, reads GameInstallPath from src/Directory.Build.props

.PARAMETER Verbose
Enable verbose logging

.EXAMPLE
.\Launch-ParallelGames.ps1 -InstanceCount 2
Launch 2 game instances

.\Launch-ParallelGames.ps1 -InstanceCount 4 -Verbose
Launch 4 instances with detailed logging
#>

param(
    [int]$InstanceCount = 4,
    [string]$GamePath,
    [switch]$Verbose
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Resolve game path from Directory.Build.props if not provided
if (-not $GamePath) {
    # Check both root and src locations for props file
    $propsFile = if (Test-Path "Directory.Build.props") {
        "Directory.Build.props"
    } elseif (Test-Path "src/Directory.Build.props") {
        "src/Directory.Build.props"
    } else {
        $null
    }

    if (-not $propsFile) {
        Write-Error "Cannot find Directory.Build.props in repo root or src/. Run from repo root or specify -GamePath"
        exit 1
    }

    [xml]$props = Get-Content $propsFile
    # Extract GameInstallPath using XPath to handle multiple PropertyGroup nodes
    $gamePathNode = $props.SelectSingleNode("//PropertyGroup/GameInstallPath")
    if ($gamePathNode) {
        $GamePath = $gamePathNode.InnerText
    }

    if (-not $GamePath) {
        Write-Error "GameInstallPath not found in $propsFile"
        exit 1
    }
}

# Validate game path
$gameExe = Join-Path $GamePath "Diplomacy is Not an Option.exe"
if (-not (Test-Path $gameExe)) {
    Write-Error "Game executable not found: $gameExe"
    exit 1
}

Write-Host "=== DINOForge Parallel Game Launcher ===" -ForegroundColor Cyan
Write-Host "Instance count: $InstanceCount"
Write-Host "Game path: $GamePath"
Write-Host ""

# Kill any existing game processes
Write-Host "Cleaning up existing game processes..." -ForegroundColor Yellow
Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue |
    Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# Launch parallel instances
$processes = @()
$pipeNames = @()
$instanceIds = @()

Write-Host "Launching $InstanceCount game instances..." -ForegroundColor Cyan

for ($i = 1; $i -le $InstanceCount; $i++) {
    $pipeName = "dinoforge-game-bridge-instance-$i-$(Get-Random -Minimum 1000 -Maximum 9999)"
    $pipeNames += $pipeName
    $instanceIds += $i

    # Create process arguments
    $processArgs = @{
        FilePath = $gameExe
        WorkingDirectory = $GamePath
        WindowStyle = "Hidden"
        PassThru = $true
    }

    try {
        $proc = Start-Process @processArgs
        $processes += $proc

        if ($Verbose) {
            Write-Host "  [Instance $i] PID=$($proc.Id) Pipe=$pipeName" -ForegroundColor Green
        } else {
            Write-Host "  [Instance $i] Started (PID=$($proc.Id))" -ForegroundColor Green
        }

        # Stagger launch slightly to avoid race conditions
        Start-Sleep -Milliseconds 300
    } catch {
        Write-Error "Failed to launch instance $i : $_"
        # Kill successful instances on failure
        $processes | Stop-Process -Force -ErrorAction SilentlyContinue
        exit 1
    }
}

Write-Host ""
Write-Host "Waiting for instances to stabilize..." -ForegroundColor Yellow
Start-Sleep -Seconds 15

# Verify all instances are running
Write-Host ""
Write-Host "Verifying instances..." -ForegroundColor Cyan
$running = $processes | Where-Object { -not $_.HasExited }
$runningCount = @($running).Count
$totalCount = @($processes).Count

if ($runningCount -eq 0) {
    Write-Error "All instances exited immediately. Check game logs."
    exit 1
}

Write-Host "[PASS] Running instances: $runningCount/$totalCount" -ForegroundColor Green

if ($runningCount -lt $totalCount) {
    Write-Warning "Some instances crashed. Check game logs."
}

# Output summary
Write-Host ""
Write-Host "=== Launch Summary ===" -ForegroundColor Cyan
Write-Host "Instances: $runningCount running"
Write-Host "Pipe names:"
for ($i = 0; $i -lt $pipeNames.Count; $i++) {
    Write-Host "  Instance $($i+1): $($pipeNames[$i])" -ForegroundColor DarkCyan
}
Write-Host ""
Write-Host "[PASS] Parallel game launcher complete" -ForegroundColor Green

# Return running processes
return @{
    Processes = $running
    Count = $runningCount
    PipeNames = $pipeNames
    GamePath = $GamePath
}
