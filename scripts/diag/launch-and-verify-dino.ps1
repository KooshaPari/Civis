<#
.SYNOPSIS
  Launch DINO and verify startup across 5 progressive health tiers.

.DESCRIPTION
  Closes universal launch-hang detection gap. Each tier proves a progressively
  deeper layer of game readiness, from window paint to ECS bridge pipe health.
  Emits a JSON report and returns exit code 0 (PASS) / 1 (FAIL).

.PARAMETER VerboseDiag
  Emit per-tier diagnostic lines to host stream.

.PARAMETER TimeoutMultiplier
  Scale all per-tier timeouts by this factor (default 1).
#>
[CmdletBinding()]
param(
    [switch]$VerboseDiag,
    [int]$TimeoutMultiplier = 1
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
$GameDir       = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
$GameExe       = Join-Path $GameDir 'Diplomacy is Not an Option.exe'
$BepInExLog    = Join-Path $GameDir 'BepInEx\LogOutput.log'
$DinoForgeLog  = Join-Path $GameDir 'BepInEx\dinoforge_debug.log'
$PipeName      = 'dinoforge-game-bridge'
$ProcessName   = 'Diplomacy is Not an Option'

$T1_Timeout = 30 * $TimeoutMultiplier   # window paint
$T2_Timeout = 60 * $TimeoutMultiplier   # bepinex log mtime
$T3_Timeout = 60 * $TimeoutMultiplier   # dinoforge log activity
$T4_Timeout = 90 * $TimeoutMultiplier   # pipe ready
$T5_Duration = 30 * $TimeoutMultiplier  # health loop window
$T5_Tick     = 5                        # tick interval

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
function Write-Diag {
    param([string]$Message)
    if ($VerboseDiag) {
        Write-Host "[diag] $Message"
    }
}

function Get-DinoProcesses {
    Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
}

function Get-FileMTime {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return $null }
    try {
        return (Get-Item -LiteralPath $Path -Force).LastWriteTime
    } catch {
        return $null
    }
}

function Get-FileSizeSafe {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return $null }
    try {
        return (Get-Item -LiteralPath $Path -Force).Length
    } catch {
        return $null
    }
}

