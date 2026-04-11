<#
.SYNOPSIS
    Create a pool of lightweight temporary game instances using symlinks (not full copies).

.DESCRIPTION
    Instead of copying the entire 12GB game directory, this creates minimal
    temp instances that symlink to the main install for assets/plugins but maintain
    independent saves, logs, and temp directories.

    Supports N-instance pools with optional Steam auth copying for authenticated testing.

    Structure per instance:
    - output\box_<n>\
      ├─ Diplomacy is Not an Option.exe        (hardlink, ~50MB)
      ├─ Diplomacy is Not an Option_Data\      (symlink to main install)
      ├─ BepInEx\                               (symlink to main install)
      ├─ StreamingAssets\ (symlink to main)    (only assets, don't duplicate 4GB)
      └─ LocalAppData\                          (isolated copy for saves/logs)

    Benefits:
    - Size: <100MB vs 12GB per instance
    - Creation: <5 seconds each vs 3-5 minutes
    - Cleanup: Instant directory delete
    - Safety: Read-only symlinks, isolated writes
    - Multi-instance: Parallel creation up to 4 instances

.PARAMETER InstanceCount
    Number of instances to create. Defaults to 1.

.PARAMETER OutputDir
    Base output directory for instance pool. Defaults to G:\dino_boxes

.PARAMETER TempDir
    Fallback temp directory if OutputDir unavailable. Defaults to $env:TEMP\DINOForge\instances

.PARAMETER GameExePath
    Path to main game executable. Defaults to standard Steam location.

.PARAMETER BasePipeName
    Base name for inter-process pipes (one per instance). Defaults to dinoforge-game-bridge

.PARAMETER CopySteamAuth
    If specified, copy Steam auth files from user's Steam userdata to each instance.

.PARAMETER SteamUserDataPath
    Path to Steam userdata directory. Defaults to $env:APPDATA\Roaming\Steam\userdata

.PARAMETER Verbose
    Output detailed symlink creation steps.

.EXAMPLE
    # Create single instance
    $tempInstance = New-TempGameInstance
    Write-Host "Launched from: $($tempInstance.Instances[0].GameExePath)"

.EXAMPLE
    # Create pool of 2 instances with Steam auth
    $pool = New-TempGameInstance -InstanceCount 2 -CopySteamAuth -OutputDir "G:\dino_boxes"
    Write-Host "Pool created: $($pool.Count) instances"
    $pool.Instances | ForEach-Object { Write-Host "  Instance $_: $($_.Directory)" }
#>

param(
    [int]$InstanceCount = 1,
    [string]$OutputDir = "G:\dino_boxes",
    [string]$TempDir = "$env:TEMP\DINOForge\instances",
    [string]$GameExePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    [string]$BasePipeName = "dinoforge-game-bridge",
    [switch]$CopySteamAuth,
    [string]$SteamUserDataPath = "$env:APPDATA\Roaming\Steam\userdata",
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

function Write-Verbose {
    if ($Verbose) {
        Write-Host "[TEMP-INSTANCE] $args" -ForegroundColor Cyan
    }
}

function Copy-SteamAuth {
    param(
        [string]$SourcePath,
        [string]$TargetPath
    )

    if (-not (Test-Path $SourcePath)) {
        Write-Warning "Steam userdata not found at $SourcePath"
        return $false
    }

    try {
        if (-not (Test-Path $TargetPath)) {
            New-Item -ItemType Directory -Path $TargetPath -Force | Out-Null
        }

        Copy-Item -Path "$SourcePath\*" -Destination $TargetPath -Recurse -Force -ErrorAction SilentlyContinue
        Write-Verbose "Copied Steam auth from $SourcePath to $TargetPath"
        return $true
    } catch {
        Write-Warning "Failed to copy Steam auth: $_"
        return $false
    }
}

function New-SingleInstance {
    param(
        [int]$InstanceNumber,
        [string]$OutputDir,
        [string]$GameExePath,
        [string]$BasePipeName,
        [bool]$CopySteamAuth,
        [string]$SteamUserDataPath,
        [bool]$Verbose
    )

    $GameRootDir = Split-Path $GameExePath -Parent
    $InstanceId = [System.Guid]::NewGuid().ToString().Substring(0, 8)
    $TempInstanceRoot = Join-Path $OutputDir "box_$InstanceNumber"

    # Ensure output directory exists
    if (-not (Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
        if ($Verbose) { Write-Host "[TEMP-INSTANCE] Created output directory: $OutputDir" -ForegroundColor Cyan }
    }

    # Clean up any previous instance at this number
    if (Test-Path $TempInstanceRoot) {
        if ($Verbose) { Write-Host "[TEMP-INSTANCE] Removing existing instance at: $TempInstanceRoot" -ForegroundColor Cyan }
        Remove-Item $TempInstanceRoot -Recurse -Force -ErrorAction SilentlyContinue
    }

    # Create instance root
    New-Item -ItemType Directory -Path $TempInstanceRoot -Force | Out-Null
    if ($Verbose) { Write-Host "[TEMP-INSTANCE] Created instance root: $TempInstanceRoot" -ForegroundColor Cyan }

    # --- Create Hardlink for Game Executable ---
    $destExe = Join-Path $TempInstanceRoot (Split-Path $GameExePath -Leaf)
    $linkCreated = $false

    # Try hardlink first (same disk only)
    try {
        cmd /c mklink /h "$destExe" "$GameExePath" 2>&1 | Out-Null
        if ($Verbose) { Write-Host "[TEMP-INSTANCE] Hardlinked executable: $destExe" -ForegroundColor Cyan }
        $linkCreated = $true
    } catch {
        if ($Verbose) { Write-Host "[TEMP-INSTANCE] Hardlink failed (cross-disk), trying symlink: $_" -ForegroundColor Cyan }
    }

    # If hardlink failed, try symlink
    if (-not $linkCreated) {
        try {
            cmd /c mklink "$destExe" "$GameExePath" 2>&1 | Out-Null
            if ($Verbose) { Write-Host "[TEMP-INSTANCE] Symlinked executable: $destExe" -ForegroundColor Cyan }
            $linkCreated = $true
        } catch {
            Write-Error "Failed to create both hardlink and symlink for executable: $_"
            return $null
        }
    }

    # --- Create Symlinks for Large Directories ---
    @(
        @{name = "Diplomacy is Not an Option_Data"; src = (Join-Path $GameRootDir "Diplomacy is Not an Option_Data") },
        @{name = "BepInEx"; src = (Join-Path $GameRootDir "BepInEx") },
        @{name = "StreamingAssets"; src = (Join-Path $GameRootDir "StreamingAssets") }
    ) | ForEach-Object {
        $linkName = $_.name
        $srcPath = $_.src

        if (-not (Test-Path $srcPath)) {
            if ($Verbose) { Write-Host "[TEMP-INSTANCE] Source directory not found, skipping: $linkName" -ForegroundColor Cyan }
            return
        }

        $destLink = Join-Path $TempInstanceRoot $linkName
        try {
            cmd /c mklink /d "$destLink" "$srcPath" 2>&1 | Out-Null
            if ($Verbose) { Write-Host "[TEMP-INSTANCE] Created symlink: $destLink -> $srcPath" -ForegroundColor Cyan }
        } catch {
            Write-Error "Failed to create symlink for $linkName`: $_"
            return $null
        }
    }

    # --- Create Isolated LocalAppData Directory ---
    $LocalAppDataSrc = "$env:LOCALAPPDATA\..\LocalLow\Door 407\Diplomacy is Not an Option"
    $tempLocalAppData = Join-Path $TempInstanceRoot "LocalAppData"

    if (Test-Path $LocalAppDataSrc) {
        New-Item -ItemType Directory -Path $tempLocalAppData -Force | Out-Null
        Copy-Item "$LocalAppDataSrc\CurrentSettings.json" "$tempLocalAppData\" -ErrorAction SilentlyContinue
        Copy-Item "$LocalAppDataSrc\Unity" "$tempLocalAppData\" -Recurse -ErrorAction SilentlyContinue
        if ($Verbose) { Write-Host "[TEMP-INSTANCE] Created isolated LocalAppData: $tempLocalAppData" -ForegroundColor Cyan }
    } else {
        New-Item -ItemType Directory -Path $tempLocalAppData -Force | Out-Null
    }

    # --- Copy Steam Auth if Requested ---
    if ($CopySteamAuth) {
        $steamAuthDest = Join-Path $tempLocalAppData "Steam"
        Copy-SteamAuth -SourcePath $SteamUserDataPath -TargetPath $steamAuthDest | Out-Null
    }

    # --- Generate Pipe Name ---
    $pipeName = "$BasePipeName-instance-$InstanceNumber-$(Get-Random -Maximum 10000)"

    # --- Output Instance Info ---
    $debugLogPath = Join-Path $TempInstanceRoot "BepInEx\dinoforge_debug.log"

    $instanceInfo = @{
        InstanceNumber   = $InstanceNumber
        InstanceId       = $InstanceId
        Directory        = $TempInstanceRoot
        GameExePath      = $destExe
        WorkingDirectory = $TempInstanceRoot
        DebugLogPath     = $debugLogPath
        PipeName         = $pipeName
        Size_MB          = 50
        Status           = "ready"
    }

    Write-Host "Temp instance #$InstanceNumber created"
    Write-Host "  ID:          $($instanceInfo.InstanceId)"
    Write-Host "  Path:        $($instanceInfo.Directory)"
    Write-Host "  Exe:         $($instanceInfo.GameExePath)"
    Write-Host "  Pipe:        $($instanceInfo.PipeName)"
    Write-Host "  Log:         $($instanceInfo.DebugLogPath)"
    Write-Host "  Steam Auth:  $(if ($CopySteamAuth) { 'yes' } else { 'no' })"

    [PSCustomObject]$instanceInfo
}

# --- Main Execution ---

# Validate source game exists
if (-not (Test-Path $GameExePath)) {
    Write-Error "Game executable not found at: $GameExePath"
    exit 1
}

# Determine effective output directory
$effectiveOutputDir = $OutputDir
if (-not (Test-Path $effectiveOutputDir)) {
    Write-Host "OutputDir not available, using TempDir: $TempDir"
    $effectiveOutputDir = $TempDir
}

Write-Host "Creating pool of $InstanceCount instance(s)..." -ForegroundColor Green

# Create instances sequentially (PowerShell 5.1 compatible)
$results = @()

Write-Host "Creating $InstanceCount instance(s) sequentially..." -ForegroundColor Green
for ($i = 1; $i -le $InstanceCount; $i++) {
    $result = New-SingleInstance `
        -InstanceNumber $i `
        -OutputDir $effectiveOutputDir `
        -GameExePath $GameExePath `
        -BasePipeName $BasePipeName `
        -CopySteamAuth $CopySteamAuth `
        -SteamUserDataPath $SteamUserDataPath `
        -Verbose $Verbose

    if ($result) {
        $results += $result
    }
}

# --- Return Pool Information ---
$poolInfo = @{
    Count      = $InstanceCount
    OutputDir  = $effectiveOutputDir
    Instances  = $results
    Status     = "pool_created"
    CreatedAt  = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
}

Write-Host "`nPool Summary:" -ForegroundColor Green
Write-Host "  Total Instances: $($poolInfo.Count)"
Write-Host "  Location:        $($poolInfo.OutputDir)"
Write-Host "  Status:          $($poolInfo.Status)"
Write-Host "  Created:         $($poolInfo.CreatedAt)"

[PSCustomObject]$poolInfo
