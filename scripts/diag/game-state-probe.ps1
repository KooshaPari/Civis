<#
.SYNOPSIS
  Autonomous game-state diagnostics for DINOForge orchestrator sessions.
  Replaces AskUserQuestion by self-probing critical state vectors.
.DESCRIPTION
  8 independent probes, output structured JSON + terse table.
  All wrapped in try/catch; partial failures do not break the script.
#>

param([switch]$Json)

$GamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$LogPath = "$GamePath\BepInEx"
$DebugLog = "$LogPath\dinoforge_debug.log"
$BepInExLog = "$LogPath\LogOutput.log"
$BinPath = "C:\Users\koosh\Dino\src\Runtime\bin\Release"

$Results = @{}

# Probe 1: Game process running
try {
    $proc = Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
    $Results.GameRunning = $proc -ne $null
    $Results.GamePid = if ($proc) { $proc.Id } else { $null }
} catch { $Results.GameRunning = "ERROR"; $Results.GameError = $_.Message }

# Probe 2: Plugin DLL state (enabled/disabled/missing)
try {
    $dll = Get-Item "$LogPath\plugins\DINOForge.Runtime.dll" -ErrorAction SilentlyContinue
    $disabled = Get-Item "$LogPath\plugins\DINOForge.Runtime.dll.disabled" -ErrorAction SilentlyContinue
    if ($disabled) { $Results.PluginState = "disabled" }
    elseif ($dll) { $Results.PluginState = "enabled" }
    else { $Results.PluginState = "missing" }
} catch { $Results.PluginState = "ERROR"; $Results.PluginStateError = $_.Message }

# Probe 3: Deploy hash (compare deployed vs bin/Release)
try {
    $deployed = Get-Item "$LogPath\plugins\DINOForge.Runtime.dll" -ErrorAction SilentlyContinue
    $built = Get-Item "$BinPath\DINOForge.Runtime.dll" -ErrorAction SilentlyContinue
    if ($deployed -and $built) {
        $depHash = (Get-FileHash $deployed.FullName).Hash
        $buildHash = (Get-FileHash $built.FullName).Hash
        $Results.DeployMatch = ($depHash -eq $buildHash)
        $Results.DeployedTime = $deployed.LastWriteTime
        $Results.BuiltTime = $built.LastWriteTime
    } else {
        $Results.DeployMatch = $null
    }
} catch { $Results.DeployMatch = "ERROR"; $Results.DeployHashError = $_.Message }

# Probe 4: Plugin load confirmation (recent log scan)
try {
    if (Test-Path $DebugLog) {
        $tail = Get-Content $DebugLog -Tail 200 -ErrorAction SilentlyContinue
        $Results.PluginLoaded = ($tail -match "Plugin loaded" -or $tail -match "DINOForge.*loaded").Count -gt 0
        $Results.LastLogLine = ($tail | Select-Object -Last 1) -replace '^.{0,80}', ''
    } else {
        $Results.PluginLoaded = $false
    }
} catch { $Results.PluginLoaded = "ERROR"; $Results.PluginLoadError = $_.Message }

# Probe 5: Pack load count
try {
    if (Test-Path $DebugLog) {
        $tail = Get-Content $DebugLog -Tail 500 -ErrorAction SilentlyContinue
        $match = $tail | Select-String "Loaded (\d+) pack" | Select-Object -Last 1
        $Results.PackCount = if ($match) { [int]($match.Matches.Groups[1].Value) } else { 0 }
    }
} catch { $Results.PackCount = "ERROR" }

# Probe 6: Entity count in Default World
try {
    if (Test-Path $DebugLog) {
        $tail = Get-Content $DebugLog -Tail 500 -ErrorAction SilentlyContinue
        $match = $tail | Select-String "(\d+).*entit.*Default World" | Select-Object -Last 1
        $Results.EntityCount = if ($match) { [int]($match.Matches.Groups[1].Value) } else { $null }
    }
} catch { $Results.EntityCount = "ERROR" }

# Probe 7: Recent errors (last 5min of logs)
try {
    if (Test-Path $DebugLog) {
        $tail = Get-Content $DebugLog -Tail 300 -ErrorAction SilentlyContinue
        $errors = @($tail | Select-String "ERROR|Exception|NullReference|FATAL" -ErrorAction SilentlyContinue)
        $Results.RecentErrors = $errors.Count
        $Results.LastError = if ($errors) { $errors[-1].Line -replace '^.{0,80}', '' } else { $null }
    }
} catch { $Results.RecentErrors = "ERROR" }

# Probe 8: World readiness (ECS active)
try {
    if (Test-Path $DebugLog) {
        $tail = Get-Content $DebugLog -Tail 200 -ErrorAction SilentlyContinue
        $Results.WorldReady = ($tail -match "World initialized|ECS.*ready").Count -gt 0
    }
} catch { $Results.WorldReady = "ERROR" }

# Output
if ($Json) {
    $Results | ConvertTo-Json -Depth 2
} else {
    Write-Host "=== DINOForge Game State Probe ===" -ForegroundColor Cyan
    Write-Host "Game Running: $(if($Results.GameRunning) {'✓ Yes'} else {'✗ No'}) (PID: $($Results.GamePid))"
    Write-Host "Plugin State: $($Results.PluginState) | Loaded: $(if($Results.PluginLoaded) {'✓'} else {'✗'})"
    Write-Host "Deploy Match: $(if($Results.DeployMatch -eq $true) {'✓'} elseif($Results.DeployMatch -eq $false) {'✗ MISMATCH'} else {'N/A'})"
    Write-Host "Packs: $($Results.PackCount) | Entities: $($Results.EntityCount)"
    Write-Host "Recent Errors: $($Results.RecentErrors) | World Ready: $(if($Results.WorldReady) {'✓'} else {'?'})"
    if ($Results.LastError) {
        Write-Host "Last Error: $($Results.LastError)" -ForegroundColor Yellow
    }
}