function Test-PipeExists {
    param([string]$Name)
    try {
        $pipes = [System.IO.Directory]::GetFiles('\\.\pipe\')
        foreach ($p in $pipes) {
            if ($p -like "*\$Name") { return $true }
        }
        return $false
    } catch {
        try {
            return Test-Path -LiteralPath "\\.\pipe\$Name"
        } catch {
            return $false
        }
    }
}

# ---------------------------------------------------------------------------
# Report scaffold
# ---------------------------------------------------------------------------
$report = [ordered]@{
    status           = 'UNKNOWN'
    pid              = $null
    launchAt         = $null
    durationSeconds  = 0
    tier1            = [ordered]@{ name='window_paint';      status='SKIPPED'; detail=$null }
    tier2            = [ordered]@{ name='bepinex_log_mtime'; status='SKIPPED'; detail=$null }
    tier3            = [ordered]@{ name='dinoforge_init';    status='SKIPPED'; detail=$null }
    tier4            = [ordered]@{ name='pipe_ready';        status='SKIPPED'; detail=$null }
    tier5            = [ordered]@{ name='health_loop';       status='SKIPPED'; detail=$null }
}

$startWallclock = Get-Date
$failed = $false
$failCode = $null

function Complete-Run {
    param([string]$Status, [string]$FailCode = $null)
    $report.status = $Status
    if ($FailCode) { $report.status = $FailCode }
    $report.durationSeconds = [math]::Round(((Get-Date) - $startWallclock).TotalSeconds, 2)
    $json = $report | ConvertTo-Json -Depth 6
    Write-Output $json
    if ($Status -eq 'PASS') { exit 0 } else { exit 1 }
}

# ---------------------------------------------------------------------------
# Step 1: Stop existing processes
# ---------------------------------------------------------------------------
Write-Diag "Stopping any existing $ProcessName processes..."
try {
    Get-DinoProcesses | ForEach-Object {
        try { Stop-Process -Id $_.Id -Force -ErrorAction Stop } catch { }
    }
} catch { }

Start-Sleep -Seconds 3

$remaining = @(Get-DinoProcesses)
if ($remaining.Count -gt 0) {
    $report.tier1.status = 'FAIL_PRECONDITION'
    $report.tier1.detail = "Could not kill existing processes (count=$($remaining.Count))"
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_PRECONDITION'
}
Write-Diag "No prior DINO processes remain."

# ---------------------------------------------------------------------------
# Step 2: Launch
# ---------------------------------------------------------------------------
if (-not (Test-Path -LiteralPath $GameExe)) {
    $report.tier1.status = 'FAIL_LAUNCH'
    $report.tier1.detail = "Game exe not found: $GameExe"
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_LAUNCH'
}

$launchAt = Get-Date
$report.launchAt = $launchAt.ToString('o')
Write-Diag "Launching $GameExe at $launchAt"

try {
    $proc = Start-Process -FilePath $GameExe -WorkingDirectory $GameDir -PassThru
    $report.pid = $proc.Id
} catch {
    $report.tier1.status = 'FAIL_LAUNCH'
    $report.tier1.detail = "Start-Process failed: $($_.Exception.Message)"
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_LAUNCH'
}

# ---------------------------------------------------------------------------
# Tier 1: Window paint
# ---------------------------------------------------------------------------
Write-Diag "Tier 1: waiting up to ${T1_Timeout}s for MainWindowHandle..."
$t1Start = Get-Date
$t1Ok = $false
$t1Detail = $null
while (((Get-Date) - $t1Start).TotalSeconds -lt $T1_Timeout) {
    $p = Get-DinoProcesses | Select-Object -First 1
    if ($null -eq $p) {
        $t1Detail = 'Process disappeared before window paint'
        break
    }
    try { $p.Refresh() } catch { }
    $title = $null
    try { $title = $p.MainWindowTitle } catch { }
    if ($title) {
        if ($title -match 'Fatal error' -or $title -match 'another instance') {
            $t1Detail = "Bad window title: '$title'"
            $report.tier1.status = 'FAIL_WINDOW'
            $report.tier1.detail = $t1Detail
            Complete-Run -Status 'FAIL' -FailCode 'FAIL_WINDOW'
        }
    }
    $handle = [IntPtr]::Zero
    try { $handle = $p.MainWindowHandle } catch { }
    if ($handle -ne [IntPtr]::Zero) {
        $t1Ok = $true
        $t1Detail = "MainWindowHandle=$handle title='$title' afterSec=$([math]::Round(((Get-Date)-$t1Start).TotalSeconds,2))"
        break
    }
    Start-Sleep -Milliseconds 500
}
if (-not $t1Ok) {
    $report.tier1.status = 'FAIL_WINDOW'
    $report.tier1.detail = ($t1Detail ?? "No MainWindowHandle within ${T1_Timeout}s")
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_WINDOW'
}
$report.tier1.status = 'PASS'
$report.tier1.detail = $t1Detail
Write-Diag "Tier 1 PASS: $t1Detail"

# ---------------------------------------------------------------------------
# Tier 2: BepInEx LogOutput.log mtime advance past $launchAt
# ---------------------------------------------------------------------------
Write-Diag "Tier 2: waiting up to ${T2_Timeout}s for BepInEx log mtime > launchAt..."
$t2Start = Get-Date
$t2Ok = $false
$t2Detail = $null
while (((Get-Date) - $t2Start).TotalSeconds -lt $T2_Timeout) {
    if (Test-Path -LiteralPath $BepInExLog) {
        $mtime = Get-FileMTime -Path $BepInExLog
        if ($mtime -and $mtime -gt $launchAt) {
            $t2Ok = $true
            $t2Detail = "mtime=$($mtime.ToString('o')) launchAt=$($launchAt.ToString('o'))"
            break
        }
    }
    Start-Sleep -Milliseconds 500
}
if (-not $t2Ok) {
    $existingMtime = Get-FileMTime -Path $BepInExLog
    $report.tier2.status = 'FAIL_BEPINEX'
    $report.tier2.detail = "BepInEx log missing or stale (mtime=$existingMtime launchAt=$($launchAt.ToString('o')))"
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_BEPINEX'
}
$report.tier2.status = 'PASS'
$report.tier2.detail = $t2Detail
Write-Diag "Tier 2 PASS: $t2Detail"

# ---------------------------------------------------------------------------
# Tier 3: DINOForge log activity
# ---------------------------------------------------------------------------
Write-Diag "Tier 3: waiting up to ${T3_Timeout}s for dinoforge_debug.log activity..."
$initialDfSize = Get-FileSizeSafe -Path $DinoForgeLog
$initialDfMtime = Get-FileMTime -Path $DinoForgeLog
$t3Start = Get-Date
$t3Ok = $false
$t3Detail = $null
while (((Get-Date) - $t3Start).TotalSeconds -lt $T3_Timeout) {
    if (Test-Path -LiteralPath $DinoForgeLog) {
        $size = Get-FileSizeSafe -Path $DinoForgeLog
        $mtime = Get-FileMTime -Path $DinoForgeLog
        $grew = ($null -ne $initialDfSize -and $size -gt $initialDfSize) -or ($null -eq $initialDfSize -and $size -gt 0)
        $advanced = $mtime -and $mtime -gt $launchAt -and (($null -eq $initialDfMtime) -or ($mtime -gt $initialDfMtime))
        if ($grew -or $advanced) {
            $t3Ok = $true
            $t3Detail = "size=$size mtime=$($mtime.ToString('o')) initialSize=$initialDfSize"
            break
        }
    }
    Start-Sleep -Milliseconds 500
}
if (-not $t3Ok) {
    $report.tier3.status = 'FAIL_DINOFORGE_INIT'
    $report.tier3.detail = "No activity in $DinoForgeLog within ${T3_Timeout}s (initialSize=$initialDfSize)"
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_DINOFORGE_INIT'
}
$report.tier3.status = 'PASS'
$report.tier3.detail = $t3Detail
Write-Diag "Tier 3 PASS: $t3Detail"

# ---------------------------------------------------------------------------
# Tier 4: Named pipe ready
# ---------------------------------------------------------------------------
Write-Diag "Tier 4: waiting up to ${T4_Timeout}s for pipe \\.\pipe\$PipeName..."
$t4Start = Get-Date
$t4Ok = $false
while (((Get-Date) - $t4Start).TotalSeconds -lt $T4_Timeout) {
    if (Test-PipeExists -Name $PipeName) {
        $t4Ok = $true
        break
    }
    Start-Sleep -Milliseconds 500
}
if (-not $t4Ok) {
    $report.tier4.status = 'FAIL_PIPE'
    $report.tier4.detail = "Pipe '$PipeName' not present within ${T4_Timeout}s"
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_PIPE'
}
$report.tier4.status = 'PASS'
$report.tier4.detail = "Pipe '$PipeName' present after $([math]::Round(((Get-Date)-$t4Start).TotalSeconds,2))s"
Write-Diag "Tier 4 PASS: $($report.tier4.detail)"

# ---------------------------------------------------------------------------
# Tier 5: Health loop (30s, tick every 5s)
# ---------------------------------------------------------------------------
Write-Diag "Tier 5: health loop for ${T5_Duration}s (tick=${T5_Tick}s)..."
$t5Start = Get-Date
$ticks = @()
$t5Ok = $true
$t5FailReason = $null
$consecutiveFailures = 0
$maxConsecutiveFailures = 3
$maxObservedConsecutive = 0
while (((Get-Date) - $t5Start).TotalSeconds -lt $T5_Duration) {
    Start-Sleep -Seconds $T5_Tick
    $tickAt = Get-Date
    $p = Get-DinoProcesses | Select-Object -First 1
    if ($null -eq $p) {
        $t5Ok = $false
        $t5FailReason = "Process exited at tick $tickAt"
        break
    }
    try { $p.Refresh() } catch { }
    $responding = $false
    try { $responding = $p.Responding } catch { $responding = $false }
    $mtime = Get-FileMTime -Path $BepInExLog
    $advancedRecent = $false
    if ($mtime) {
        $age = ($tickAt - $mtime).TotalSeconds
        $advancedRecent = ($age -le 10)
    }
    $tickFailed = (-not $responding) -or (-not $advancedRecent)
    $tickFailReason = $null
    if ($tickFailed) {
        $consecutiveFailures++
        if ($consecutiveFailures -gt $maxObservedConsecutive) { $maxObservedConsecutive = $consecutiveFailures }
        if (-not $responding) { $tickFailReason = "not_responding" }
        elseif (-not $advancedRecent) { $tickFailReason = "log_stale_gt_10s" }
    } else {
        $consecutiveFailures = 0
    }
    $tickRecord = [ordered]@{
        at                   = $tickAt.ToString('o')
        responding           = $responding
        bepinexMtime         = if ($mtime) { $mtime.ToString('o') } else { $null }
        mtimeAgeSec          = if ($mtime) { [math]::Round(($tickAt - $mtime).TotalSeconds, 2) } else { $null }
        advancedRecent       = $advancedRecent
        tickFailed           = $tickFailed
        tickFailReason       = $tickFailReason
        consecutiveFailures  = $consecutiveFailures
    }
    $ticks += $tickRecord
    Write-Diag ("Tier 5 tick: responding={0} mtimeAge={1}s advanced={2} consecFails={3}/{4}" -f $responding, $tickRecord.mtimeAgeSec, $advancedRecent, $consecutiveFailures, $maxConsecutiveFailures)
    if ($consecutiveFailures -ge $maxConsecutiveFailures) {
        $t5Ok = $false
        $t5FailReason = "Exceeded $maxConsecutiveFailures consecutive failed ticks (last reason=$tickFailReason) at $tickAt"
        break
    }
}

$report.tier5.detail = [ordered]@{
    ticks                       = $ticks
    failReason                  = $t5FailReason
    maxConsecutiveFailures      = $maxConsecutiveFailures
    maxObservedConsecutive      = $maxObservedConsecutive
    finalConsecutiveFailures    = $consecutiveFailures
}
if (-not $t5Ok) {
    $report.tier5.status = 'FAIL_HEALTH_LOOP'
    Complete-Run -Status 'FAIL' -FailCode 'FAIL_HEALTH_LOOP'
}
$report.tier5.status = 'PASS'
Write-Diag "Tier 5 PASS: $($ticks.Count) ticks recorded"

# ---------------------------------------------------------------------------
# All tiers passed
# ---------------------------------------------------------------------------
Complete-Run -Status 'PASS'
