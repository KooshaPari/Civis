<#
.SYNOPSIS
    Create N isolated game instances (DINOBox pool) for parallel testing.

.DESCRIPTION
    Creates a pool of isolated game instances with:
    - Symlinked read-only assets (avoids 12GB duplication)
    - Isolated pipe names (dinoforge-game-bridge-<uuid>)
    - Unique save directories
    - Per-instance Steam auth copies (optional)

    Each box is configured for independent concurrent launch without mutex conflicts.

.PARAMETER Count
    Number of boxes to create (default: 4).

.PARAMETER BaseDir
    Base directory for boxes (default: G:\dino_boxes).

.PARAMETER SteamAuthSource
    Path to Steam auth source (e.g., "live", "C:\path\to\steam") or "none" to skip (default: "none").

.EXAMPLE
    $pool = .\New-DINOBoxPool.ps1 -Count 4 -BaseDir "G:\dino_boxes"
    # $pool contains: @{ 1 = @{BoxPath, PipeName, ...}, 2 = ... }

    # Launch all boxes
    $pool.Keys | ForEach-Object {
        $box = $pool[$_]
        Start-Process -FilePath "$($box.BoxPath)\Diplomacy is Not an Option.exe" -WorkingDirectory $box.BoxPath
    }
#>

param(
    [int]$Count = 4,
    [string]$BaseDir = "G:\dino_boxes",
    [string]$SteamAuthSource = "none"
)

$ErrorActionPreference = "Stop"

# Validate game install exists
$mainGameDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
if (-not (Test-Path $mainGameDir)) {
    throw "Main game install not found at $mainGameDir"
}

Write-Host "=== DINOBox Pool Creator ==="
Write-Host "Creating $Count isolated game instances..."
Write-Host "Base directory: $BaseDir"
Write-Host ""

# Create base directory
if (-not (Test-Path $BaseDir)) {
    New-Item -ItemType Directory -Path $BaseDir -Force | Out-Null
    Write-Host "[+] Created base directory"
}

$pool = @{}

for ($i = 1; $i -le $Count; $i++) {
    $boxName = "box_$i"
    $boxPath = Join-Path $BaseDir $boxName
    $boxUuid = [guid]::NewGuid().ToString().Substring(0, 8)
    $pipeName = "dinoforge-game-bridge-$boxUuid"

    Write-Host ""
    Write-Host "Creating box $i/$Count ($boxName)..."

    # Create box root directory
    if (-not (Test-Path $boxPath)) {
        New-Item -ItemType Directory -Path $boxPath -Force | Out-Null
    }

    # Create game data directories
    $dataDir = Join-Path $boxPath "Diplomacy is Not an Option_Data"
    if (-not (Test-Path $dataDir)) {
        New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
    }

    # Symlink read-only assets to main install
    # _Data and StreamingAssets are read-only, so symlinks are safe
    $mainDataDir = Join-Path $mainGameDir "Diplomacy is Not an Option_Data"

    # Create symlinks for critical read-only directories
    $symlinkTargets = @(
        @{ Link = "Managed"; Target = "$mainDataDir\Managed" },
        @{ Link = "Plugins"; Target = "$mainDataDir\Plugins" },
        @{ Link = "StreamingAssets"; Target = "$mainDataDir\StreamingAssets" },
        @{ Link = "Resources"; Target = "$mainDataDir\Resources" }
    )

    foreach ($link in $symlinkTargets) {
        $linkPath = Join-Path $dataDir $link.Link
        if (-not (Test-Path $linkPath)) {
            # Remove if exists (shouldn't happen on first run)
            if (Test-Path $linkPath) {
                Remove-Item $linkPath -Force -ErrorAction SilentlyContinue
            }

            cmd /c mklink /d "$linkPath" "$($link.Target)" 2>&1 | Out-Null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  [+] Symlinked $($link.Link)"
            } else {
                Write-Host "  [!] Failed to symlink $($link.Link) (may already exist)"
            }
        }
    }

    # Copy game executable (relatively small, ~50MB)
    $srcExe = Join-Path $mainGameDir "Diplomacy is Not an Option.exe"
    $dstExe = Join-Path $boxPath "Diplomacy is Not an Option.exe"
    if (-not (Test-Path $dstExe)) {
        Copy-Item $srcExe $dstExe -Force
        Write-Host "  [+] Copied game executable"
    }

    # Copy steam_api64.dll (Steam Stub DRM)
    $srcSteamApi = Join-Path $mainGameDir "steam_api64.dll"
    $dstSteamApi = Join-Path $boxPath "steam_api64.dll"
    if ((Test-Path $srcSteamApi) -and -not (Test-Path $dstSteamApi)) {
        Copy-Item $srcSteamApi $dstSteamApi -Force
        Write-Host "  [+] Copied steam_api64.dll"
    }

    # Create BepInEx directory structure
    $bepinexDir = Join-Path $boxPath "BepInEx"
    $pluginsDir = Join-Path $bepinexDir "plugins"
    $configDir = Join-Path $bepinexDir "config"
    $ecsPluginsDir = Join-Path $bepinexDir "ecs_plugins"

    foreach ($dir in @($bepinexDir, $pluginsDir, $configDir, $ecsPluginsDir)) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    Write-Host "  [+] Created BepInEx structure"

    # Create BepInEx.cfg with unique pipe name
    $configFile = Join-Path $configDir "BepInEx.cfg"
    $bepinexCfgContent = @"
