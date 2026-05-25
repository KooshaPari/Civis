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
    [switch]$Verbose,
    [switch]$SkipValidation
)

$ErrorActionPreference = "Stop"

# Import logging module if available
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$loggingModule = Join-Path $scriptDir "..\shared\Logging.psm1"
$requestId = $env:DINO_REQUEST_ID

if (Test-Path $loggingModule) {
    Import-Module $loggingModule -Force -ErrorAction SilentlyContinue
}

function Write-Verbose {
    if ($Verbose) {
        Write-Host "[TEMP-INSTANCE] $args" -ForegroundColor Cyan
    }
}

function Copy-SteamAuth {
    param(
        [string]$SourcePath,
        [string]$TargetPath,
        [string]$RequestId
    )

    if (-not (Test-Path $SourcePath)) {
        if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
            Write-LogWarn "Steam userdata not found at $SourcePath" @{ sourcePath = $SourcePath } -RequestId $RequestId
        } else {
            Write-Warning "Steam userdata not found at $SourcePath"
        }
        return $false
    }

    try {
        if (-not (Test-Path $TargetPath)) {
            New-Item -ItemType Directory -Path $TargetPath -Force | Out-Null
        }

        Copy-Item -Path "$SourcePath\*" -Destination $TargetPath -Recurse -Force -ErrorAction SilentlyContinue
        Write-Verbose "Copied Steam auth from $SourcePath to $TargetPath"

        if (Get-Command Write-LogDebug -ErrorAction SilentlyContinue) {
            Write-LogDebug "Steam auth copied successfully" @{ sourcePath = $SourcePath; targetPath = $TargetPath } -RequestId $RequestId
        }
        return $true
    } catch {
        if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
            Write-LogWarn "Failed to copy Steam auth: $_" @{ sourcePath = $SourcePath; targetPath = $TargetPath; error = $_.ToString() } -RequestId $RequestId
        } else {
            Write-Warning "Failed to copy Steam auth: $_"
        }
        return $false
    }
}

function Validate-Symlink {
    param(
        [string]$LinkPath,
        [string]$LinkName,
        [string]$RequestId
    )

    if (-not (Test-Path -PathType Container $LinkPath)) {
        $msg = "Failed to create symlink: $LinkName at $LinkPath"
        if (Get-Command Write-LogError -ErrorAction SilentlyContinue) {
            Write-LogError $msg @{ linkName = $LinkName; linkPath = $LinkPath } -RequestId $RequestId
        } else {
            Write-Error $msg
        }
        return $false
    }

    if (Get-Command Write-LogDebug -ErrorAction SilentlyContinue) {
        Write-LogDebug "Symlink validated" @{ linkName = $LinkName; linkPath = $LinkPath } -RequestId $RequestId
    }
    return $true
}

function Validate-SteamAuth {
    param(
        [string]$InstanceDir,
        [string]$RequestId
    )

    $steamAuthDir = Join-Path $InstanceDir "LocalAppData\Steam"
    if (-not (Test-Path $steamAuthDir)) {
        if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
            Write-LogWarn "Steam auth directory not created" @{ directory = $steamAuthDir } -RequestId $RequestId
        } else {
            Write-Warning "Steam auth directory not created"
        }
        return $false
    }

    # Check for key Steam files
    $appConfig = Join-Path $steamAuthDir "7970\local\config"
    if (-not (Test-Path $appConfig)) {
        if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
            Write-LogWarn "Steam app config not found, game may not authenticate" @{ expectedPath = $appConfig; gameId = 7970 } -RequestId $RequestId
        } else {
            Write-Warning "Steam app config not found, game may not authenticate"
        }
        return $false
    }

    if (Get-Command Write-LogDebug -ErrorAction SilentlyContinue) {
        Write-LogDebug "Steam auth validated" @{ steamAuthDir = $steamAuthDir; appConfig = $appConfig } -RequestId $RequestId
    }
    return $true
}

