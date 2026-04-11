#!/usr/bin/env powershell
<#
.SYNOPSIS
Launch N game instances in parallel from isolated sandbox containers (DINOBox).

.DESCRIPTION
Creates isolated temporary game instances using symlinks and isolated LocalAppData,
then launches games from sandbox directories. Each instance has its own:
- Save files and settings
- Log files (dinoforge_debug.log)
- Unique pipe name for MCP bridge
- Independent Steam auth (optional)

.PARAMETER InstanceCount
Number of game instances to launch (default: 4)

.PARAMETER OutputDir
Base directory for sandbox containers (default: G:\dino_boxes)
Falls back to $env:TEMP\DINOForge\instances if unavailable

.PARAMETER GamePath
Path to main DINO game directory (not sandbox).
If not specified, reads GameInstallPath from src/Directory.Build.props

.PARAMETER CopySteamAuth
Copy Steam auth files to each sandbox for authenticated testing

.PARAMETER Verbose
Enable verbose logging

.EXAMPLE
.\Launch-ParallelGames.ps1 -InstanceCount 2
Launch 2 sandboxed instances (uses G:\dino_boxes or temp fallback)

.\Launch-ParallelGames.ps1 -InstanceCount 4 -CopySteamAuth -Verbose
Launch 4 instances with Steam auth and detailed logging

.\Launch-ParallelGames.ps1 -InstanceCount 3 -OutputDir "D:\temp_games" -Verbose
Launch 3 instances in custom output directory
#>

param(
    [int]$InstanceCount = 4,
    [string]$GamePath,
    [string]$OutputDir = "G:\dino_boxes",
    [switch]$CopySteamAuth,
    [switch]$Verbose
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Resolve main game path from Directory.Build.props if not provided
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

# Validate main game path (source, not sandbox)
$mainGameExe = Join-Path $GamePath "Diplomacy is Not an Option.exe"
if (-not (Test-Path $mainGameExe)) {
    Write-Error "Main game executable not found: $mainGameExe"
    exit 1
}

Write-Host "=== DINOForge Parallel Game Launcher (Sandboxed) ===" -ForegroundColor Cyan
Write-Host "Instance count: $InstanceCount"
Write-Host "Main game path: $GamePath"
Write-Host "Sandbox output: $OutputDir"
Write-Host ""

# Kill any existing game processes
Write-Host "Cleaning up existing game processes..." -ForegroundColor Yellow
Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue |
    Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# Create sandbox pool using DINOBox infrastructure
Write-Host ""
Write-Host "Creating isolated sandbox containers..." -ForegroundColor Cyan
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$newTempInstanceScript = Join-Path $scriptDir "..\game\New-TempGameInstance.ps1"

if (-not (Test-Path $newTempInstanceScript)) {
    Write-Error "New-TempGameInstance.ps1 not found at: $newTempInstanceScript"
    exit 1
}

try {
    $boxPool = & $newTempInstanceScript `
        -InstanceCount $InstanceCount `
        -OutputDir $OutputDir `
        -GameExePath $mainGameExe `
        -CopySteamAuth:$CopySteamAuth `
        -Verbose:$Verbose
} catch {
    Write-Error "Failed to create sandbox pool: $_"
    exit 1
}

if (-not $boxPool -or -not $boxPool.Instances) {
    Write-Error "Sandbox pool creation returned empty results"
    exit 1
}

Write-Host ""
Write-Host "Launching game instances from sandbox containers..." -ForegroundColor Cyan

# Launch game instances FROM sandbox directories
$processes = @()
$sandboxInstances = @()

for ($i = 0; $i -lt $boxPool.Instances.Count; $i++) {
    $instance = $boxPool.Instances[$i]
    $sandboxGameExe = $instance.GameExePath
    $sandboxWorkDir = $instance.WorkingDirectory
    $instanceNum = $i + 1

    if (-not (Test-Path $sandboxGameExe)) {
        Write-Error "Sandbox game executable not found at: $sandboxGameExe"
        exit 1
    }

    # Create process arguments for sandbox instance
    $processArgs = @{
        FilePath = $sandboxGameExe
        WorkingDirectory = $sandboxWorkDir
        WindowStyle = "Hidden"
        PassThru = $true
    }

    try {
        $proc = Start-Process @processArgs
        $processes += $proc
        $sandboxInstances += $instance

        if ($Verbose) {
            Write-Host "  [Sandbox $instanceNum] PID=$($proc.Id) Box=$($instance.Directory) Pipe=$($instance.PipeName)" -ForegroundColor Green
        } else {
            Write-Host "  [Sandbox $instanceNum] Launched from $($instance.Directory) (PID=$($proc.Id))" -ForegroundColor Green
        }

        # Stagger launch slightly to avoid race conditions
        Start-Sleep -Milliseconds 300
    } catch {
        Write-Error "Failed to launch sandbox instance $instanceNum : $_"
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
    Write-Error "All instances exited immediately. Check sandbox logs at $($boxPool.OutputDir)"
    exit 1
}

Write-Host "[PASS] Running instances: $runningCount/$totalCount" -ForegroundColor Green

if ($runningCount -lt $totalCount) {
    Write-Warning "Some instances crashed. Check logs in sandbox directories."
}

# Output summary
Write-Host ""
Write-Host "=== Launch Summary ===" -ForegroundColor Cyan
Write-Host "Instances: $runningCount running (from $totalCount created)"
Write-Host "Sandbox location: $($boxPool.OutputDir)"
Write-Host ""
Write-Host "Sandbox Details:"
for ($i = 0; $i -lt $sandboxInstances.Count; $i++) {
    $inst = $sandboxInstances[$i]
    Write-Host "  Sandbox $($i+1):" -ForegroundColor DarkCyan
    Write-Host "    Directory: $($inst.Directory)" -ForegroundColor DarkGray
    Write-Host "    Exe:       $($inst.GameExePath)" -ForegroundColor DarkGray
    Write-Host "    Pipe:      $($inst.PipeName)" -ForegroundColor DarkGray
    Write-Host "    Log:       $($inst.DebugLogPath)" -ForegroundColor DarkGray
}
Write-Host ""
Write-Host "[PASS] Parallel sandboxed launcher complete" -ForegroundColor Green

# Return pool and process information
return @{
    Processes = $running
    Count = $runningCount
    Pool = $boxPool
    Instances = $sandboxInstances
    MainGamePath = $GamePath
}
