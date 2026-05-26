<#
.SYNOPSIS
    Launch two concurrent game instances (main + temp) using symlink-based isolation.

.DESCRIPTION
    Launches two independent game instances:
    1. Main instance: original install at G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option
    2. Temp instance: lightweight symlink-based copy in $env:TEMP\DINOForge\instances\

    Both instances can run simultaneously without mutex conflicts (boot.config: single-instance=0).

.PARAMETER WaitForBoth
    Wait for both instances to fully initialize before returning (default: true).

.PARAMETER HideTemp
    Launch temp instance on isolated hidden Win32 desktop (default: false).

.EXAMPLE
    $instances = .\Launch-ConcurrentInstances.ps1
    # Both instances now running independently
    Write-Host "Main logs: $($instances.main.DebugLogPath)"
    Write-Host "Temp logs: $($instances.temp.DebugLogPath)"
#>

param(
    [switch]$WaitForBoth = $true,
    [switch]$HideTemp = $false
)

$ErrorActionPreference = "Stop"

# Get paths
$mainGamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Verify boot.config fix is applied to both main install
Write-Host "Verifying boot.config fixes..."
$bootConfigMain = Split-Path $mainGamePath -Parent | Join-Path -ChildPath "Diplomacy is Not an Option_Data\boot.config"

@($bootConfigMain) | ForEach-Object {
    if (Test-Path $_) {
        $content = Get-Content $_ -Raw
        if ($content -notmatch "single-instance\s*=\s*0") {
            Write-Host "Fixing boot.config: $_"
            $content = $content -replace "single-instance\s*=.*", "single-instance=0"
            Set-Content $_ -Value $content -Force
        }
    }
}
Write-Host "[OK] boot.config verified"

# Kill any existing instances
Write-Host "Cleaning up existing instances..."
Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 3

# Create temp instance
Write-Host "`nCreating lightweight temp instance..."
& "$scriptDir\New-TempGameInstance.ps1" -Verbose | Out-Null
$tempInstance = & "$scriptDir\New-TempGameInstance.ps1"
Write-Host "[OK] Temp instance created at: $($tempInstance.RootDir)"

# Fix boot.config for temp instance
$tempBootConfig = Join-Path $tempInstance.RootDir "Diplomacy is Not an Option_Data\boot.config"
if (Test-Path $tempBootConfig) {
    $content = Get-Content $tempBootConfig -Raw
    if ($content -notmatch "single-instance\s*=\s*0") {
        $content = $content -replace "single-instance\s*=.*", "single-instance=0"
        Set-Content $tempBootConfig -Value $content -Force
        Write-Host "[OK] Fixed temp boot.config"
    }
}

# Launch main instance
Write-Host "`nLaunching main instance..."
$mainProc = Start-Process `
    -FilePath $mainGamePath `
    -WorkingDirectory (Split-Path $mainGamePath -Parent) `
    -PassThru

Write-Host "  PID: $($mainProc.Id)"

# Launch temp instance
Write-Host "Launching temp instance..."
$tempProc = Start-Process `
    -FilePath $tempInstance.GameExePath `
    -WorkingDirectory $tempInstance.WorkingDirectory `
    -PassThru

Write-Host "  PID: $($tempProc.Id)"
Write-Host ""
Write-Host "[OK] Both instances launched"

if ($WaitForBoth) {
    Write-Host "`nWaiting for both instances to initialize (max 30s)..."

    $timeout = 30
    $elapsed = 0

    while ($elapsed -lt $timeout) {
        Start-Sleep -Seconds 1
        $elapsed += 1

        $mainReady = $false
        $tempReady = $false

        # Check main instance log
        $mainLog = "$(Split-Path $mainGamePath -Parent)\BepInEx\dinoforge_debug.log"
        if (Test-Path $mainLog) {
            $mainReady = (Get-Content $mainLog -ErrorAction SilentlyContinue) -match "Awake completed"
        }

        # Check temp instance log
        if (Test-Path $tempInstance.DebugLogPath) {
            $tempReady = (Get-Content $tempInstance.DebugLogPath -ErrorAction SilentlyContinue) -match "Awake completed"
        }

        if ($mainReady -and $tempReady) {
            Write-Host "[OK] Both instances initialized"
            break
        }

        if ($mainReady) { Write-Host "  [OK] Main instance ready" -ForegroundColor Green }
        if ($tempReady) { Write-Host "  [OK] Temp instance ready" -ForegroundColor Green }
    }

    if ($elapsed -ge $timeout) {
        Write-Host "[WARN] Timeout waiting for initialization (check logs manually)"
    }
}

# Output summary
Write-Host ""
Write-Host "=== Instance Summary ==="
Write-Host "Main Instance:"
Write-Host "  Path: $(Split-Path $mainGamePath -Parent)"
Write-Host "  Logs: $mainLog"
Write-Host ""
Write-Host "Temp Instance:"
Write-Host "  Path: $($tempInstance.RootDir)"
Write-Host "  Logs: $($tempInstance.DebugLogPath)"
Write-Host ""
$cleanupCmd = "Remove-Item '$($tempInstance.RootDir)' -Recurse -Force"
Write-Host "Cleanup: $cleanupCmd"

# Return instance info
@{
    main = @{
        ProcessId = $mainProc.Id
        GameExePath = $mainGamePath
        DebugLogPath = $mainLog
    }
    temp = @{
        ProcessId = $tempProc.Id
        GameExePath = $tempInstance.GameExePath
        DebugLogPath = $tempInstance.DebugLogPath
        RootDir = $tempInstance.RootDir
    }
}
