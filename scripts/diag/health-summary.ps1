<#
.SYNOPSIS
Consolidated single-call health probe: git + deploy + game + MCP + detectors.

.DESCRIPTION
Replaces ad-hoc multi-command health checks. Each section wrapped in try/catch
so partial failures don't kill the rest. Mirrors style of game-state-probe.ps1
and git-state-probe.ps1.

.PARAMETER Json
Output as compact JSON when $true. Default $false = human-readable.

.EXAMPLE
pwsh scripts/diag/health-summary.ps1
pwsh scripts/diag/health-summary.ps1 -Json $true
#>
[CmdletBinding()]
param([bool]$Json = $false)

$ErrorActionPreference = 'SilentlyContinue'
$GamePath = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
$DeployedDll = "$GamePath\BepInEx\plugins\DINOForge.Runtime.dll"
$DebugLog = "$GamePath\BepInEx\dinoforge_debug.log"
$RepoRoot = 'C:\Users\koosh\Dino'

$H = [ordered]@{ timestamp_utc = (Get-Date).ToUniversalTime().ToString('o'); git=@{}; deploy=@{}; game=@{}; mcp=@{}; detectors=@{} }
$AnyFail = $false

# [GIT]
try {
    $H.git.repo_root      = (git -C $RepoRoot rev-parse --show-toplevel 2>$null)
    $H.git.branch         = (git -C $RepoRoot branch --show-current 2>$null)
    $H.git.head           = (git -C $RepoRoot rev-parse --short HEAD 2>$null)
    $H.git.dirty_count    = ((git -C $RepoRoot status --porcelain 2>$null | Measure-Object).Count)
    $H.git.stash_count    = ((git -C $RepoRoot stash list 2>$null | Measure-Object).Count)
    $base = git -C $RepoRoot merge-base HEAD main 2>$null
    if ($base) {
        $H.git.ahead_main  = ((git -C $RepoRoot rev-list "$base..HEAD" 2>$null | Measure-Object).Count)
        $H.git.behind_main = ((git -C $RepoRoot rev-list "HEAD..main" 2>$null | Measure-Object).Count)
    }
    $H.git.status = if ($H.git.branch) { 'OK' } else { 'FAIL' }
} catch { $H.git.status='FAIL'; $H.git.error=$_.Exception.Message; $AnyFail=$true }

# [DEPLOY]
try {
    if (Test-Path $DeployedDll) {
        $info = Get-Item $DeployedDll
        $H.deploy.present  = $true
        $H.deploy.mtime    = $info.LastWriteTime.ToString('o')
        $H.deploy.age_min  = [math]::Round(((Get-Date) - $info.LastWriteTime).TotalMinutes, 1)
        $hash = (Get-FileHash $DeployedDll -Algorithm SHA256).Hash
        $H.deploy.sha8     = $hash.Substring(0, 8)
        $H.deploy.status   = 'OK'
    } else {
        $H.deploy.present=$false; $H.deploy.status='WARN'
    }
} catch { $H.deploy.status='FAIL'; $H.deploy.error=$_.Exception.Message; $AnyFail=$true }

# [GAME]
try {
    $proc = Get-Process -Name 'Diplomacy is Not an Option' -ErrorAction SilentlyContinue
    if ($proc) {
        $H.game.running     = $true
        $H.game.pid_value   = $proc.Id
        $H.game.title       = $proc.MainWindowTitle
        $H.game.responding  = $proc.Responding
    } else {
        $H.game.running = $false
    }
    if (Test-Path $DebugLog) {
        $H.game.log_tail = @(Get-Content $DebugLog -Tail 3 -ErrorAction SilentlyContinue)
    } else {
        $H.game.log_tail = @()
    }
    $H.game.status = 'OK'
} catch { $H.game.status='FAIL'; $H.game.error=$_.Exception.Message; $AnyFail=$true }

