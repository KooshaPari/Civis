#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Multi-step live-bridge journey screenshot capture for phenotype visual evidence.

.DESCRIPTION
    Verifies bridge readiness (pipe, MCP, GameControlCli status), then captures 2+ indexed
    PNGs under docs/qa/evidence/live-bridge-journey_<date>/steps/ (step-000.png, ...).

    Step 0: baseline in-game frame
    Step 1: debug panel open (toggle-ui debug) for visible UI contrast
    Step 2: debug closed, second world frame

    - With a live bridge: writes capture-receipt.json + step PNGs
    - Without bridge / failed status: records skip reason in receipt; exit 1

.EXAMPLE
    pwsh -File scripts/qa/live-bridge-journey-capture.ps1
    pwsh -File scripts/qa/live-bridge-journey-capture.ps1 -EvidenceDate 2026-05-23
#>
[CmdletBinding()]
param(
    [string]$EvidenceDate = (Get-Date -Format 'yyyy-MM-dd'),
    [int]$MinSteps = 2,
    [int]$SettleMs = 600,
    [int]$BridgeWaitSec = 120,
    [int]$BridgePollSec = 8,
    [int]$ConnectRetries = 8,
    [int]$RetryDelayMs = 1500,
    [int]$PostConnectCooldownMs = 2000
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $repoRoot

$gameControlProj = Join-Path $repoRoot 'src\Tools\GameControlCli\GameControlCli.csproj'
$gameControlDll = Join-Path $repoRoot 'src\Tools\GameControlCli\bin\Release\net11.0\game-control.dll'
$evidenceDir = Join-Path $repoRoot "docs\qa\evidence\live-bridge-journey_$EvidenceDate"
$stepsDir = Join-Path $evidenceDir 'steps'
$receiptPath = Join-Path $evidenceDir 'capture-receipt.json'
$mcpHealthUrl = 'http://127.0.0.1:8765/health'
$pipePath = '\\.\pipe\dinoforge-game-bridge'

Write-Host 'Building GameControlCli (Release)...' -ForegroundColor DarkGray
$buildOut = & dotnet build $gameControlProj -c Release --nologo -v q 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host ($buildOut | Out-String)
    throw "GameControlCli build failed (exit $LASTEXITCODE)"
}
$useExec = Test-Path -LiteralPath $gameControlDll

function Invoke-GameControlCli {
    param([Parameter(Mandatory)][string[]]$CliArgs)
    if ($useExec) {
        # dotnet exec: application args follow the DLL path (no "--" separator)
        $out = & dotnet exec $gameControlDll @CliArgs 2>&1
    }
    else {
        $out = & dotnet run --project $gameControlProj -c Release -- @CliArgs 2>&1
    }
    $code = $LASTEXITCODE
    return @{ ExitCode = $code; Output = ($out | Out-String).Trim() }
}

function Invoke-GameControlCliWithRetry {
    param(
        [Parameter(Mandatory)][string[]]$CliArgs,
        [int]$MaxAttempts = $ConnectRetries,
        [int]$DelayMs = $RetryDelayMs
    )
    $last = $null
    for ($i = 1; $i -le $MaxAttempts; $i++) {
        $last = Invoke-GameControlCli -CliArgs $CliArgs
        if ($last.ExitCode -eq 0) {
            return $last
        }
        $connFail = $last.Output -match 'Failed to connect|Connection timeout'
        if (-not $connFail -or $i -eq $MaxAttempts) {
            return $last
        }
        Start-Sleep -Milliseconds $DelayMs
    }
    return $last
}

function Test-BridgeStatusOk {
    $status = Invoke-GameControlCliWithRetry -CliArgs @('status')
    $ok = ($status.ExitCode -eq 0) -and ($status.Output -match 'Connected to game bridge')
    return @{ Ok = $ok; Result = $status }
}

function Wait-BridgeReady {
    param([int]$TimeoutSec, [int]$PollSec)
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    $attempt = 0
    while ((Get-Date) -lt $deadline) {
        $attempt++
        $probe = Test-BridgeStatusOk
        if ($probe.Ok) {
            return @{ Ready = $true; Attempt = $attempt; Status = $probe.Result }
        }
        Start-Sleep -Seconds $PollSec
    }
    $last = Test-BridgeStatusOk
    return @{
        Ready   = $last.Ok
        Attempt = $attempt
        Status  = $last.Result
    }
}

function Step-PngPath {
    param([int]$Index)
    Join-Path $stepsDir ("step-{0:D3}.png" -f $Index)
}

New-Item -ItemType Directory -Force -Path $stepsDir | Out-Null

$steps = [System.Collections.Generic.List[object]]::new()
$overallPass = $true
$capturedPaths = [System.Collections.Generic.List[string]]::new()

function Add-Step {
    param(
        [string]$Name,
        [bool]$Pass,
        [string]$Detail,
        [hashtable]$Extra = @{}
    )
    if (-not $Pass) { $script:overallPass = $false }
    $row = [ordered]@{
        step   = $Name
        pass   = $Pass
        detail = $Detail
        at     = (Get-Date).ToString('o')
    }
    foreach ($k in $Extra.Keys) { $row[$k] = $Extra[$k] }
    $steps.Add([pscustomobject]$row) | Out-Null
}