[Logging.Harmonyxs]
Enabled = false

[Logging.Console]
Enabled = true

[Logging.Disk]
Enabled = true
LogLevels = 15

[DINOForge]
PipeName = $pipeName
"@

    Set-Content -Path $configFile -Value $bepinexCfgContent -Force
    Write-Host "  [+] Created BepInEx.cfg (pipe: $pipeName)"

    # Copy boot.config from main install (handles single-instance setting)
    $srcBootConfig = Join-Path $mainDataDir "boot.config"
    $dstBootConfig = Join-Path $dataDir "boot.config"
    if ((Test-Path $srcBootConfig) -and -not (Test-Path $dstBootConfig)) {
        Copy-Item $srcBootConfig $dstBootConfig -Force

        # Ensure single-instance=0
        $content = Get-Content $dstBootConfig -Raw
        if ($content -notmatch "single-instance\s*=\s*0") {
            $content = $content -replace "single-instance\s*=.*", "single-instance=0"
            Set-Content $dstBootConfig -Value $content -Force
        }
        Write-Host "  [+] Created boot.config (single-instance=0)"
    }

    # Copy Steam auth if requested
    if ($SteamAuthSource -and $SteamAuthSource -ne "none") {
        $steamUserdata = Join-Path $boxPath "steam_userdata"
        if (-not (Test-Path $steamUserdata)) {
            New-Item -ItemType Directory -Path $steamUserdata -Force | Out-Null
        }

        # Try to copy from source
        if ($SteamAuthSource -eq "live") {
            $srcSteamUserdata = "$env:APPDATA\Roaming\Steam\userdata"
            if (Test-Path $srcSteamUserdata) {
                Copy-Item $srcSteamUserdata\* $steamUserdata -Recurse -Force -ErrorAction SilentlyContinue
                Write-Host "  [+] Copied Steam userdata"
            }
        } elseif (Test-Path $SteamAuthSource) {
            Copy-Item $SteamAuthSource\* $steamUserdata -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "  [+] Copied Steam userdata from $SteamAuthSource"
        }
    }

    # Create save directory
    $saveDir = Join-Path $boxPath "saves"
    if (-not (Test-Path $saveDir)) {
        New-Item -ItemType Directory -Path $saveDir -Force | Out-Null
        Write-Host "  [+] Created save directory"
    }

    # Store box info in pool
    $pool[$i] = @{
        Index      = $i
        BoxPath    = $boxPath
        PipeName   = $pipeName
        Uuid       = $boxUuid
        ExePath    = $dstExe
        BepInExDir = $bepinexDir
        SaveDir    = $saveDir
        DebugLogPath = Join-Path $bepinexDir "dinoforge_debug.log"
    }

    Write-Host "  [OK] Box ready"
}

Write-Host ""
Write-Host "=== Pool Summary ==="
Write-Host "Created $Count boxes in $BaseDir"
Write-Host ""

foreach ($i in $pool.Keys | Sort-Object) {
    $box = $pool[$i]
    Write-Host "Box $($i):"
    Write-Host "  Path: $($box.BoxPath)"
    Write-Host "  PipeName: $($box.PipeName)"
    Write-Host "  DebugLog: $($box.DebugLogPath)"
}

Write-Host ""
Write-Host "[OK] DINOBox pool created successfully"
Write-Host ""

# Return pool hashtable for use in scripts
$pool