# [MCP]
try {
    $resp = Invoke-RestMethod -Uri 'http://127.0.0.1:8765/health' -TimeoutSec 2 -ErrorAction Stop
    $H.mcp.alive = $true
    $H.mcp.tool_count = if ($resp.tool_count) { $resp.tool_count } elseif ($resp.tools) { @($resp.tools).Count } else { $null }
    $H.mcp.status = 'OK'
} catch {
    $H.mcp.alive = $false
    $H.mcp.status = 'WARN'
}

# [DETECTORS]
$detectors = @(
    'scripts/ci/detect_logerror_no_stack.py',
    'scripts/ci/detect_silent_catch.py',
    'scripts/ci/detect_test_pack_leak.py',
    'scripts/ci/detect_graphicraycaster_no_eventsystem.py'
)
foreach ($d in $detectors) {
    $full = Join-Path $RepoRoot $d
    $name = [IO.Path]::GetFileNameWithoutExtension($d)
    try {
        if (-not (Test-Path $full)) { $H.detectors[$name] = 'N/A'; continue }
        $out = & python $full 2>&1 | Out-String
        $highMatches = [regex]::Matches($out, '(?im)^\s*HIGH:?\s*(\d+)|HIGH\s+violations?:\s*(\d+)|(\d+)\s+HIGH\b')
        $high = 0
        foreach ($m in $highMatches) {
            foreach ($g in $m.Groups | Select-Object -Skip 1) {
                if ($g.Success -and $g.Value) { $high = [int]$g.Value; break }
            }
        }
        if ($high -eq 0) {
            $vmatch = [regex]::Match($out, '(?i)(\d+)\s+violations?')
            if ($vmatch.Success) { $high = [int]$vmatch.Groups[1].Value }
        }
        $H.detectors[$name] = $high
    } catch { $H.detectors[$name] = 'FAIL'; $AnyFail=$true }
}

# Verdict
$verdict = 'GREEN'
if ($AnyFail) { $verdict = 'RED' }
elseif ($H.deploy.status -eq 'WARN' -or $H.mcp.status -eq 'WARN') { $verdict = 'YELLOW' }
$H.verdict = $verdict

if ($Json) {
    $H | ConvertTo-Json -Depth 4 -Compress
} else {
    Write-Host "=== DINOForge Health Summary ===" -ForegroundColor Cyan
    Write-Host "[GIT]      [$($H.git.status)] branch=$($H.git.branch) head=$($H.git.head) dirty=$($H.git.dirty_count) stash=$($H.git.stash_count) ahead/behind=$($H.git.ahead_main)/$($H.git.behind_main)"
    if ($H.deploy.present) {
        Write-Host "[DEPLOY]   [$($H.deploy.status)] sha8=$($H.deploy.sha8) age=$($H.deploy.age_min)min mtime=$($H.deploy.mtime)"
    } else {
        Write-Host "[DEPLOY]   [$($H.deploy.status)] DLL not present at $DeployedDll"
    }
    if ($H.game.running) {
        Write-Host "[GAME]     [$($H.game.status)] running=yes pid=$($H.game.pid_value) responding=$($H.game.responding) title='$($H.game.title)'"
    } else {
        Write-Host "[GAME]     [$($H.game.status)] running=no"
    }
    if ($H.game.log_tail.Count -gt 0) { $H.game.log_tail | ForEach-Object { Write-Host "           log| $_" -ForegroundColor DarkGray } }
    Write-Host "[MCP]      [$($H.mcp.status)] alive=$($H.mcp.alive) tools=$($H.mcp.tool_count)"
    Write-Host "[DETECTORS]"
    foreach ($k in $H.detectors.Keys) { Write-Host "           $k = $($H.detectors[$k])" }
    $color = switch ($verdict) { 'GREEN' { 'Green' } 'YELLOW' { 'Yellow' } 'RED' { 'Red' } }
    Write-Host "VERDICT: $verdict" -ForegroundColor $color
}
