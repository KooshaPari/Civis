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

# Generate request ID for tracing this entire operation
$requestId = [guid]::NewGuid().ToString()
$env:DINO_REQUEST_ID = $requestId

# Import logging module
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$loggingModule = Join-Path $scriptDir "..\shared\Logging.psm1"
if (Test-Path $loggingModule) {
    Import-Module $loggingModule -Force
} else {
    Write-Warning "Logging module not found at $loggingModule - falling back to Write-Host"
}

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
Write-LogInfo "Starting parallel game launcher" @{
    instanceCount = $InstanceCount
    gamePath = $GamePath
    outputDir = $OutputDir
    copySteamAuth = $CopySteamAuth
    requestId = $requestId
} -RequestId $requestId

# Trap: ensure cleanup always happens on any unhandled error
trap {
    Write-LogError "Unhandled error in launcher: $_" @{ error = $_.Exception.Message } -RequestId $requestId
    # Continue to finally block (implicit)
}

# Kill any existing game processes
Write-LogInfo "Cleaning up existing game processes" @{ } -RequestId $requestId
Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue |
    Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500

# Create sandbox pool using DINOBox infrastructure
Write-LogInfo "Creating isolated sandbox containers" @{ instanceCount = $InstanceCount } -RequestId $requestId
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
    Write-LogError "Failed to create sandbox pool: $_" @{
        error = $_.Exception.Message
        instanceCount = $InstanceCount
    } -RequestId $requestId
    exit 1
}

# Allow partial pools (some instances may have failed to create)
if (-not $boxPool) {
    Write-LogError "Sandbox pool creation returned no instances" @{ boxPool = $null } -RequestId $requestId
    exit 1
}

if (-not $boxPool.Instances -or $boxPool.Instances.Count -eq 0) {
    Write-LogError "Sandbox pool has no valid instances" @{ count = 0 } -RequestId $requestId
    exit 1
}

# Warn if partial pool, but continue with what we have
if ($boxPool.Count -lt $InstanceCount) {
    Write-LogWarn "Using partial sandbox pool ($($boxPool.Count)/$InstanceCount instances)" @{
        requested = $InstanceCount
        created = $boxPool.Count
    } -RequestId $requestId
}

Write-LogInfo "Launching game instances from sandbox containers" @{ instanceCount = $InstanceCount } -RequestId $requestId

# Launch game instances FROM sandbox directories
$processes = @()
$sandboxInstances = @()
$launchFailureFlag = $false

for ($i = 0; $i -lt $boxPool.Instances.Count; $i++) {
    $instance = $boxPool.Instances[$i]
    $sandboxGameExe = $instance.GameExePath
    $sandboxWorkDir = $instance.WorkingDirectory
    $instanceNum = $i + 1

    if (-not (Test-Path $sandboxGameExe)) {
        Write-LogError "Sandbox game executable not found" @{
            instanceNum = $instanceNum
            exePath = $sandboxGameExe
        } -RequestId $requestId
        $launchFailureFlag = $true
        break
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

        Write-LogInfo "Sandbox instance launched" @{
            instanceNum = $instanceNum
            pid = $proc.Id
            boxDir = $instance.Directory
            pipeName = $instance.PipeName
        } -RequestId $requestId

        # Stagger launch slightly to avoid race conditions
        Start-Sleep -Milliseconds 300
    } catch {
        Write-LogError "Failed to launch sandbox instance" @{
            instanceNum = $instanceNum
            error = $_.ToString()
            exePath = $sandboxGameExe
        } -RequestId $requestId
        $launchFailureFlag = $true
        break
    }
}

# If any launch failed, kill successful instances and cleanup
if ($launchFailureFlag) {
    Write-LogWarn "Launch failure detected, killing launched instances and cleaning up sandboxes" @{
        launchedCount = @($processes).Count
    } -RequestId $requestId

    # Kill all launched processes
    $processes | Stop-Process -Force -ErrorAction SilentlyContinue | Out-Null
    Start-Sleep -Milliseconds 500

    # Cleanup sandbox directories
    if ($boxPool.CreatedDirs) {
        Add-Type -AssemblyName Microsoft.VisualBasic
        foreach ($dir in $boxPool.CreatedDirs) {
            try {
                if (Test-Path $dir) {
                    Write-LogInfo "Removing sandbox directory after launch failure" @{ directory = $dir } -RequestId $requestId
                    [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory(
                        (Resolve-Path $dir).ProviderPath,
                        [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
                        [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin)
                }
            } catch {
                Write-LogError "Failed to cleanup sandbox directory" @{
                    directory = $dir
                    error = $_.ToString()
                } -RequestId $requestId
            }
        }
    }

    Write-LogError "Parallel launch operation failed" @{
        launchedCount = @($processes).Count
        requestedCount = $InstanceCount
    } -RequestId $requestId
    exit 1
}

Write-LogInfo "Waiting for instances to stabilize" @{ waitSeconds = 15 } -RequestId $requestId
Start-Sleep -Seconds 15

# Verify all instances are running
Write-LogInfo "Verifying instances" @{ } -RequestId $requestId
$running = $processes | Where-Object { -not $_.HasExited }
$runningCount = @($running).Count
$totalCount = @($processes).Count

if ($runningCount -eq 0) {
    Write-LogError "All instances exited immediately" @{
        outputDir = $boxPool.OutputDir
        totalLaunched = $totalCount
    } -RequestId $requestId
    exit 1
}

Write-LogInfo "Instance verification complete" @{
    runningCount = $runningCount
    totalCount = $totalCount
} -RequestId $requestId

if ($runningCount -lt $totalCount) {
    Write-LogWarn "Some instances crashed - check logs in sandbox directories" @{
        runningCount = $runningCount
        totalCount = $totalCount
    } -RequestId $requestId
}

# Output summary
Write-LogInfo "Launch operation completed successfully" @{
    runningCount = $runningCount
    totalCount = $totalCount
    sandboxLocation = $boxPool.OutputDir
} -RequestId $requestId

for ($i = 0; $i -lt $sandboxInstances.Count; $i++) {
    $inst = $sandboxInstances[$i]
    Write-LogDebug "Sandbox instance details" @{
        instanceNum = $i + 1
        directory = $inst.Directory
        exePath = $inst.GameExePath
        pipeName = $inst.PipeName
        debugLogPath = $inst.DebugLogPath
    } -RequestId $requestId
}

# Return pool and process information
return @{
    Processes = $running
    Count = $runningCount
    Pool = $boxPool
    Instances = $sandboxInstances
    MainGamePath = $GamePath
}