function Validate-LocalAppDataIsolation {
    param(
        [string]$InstanceDir,
        [string]$RequestId
    )

    $isolatedLocalAppData = Join-Path $InstanceDir "LocalAppData"
    if (-not (Test-Path -PathType Container $isolatedLocalAppData)) {
        $msg = "LocalAppData isolation failed: directory $isolatedLocalAppData does not exist"
        if (Get-Command Write-LogError -ErrorAction SilentlyContinue) {
            Write-LogError $msg @{ directory = $isolatedLocalAppData } -RequestId $RequestId
        } else {
            Write-Error $msg
        }
        return $false
    }

    # Check if it's a reparse point (symlink/mount point)
    $linkInfo = cmd /c 'fsutil reparsepoint query "$isolatedLocalAppData"' 2>&1
    if ($linkInfo -match "Mount point|Symlink|not a reparse point") {
        if ($linkInfo -match "Mount point|Symlink") {
            if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
                Write-LogWarn "LocalAppData appears to be a link, may cause state sharing" @{ directory = $isolatedLocalAppData; linkInfo = $linkInfo } -RequestId $RequestId
            } else {
                Write-Warning "LocalAppData appears to be a link, may cause state sharing"
            }
        }
    }

    if (Get-Command Write-LogDebug -ErrorAction SilentlyContinue) {
        Write-LogDebug "LocalAppData isolation validated" @{ directory = $isolatedLocalAppData } -RequestId $RequestId
    }
    return $true
}