function Capture-IndexedScreenshot {
    param(
        [int]$Index,
        [string]$Slug,
        [object[]]$PreArgs = @()
    )
    $path = Step-PngPath -Index $Index
    $bridge = Test-BridgeStatusOk
    if (-not $bridge.Ok) {
        Add-Step -Name "capture_bridge_$Index" -Pass $false -Detail 'bridge not ready before capture' -Extra @{
            slug = $Slug
        }
        return $false
    }
    Start-Sleep -Milliseconds $PostConnectCooldownMs
    foreach ($prep in $PreArgs) {
        if ($null -eq $prep) { continue }
        $cliArgs = if ($prep -is [System.Array]) { [string[]]$prep } else { @([string]$prep) }
        if ($cliArgs.Count -eq 0) { continue }
        $pre = Invoke-GameControlCliWithRetry -CliArgs $cliArgs
        if ($pre.ExitCode -ne 0) {
            Add-Step -Name "capture_prep_$Index" -Pass $false -Detail $pre.Output -Extra @{
                slug      = $Slug
                exit_code = $pre.ExitCode
            }
            return $false
        }
        Start-Sleep -Milliseconds $SettleMs
    }
    $ss = Invoke-GameControlCliWithRetry -CliArgs @('screenshot', $path)
    $ok = ($ss.ExitCode -eq 0) -and (Test-Path -LiteralPath $path)
    Add-Step -Name "capture_$Index" -Pass $ok -Detail $(if ($ok) { $path } else { $ss.Output }) -Extra @{
        slug             = $Slug
        screenshot_index = $Index
        screenshot_path  = $(if ($ok) { $path } else { $null })
        exit_code        = $ss.ExitCode
    }
    if ($ok) { $script:capturedPaths.Add($path) | Out-Null }
    return $ok
}

# Preflight: DINO_GAME_PATH (informational)
$dinoGamePath = [Environment]::GetEnvironmentVariable('DINO_GAME_PATH', 'Process')
if ([string]::IsNullOrWhiteSpace($dinoGamePath)) {
    $dinoGamePath = [Environment]::GetEnvironmentVariable('DINO_GAME_PATH', 'User')
}
$gamePathSet = -not [string]::IsNullOrWhiteSpace($dinoGamePath)
Add-Step -Name 'dino_game_path' -Pass $true -Detail $(if ($gamePathSet) { $dinoGamePath } else { 'unset — bridge checks only' })

$pipePresent = Test-Path -LiteralPath $pipePath
Add-Step -Name 'named_pipe' -Pass $pipePresent -Detail $(if ($pipePresent) { $pipePath } else { 'pipe not found' })

try {
    $health = Invoke-RestMethod -Uri $mcpHealthUrl -Method Get -TimeoutSec 5
    Add-Step -Name 'mcp_health' -Pass $true -Detail ($health | ConvertTo-Json -Compress)
}
catch {
    Add-Step -Name 'mcp_health' -Pass $false -Detail $_.Exception.Message
}

$bridgeWait = Wait-BridgeReady -TimeoutSec $BridgeWaitSec -PollSec $BridgePollSec
$status = $bridgeWait.Status
$statusOk = $bridgeWait.Ready
Add-Step -Name 'bridge_wait' -Pass $statusOk -Detail $(if ($statusOk) {
    "ready after $($bridgeWait.Attempt) attempt(s) within ${BridgeWaitSec}s"
} else {
    "not ready within ${BridgeWaitSec}s (last exit=$($status.ExitCode))"
}) -Extra @{ attempts = $bridgeWait.Attempt; wait_sec = $BridgeWaitSec }
Add-Step -Name 'game_control_cli_status' -Pass $statusOk -Detail $status.Output -Extra @{ exit_code = $status.ExitCode }

if (-not $statusOk) {
    Add-Step -Name 'capture_skipped' -Pass $false -Detail 'skipped — status check did not pass (game bridge not available)'
    $receipt = [ordered]@{
        schema           = 'dinoforge.live-bridge-journey-capture/v1'
        observed_at      = (Get-Date).ToString('o')
        evidence_dir     = $evidenceDir
        steps_dir        = $stepsDir
        min_steps        = $MinSteps
        captured_count   = 0
        dino_game_path_set = $gamePathSet
        overall_pass     = $false
        steps            = $steps
    }
    $receipt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $receiptPath -Encoding utf8
    Write-Host "Live bridge journey capture: SKIPPED (bridge unavailable)" -ForegroundColor Yellow
    Write-Host "Receipt: $receiptPath"
    exit 1
}

Start-Sleep -Milliseconds $PostConnectCooldownMs

# Indexed captures (align with example-visual-evidence manifest slugs)
$null = Capture-IndexedScreenshot -Index 0 -Slug 'world-day'
$null = Capture-IndexedScreenshot -Index 1 -Slug 'lighting-variant' -PreArgs @(,@('toggle-ui', 'debug'))
$null = Capture-IndexedScreenshot -Index 2 -Slug 'material-terrain' -PreArgs @(,@('toggle-ui', 'debug'))

$capturedCount = $capturedPaths.Count
$meetsMin = $capturedCount -ge $MinSteps
if (-not $meetsMin) { $overallPass = $false }

Add-Step -Name 'capture_minimum' -Pass $meetsMin -Detail "$capturedCount of $MinSteps required step PNG(s) on disk"

$receipt = [ordered]@{
    schema             = 'dinoforge.live-bridge-journey-capture/v1'
    observed_at        = (Get-Date).ToString('o')
    evidence_dir         = $evidenceDir
    steps_dir            = $stepsDir
    min_steps            = $MinSteps
    captured_count       = $capturedCount
    screenshot_paths     = @($capturedPaths)
    dino_game_path_set   = $gamePathSet
    overall_pass         = $overallPass
    steps                = $steps
}

$receipt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $receiptPath -Encoding utf8

Write-Host "Live bridge journey capture: overall_pass=$overallPass captured=$capturedCount" -ForegroundColor $(if ($overallPass) { 'Green' } else { 'Red' })
Write-Host "Receipt: $receiptPath"
foreach ($p in $capturedPaths) { Write-Host "  $p" }

exit $(if ($overallPass) { 0 } else { 1 })
