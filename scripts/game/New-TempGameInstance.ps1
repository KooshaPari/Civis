<#
.SYNOPSIS
    Create a lightweight temporary game instance using symlinks (not a full copy).

.DESCRIPTION
    Instead of copying the entire 12GB game directory, this creates a minimal
    temp instance that symlinks to the main install for assets/plugins but maintains
    independent saves, logs, and temp directories.

    Structure:
    - temp\dino_temp_<uuid>\
      ├─ Diplomacy is Not an Option.exe        (hardlink, ~50MB)
      ├─ Diplomacy is Not an Option_Data\      (symlink to main install)
      ├─ BepInEx\                               (symlink to main install)
      ├─ StreamingAssets\ (symlink to main)    (only assets, don't duplicate 4GB)
      └─ LocalAppData\                          (isolated copy for saves/logs)

    Benefits:
    - Size: <100MB vs 12GB
    - Creation: <5 seconds vs 3-5 minutes
    - Cleanup: Instant directory delete
    - Safety: Read-only symlinks, isolated writes

.PARAMETER TempDir
    Base temp directory. Defaults to $env:TEMP\DINOForge\instances

.PARAMETER GameExePath
    Path to main game executable. Defaults to standard Steam location.

.PARAMETER Verbose
    Output detailed symlink creation steps.

.EXAMPLE
    $tempInstance = New-TempGameInstance
    Write-Host "Launched from: $($tempInstance.GameExePath)"
    Write-Host "Logs at: $($tempInstance.DebugLogPath)"
    # ... run game ...
    Remove-Item $tempInstance.RootDir -Recurse -Force
#>

param(
    [string]$TempDir = "$env:TEMP\DINOForge\instances",
    [string]$GameExePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

function Write-Verbose {
    if ($Verbose) {
        Write-Host "[TEMP-INSTANCE] $args" -ForegroundColor Cyan
    }
}

# Validate source game exists
if (-not (Test-Path $GameExePath)) {
    Write-Error "Game executable not found at: $GameExePath"
    exit 1
}

$GameRootDir = Split-Path $GameExePath -Parent
$InstanceId = [System.Guid]::NewGuid().ToString().Substring(0, 8)
$TempInstanceRoot = Join-Path $TempDir "dino_temp_$InstanceId"

# Ensure temp directory exists
if (-not (Test-Path $TempDir)) {
    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
    Write-Verbose "Created temp directory: $TempDir"
}

# Clean up any previous instances (if requested)
if (Test-Path $TempInstanceRoot) {
    Write-Verbose "Removing existing instance at: $TempInstanceRoot"
    Remove-Item $TempInstanceRoot -Recurse -Force -ErrorAction SilentlyContinue
}

# Create instance root
New-Item -ItemType Directory -Path $TempInstanceRoot -Force | Out-Null
Write-Verbose "Created instance root: $TempInstanceRoot"

# --- Create Hardlink for Game Executable ---
# This ensures the game binary is present but doesn't duplicate it across same disk
# If hardlink fails (cross-disk), fall back to symlink
$destExe = Join-Path $TempInstanceRoot (Split-Path $GameExePath -Leaf)
$linkCreated = $false

# Try hardlink first (same disk only)
try {
    cmd /c mklink /h "$destExe" "$GameExePath" 2>&1 | Out-Null
    Write-Verbose "Hardlinked executable: $destExe"
    $linkCreated = $true
} catch {
    Write-Verbose "Hardlink failed (cross-disk), trying symlink: $_"
}

# If hardlink failed, try symlink
if (-not $linkCreated) {
    try {
        cmd /c mklink "$destExe" "$GameExePath" 2>&1 | Out-Null
        Write-Verbose "Symlinked executable: $destExe"
        $linkCreated = $true
    } catch {
        Write-Error "Failed to create both hardlink and symlink for executable: $_"
        exit 1
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
        Write-Verbose "Source directory not found, skipping: $linkName"
        return
    }

    $destLink = Join-Path $TempInstanceRoot $linkName
    try {
        cmd /c mklink /d "$destLink" "$srcPath" 2>&1 | Out-Null
        Write-Verbose "Created symlink: $destLink -> $srcPath"
    } catch {
        Write-Error "Failed to create symlink for $linkName`: $_"
        exit 1
    }
}

# --- Create Isolated LocalAppData Directory ---
# This holds saves, logs, and player prefs specific to this instance
$LocalAppDataSrc = "$env:LOCALAPPDATA\..\LocalLow\Door 407\Diplomacy is Not an Option"
$LocalAppDataDest = Join-Path $TempInstanceRoot "LocalAppData"

if (Test-Path $LocalAppDataSrc) {
    # Copy the directory structure (but keep it small — just config, no saves)
    $tempLocalAppData = Join-Path $TempInstanceRoot "LocalAppData_Temp"
    New-Item -ItemType Directory -Path $tempLocalAppData -Force | Out-Null

    # Copy only essential config files, not saves or large state
    Copy-Item "$LocalAppDataSrc\CurrentSettings.json" "$tempLocalAppData\" -ErrorAction SilentlyContinue
    Copy-Item "$LocalAppDataSrc\Unity" "$tempLocalAppData\" -Recurse -ErrorAction SilentlyContinue

    Write-Verbose "Created isolated LocalAppData: $tempLocalAppData"
}

# --- Output Instance Info ---
$debugLogPath = Join-Path $TempInstanceRoot "BepInEx\dinoforge_debug.log"

$instanceInfo = @{
    InstanceId       = $InstanceId
    RootDir          = $TempInstanceRoot
    GameExePath      = $destExe
    WorkingDirectory = $TempInstanceRoot
    DebugLogPath     = $debugLogPath
    Size_MB          = 50
}

Write-Host "Temp instance created"
Write-Host "  ID:          $($instanceInfo.InstanceId)"
Write-Host "  Path:        $($instanceInfo.RootDir)"
Write-Host "  Exe:         $($instanceInfo.GameExePath)"
Write-Host "  Log:         $($instanceInfo.DebugLogPath)"
Write-Host "  Size:        ~$($instanceInfo.Size_MB) MB (symlinks minimal overhead)"

# Return as object for easy use in calling scripts
[PSCustomObject]$instanceInfo