function Remove-InstanceDirectory {
    param(
        [string]$InstanceDir,
        [string]$RequestId
    )

    if (Test-Path $InstanceDir) {
        try {
            if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
                Write-LogWarn "Cleaning up failed instance directory..." @{ directory = $InstanceDir } -RequestId $RequestId
            } else {
                Write-Warning "Cleaning up failed instance directory: $InstanceDir"
            }
            Remove-Item -Path $InstanceDir -Recurse -Force -ErrorAction Stop
            if (Get-Command Write-LogInfo -ErrorAction SilentlyContinue) {
                Write-LogInfo "Instance cleanup completed" @{ directory = $InstanceDir } -RequestId $RequestId
            }
        } catch {
            if (Get-Command Write-LogError -ErrorAction SilentlyContinue) {
                Write-LogError "Failed to cleanup instance directory: $_" @{ directory = $InstanceDir; error = $_.ToString() } -RequestId $RequestId
            } else {
                Write-Error "Failed to cleanup instance directory: $_"
            }
        }
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
        [bool]$Verbose,
        [bool]$SkipValidation,
        [string]$RequestId
    )

    $GameRootDir = Split-Path $GameExePath -Parent
    $InstanceId = [System.Guid]::NewGuid().ToString().Substring(0, 8)
    $TempInstanceRoot = Join-Path $OutputDir "box_$InstanceNumber"

    try {
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
                throw "Failed to create both hardlink and symlink for executable: $_"
            }
        }

        # Validate executable link
        if (-not $SkipValidation -and (-not (Test-Path $destExe))) {
            throw "Executable link verification failed: $destExe does not exist"
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
                throw "Failed to create symlink for $linkName`: $_"
            }

            # Validate symlink creation
            if (-not $SkipValidation) {
                $validationResult = Validate-Symlink -LinkPath $destLink -LinkName $linkName -RequestId $RequestId
                if (-not $validationResult) {
                    throw "Symlink validation failed for $linkName"
                }
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

        # Validate LocalAppData isolation
        if (-not $SkipValidation) {
            $validationResult = Validate-LocalAppDataIsolation -InstanceDir $TempInstanceRoot -RequestId $RequestId
            if (-not $validationResult) {
                throw "LocalAppData isolation validation failed"
            }
        }

        # --- Copy Steam Auth if Requested ---
        if ($CopySteamAuth) {
            $steamAuthDest = Join-Path $tempLocalAppData "Steam"
            $authCopied = Copy-SteamAuth -SourcePath $SteamUserDataPath -TargetPath $steamAuthDest -RequestId $RequestId

            # Validate Steam auth copy
            if (-not $SkipValidation -and $authCopied) {
                $validationResult = Validate-SteamAuth -InstanceDir $TempInstanceRoot -RequestId $RequestId
                if (-not $validationResult) {
                    throw "Steam auth validation failed"
                }
            }
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

        if (Get-Command Write-LogInfo -ErrorAction SilentlyContinue) {
            Write-LogInfo "Instance created successfully" @{
                instanceNumber = $InstanceNumber
                instanceId = $InstanceId
                directory = $TempInstanceRoot
                pipeName = $pipeName
            } -RequestId $RequestId
        }

        [PSCustomObject]$instanceInfo
    }
    catch {
        Write-Host "Error creating instance #$InstanceNumber : $_" -ForegroundColor Red
        if (Get-Command Write-LogError -ErrorAction SilentlyContinue) {
            Write-LogError "Instance creation failed" @{
                instanceNumber = $InstanceNumber
                instanceId = $InstanceId
                directory = $TempInstanceRoot
                error = $_.ToString()
            } -RequestId $RequestId
        }

        # Cleanup on failure
        Remove-InstanceDirectory -InstanceDir $TempInstanceRoot -RequestId $RequestId
        return $null
    }
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
$createdInstances = @()

Write-Host "Creating $InstanceCount instance(s) sequentially..." -ForegroundColor Green
for ($i = 1; $i -le $InstanceCount; $i++) {
    $result = New-SingleInstance `
        -InstanceNumber $i `
        -OutputDir $effectiveOutputDir `
        -GameExePath $GameExePath `
        -BasePipeName $BasePipeName `
        -CopySteamAuth $CopySteamAuth `
        -SteamUserDataPath $SteamUserDataPath `
        -Verbose $Verbose `
        -SkipValidation $SkipValidation `
        -RequestId $requestId

    if ($result) {
        $results += $result
        $createdInstances += $result.Directory
    } else {
        Write-Host "Instance #$i creation failed, cleaning up..." -ForegroundColor Yellow
        if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
            Write-LogWarn "Instance creation failed, skipping" @{ instanceNumber = $i } -RequestId $requestId
        }
    }
}

# Check if any instances were created
if ($results.Count -eq 0) {
    Write-Host "No instances were created successfully" -ForegroundColor Red
    if (Get-Command Write-LogError -ErrorAction SilentlyContinue) {
        Write-LogError "Pool creation failed: no instances created" @{ requestedCount = $InstanceCount; createdCount = 0 } -RequestId $requestId
    }
    # Return empty pool instead of exit 1 (allows caller to handle gracefully)
    return $null
}

# Warn if partial pool
if ($results.Count -lt $InstanceCount) {
    Write-Host "WARNING: Partial pool creation ($($results.Count)/$InstanceCount instances)" -ForegroundColor Yellow
    if (Get-Command Write-LogWarn -ErrorAction SilentlyContinue) {
        Write-LogWarn "Partial pool creation" @{ requested = $InstanceCount; created = $results.Count } -RequestId $requestId
    }
}

# --- Return Pool Information ---
$poolInfo = @{
    Count           = $results.Count
    RequestedCount  = $InstanceCount
    OutputDir       = $effectiveOutputDir
    Instances       = $results
    CreatedDirs     = $createdInstances
    Status          = if ($results.Count -eq $InstanceCount) { "pool_created" } else { "partial_creation" }
    CreatedAt       = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
}

Write-Host "`nPool Summary:" -ForegroundColor Green
Write-Host "  Requested Instances: $($poolInfo.RequestedCount)"
Write-Host "  Created Instances:   $($poolInfo.Count)"
Write-Host "  Location:            $($poolInfo.OutputDir)"
Write-Host "  Status:              $($poolInfo.Status)"
Write-Host "  Created:             $($poolInfo.CreatedAt)"

if (Get-Command Write-LogInfo -ErrorAction SilentlyContinue) {
    Write-LogInfo "Pool creation completed" @{
        requestedCount = $poolInfo.RequestedCount
        createdCount = $poolInfo.Count
        status = $poolInfo.Status
        outputDir = $poolInfo.OutputDir
    } -RequestId $requestId
}

[PSCustomObject]$poolInfo
