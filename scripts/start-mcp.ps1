#!/usr/bin/env pwsh
<#
.SYNOPSIS
Manage the DINOForge MCP server process (HTTP/SSE transport) and optional hot-reload watcher.

.DESCRIPTION
Start, stop, restart, or query status for the FastMCP 3.x server.

Preferred workflow is HTTP mode with a managed background process:
- `-Detached` keeps MCP running across shell exits (for IDE/CC startup hooks).
- `-Watch` starts `scripts/game/hot-reload.ps1 -Watch` in a companion process.
- `-Action` exposes lifecycle control (`start`, `stop`, `restart`, `status`).

Environment variables:
  DINO_GAME_DIR: Path to DINO game installation (default: G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option)
  BARE_CUA_NATIVE: Path to bare-cua-native.exe for screenshot capture
  DINOFORGE_MCP_DEBUG: Set to 1 for verbose logging
  DINOFORGE_PYTHON: Optional path to python executable (default: python in PATH)

Endpoints:
  JSON-RPC: http://127.0.0.1:8765/messages
  SSE: http://127.0.0.1:8765/sse
  HMR: POST http://127.0.0.1:8765/hmr (trigger pack reload notification)

.EXAMPLE
./scripts/start-mcp.ps1 -Detached -Watch
#>

[CmdletBinding()]
param(
    [ValidateSet("start", "stop", "status", "restart")]
    [string]$Action = "start",
    [int]$Port = 8765,
    [string]$McpHost = "127.0.0.1",
    [switch]$Detached,
    [switch]$Watch
)
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$HealthProbeUrl = "http://$McpHost`:$Port/health"
$effectiveWatch = $Watch -or ($env:DINOFORGE_MCP_WATCH -in @("1", "true", "True", "TRUE", "on", "ON"))

function Get-ScriptState {
    param([string]$PidFile, [string]$ExpectedName)

    if (-not (Test-Path $PidFile)) {
        return $null
    }

    try {
        $pidText = Get-Content -Path $PidFile -Raw -ErrorAction Stop
        $pid = [int]($pidText.Trim())
        $process = Get-Process -Id $pid -ErrorAction Stop
        if ($ExpectedName -and $process.ProcessName -notmatch $ExpectedName) {
            return $null
        }

        return $process
    }
    catch {
        Remove-Item -Path $PidFile -Force -ErrorAction SilentlyContinue
        return $null
    }
}

function Wait-ForTcpPort {
    param([string]$ServerHost, [int]$Port, [int]$TimeoutSeconds = 20)
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $client = [System.Net.Sockets.TcpClient]::new()
        try {
            $async = $client.BeginConnect($ServerHost, $Port, $null, $null)
            if ($async.AsyncWaitHandle.WaitOne(250)) {
                $client.EndConnect($async)
                $client.Close()
                return $true
            }
        }
        catch { }
        finally {
            $client.Close()
        }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

function Test-HttpHealth {
    param([string]$Url)
    try {
        $response = Invoke-WebRequest -UseBasicParsing -Uri $Url -TimeoutSec 1 -Method GET -ErrorAction Stop
        return $response.StatusCode -eq 200
    }
    catch {
        return $false
    }
}

function Wait-ForHttpHealth {
    param([string]$Url, [int]$TimeoutSeconds = 20)
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (Test-HttpHealth -Url $Url) {
            return $true
        }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

function Start-ProcessDetached {
    param(
        [string]$FilePath,
        [string[]]$ArgumentList,
        [string]$WorkingDirectory,
        [string]$PidFile,
        [string]$LogFile
    )

    $process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -WorkingDirectory $WorkingDirectory -PassThru -WindowStyle Hidden -RedirectStandardOutput $LogFile -RedirectStandardError $LogFile
    $process.Id | Set-Content -Path $PidFile
    Write-Host "[MCP] Started detached process PID=$($process.Id)" -ForegroundColor Cyan
    return $process
}

function Start-WatcherDetached {
    param([string]$RepoRoot, [string]$PidFile)

    $watcherScript = Join-Path $RepoRoot "scripts\game\hot-reload.ps1"
    if (-not (Test-Path $watcherScript)) {
        Write-Host "[MCP] Hot-reload watcher script not found: $watcherScript" -ForegroundColor Yellow
        return $null
    }

    $existing = Get-ScriptState -PidFile $PidFile -ExpectedName "pwsh"
    if ($existing) {
        Write-Host "[MCP] Hot-reload watcher already running (PID=$($existing.Id))" -ForegroundColor Green
        return $existing
    }

    $watcherLogFile = Join-Path $runtimeStateDir "mcp-hot-reload-watcher.log"
    $watcher = Start-Process -FilePath (Get-Command pwsh).Source -ArgumentList @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $watcherScript, "-Watch") -PassThru -WindowStyle Hidden -RedirectStandardOutput $watcherLogFile -RedirectStandardError $watcherLogFile
    $watcher.Id | Set-Content -Path $PidFile
    Write-Host "[MCP] Started hot-reload watcher PID=$($watcher.Id)" -ForegroundColor Cyan
    return $watcher
}

function Stop-ProcessByPidFile {
    param([string]$PidFile)

    $process = Get-ScriptState -PidFile $PidFile
    if ($process) {
        Stop-Process -Id $process.Id -ErrorAction SilentlyContinue
        $process.WaitForExit(5000) | Out-Null
        Remove-Item -Path $PidFile -Force -ErrorAction SilentlyContinue
        return $process.Id
    }
    return $null
}

# Environment setup
if (-not $env:DINO_GAME_DIR) {
    $env:DINO_GAME_DIR = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
}
if (-not $env:BARE_CUA_NATIVE) {
    $env:BARE_CUA_NATIVE = "C:\Users\koosh\bare-cua\target\release\bare-cua-native.exe"
}

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$mcpDir = Join-Path $repoRoot "src\Tools\DinoforgeMcp"
$runtimeStateDir = Join-Path $env:TEMP "DINOForge"
$mcpPidFile = Join-Path $runtimeStateDir "mcp-server.pid"
$watcherPidFile = Join-Path $runtimeStateDir "mcp-hot-reload-watcher.pid"
$mcpLogFile = Join-Path $runtimeStateDir "mcp-server.log"

New-Item -ItemType Directory -Path $runtimeStateDir -Force | Out-Null

if ($Action -eq "status") {
    $proc = Get-ScriptState -PidFile $mcpPidFile -ExpectedName "python|pwsh"
    $listener = Wait-ForTcpPort -ServerHost $McpHost -Port $Port -TimeoutSeconds 1
    $health = Test-HttpHealth -Url $HealthProbeUrl

    if ($proc) {
        Write-Host "[MCP] Status: running" -ForegroundColor Green
        Write-Host "  PID: $($proc.Id)"
        Write-Host "  Name: $($proc.ProcessName)"
    }
    elseif ($listener -and $health) {
        Write-Host "[MCP] Status: running (untracked process)" -ForegroundColor Yellow
    }
    elseif ($listener) {
        Write-Host "[MCP] Status: port in use, health failed" -ForegroundColor Yellow
    }
    else {
        Write-Host "[MCP] Status: stopped" -ForegroundColor Yellow
    }

    $listenerColor = if ($listener) { "Green" } else { "Yellow" }
    $healthColor = if ($health) { "Green" } else { "Yellow" }
    Write-Host "  Port ${McpHost}:${Port} listener: $([string]$listener)" -ForegroundColor $listenerColor
    Write-Host "  Health /health: $([string]$health)" -ForegroundColor $healthColor
    exit
}

if ($Action -in @("stop", "restart")) {
    $stoppedMcp = Stop-ProcessByPidFile -PidFile $mcpPidFile
    if ($stoppedMcp) {
        Write-Host "[MCP] Stopped MCP PID $stoppedMcp" -ForegroundColor Green
    }
    else {
        Write-Host "[MCP] MCP is already stopped" -ForegroundColor Yellow
    }

    $stoppedWatch = Stop-ProcessByPidFile -PidFile $watcherPidFile
    if ($stoppedWatch) {
        Write-Host "[MCP] Stopped hot-reload watcher PID $stoppedWatch" -ForegroundColor Green
    }

    if ($Action -eq "stop") {
        exit 0
    }
}

# Idempotent check: exit if already running on port
$portListener = Wait-ForTcpPort -ServerHost $McpHost -Port $Port -TimeoutSeconds 1
if ($portListener) {
    if (Test-HttpHealth -Url $HealthProbeUrl) {
        Write-Host "[MCP] Server already running on ${McpHost}:${Port} with active /health" -ForegroundColor Green
        exit 0
    }

    if (Get-ScriptState -PidFile $mcpPidFile -ExpectedName "python|pwsh") {
        Write-Host "[MCP] Port ${McpHost}:${Port} open and PID file appears owned, but /health is not responding yet. Retry startup in 2s..." -ForegroundColor Yellow
        Start-Sleep 2
        if (Test-HttpHealth -Url $HealthProbeUrl) {
            Write-Host "[MCP] Health is online after warmup; MCP is ready." -ForegroundColor Green
            exit 0
        }
    }

    Write-Host "[MCP] Port ${McpHost}:${Port} is already open but does not look like MCP health endpoint. Abort to avoid overriding active service." -ForegroundColor Red
    exit 1
}

$current = Get-ScriptState -PidFile $mcpPidFile -ExpectedName "python|pwsh"
if ($Action -eq "start") {
    if ($current) {
        Write-Host "[MCP] Already running (PID=$($current.Id)). Use -Action restart or -Action stop." -ForegroundColor Yellow
        if ($effectiveWatch -and -not (Get-ScriptState -PidFile $watcherPidFile -ExpectedName "pwsh")) {
            Start-WatcherDetached -RepoRoot $repoRoot -PidFile $watcherPidFile | Out-Null
        }
        exit 0
    }
}

$pythonExe = if ($env:DINOFORGE_PYTHON) { $env:DINOFORGE_PYTHON } else { "python" }
if (-not (Get-Command $pythonExe -ErrorAction SilentlyContinue)) {
    Write-Host "[MCP] python executable not found: $pythonExe" -ForegroundColor Red
    exit 1
}

Write-Host "[MCP] Starting DINOForge MCP server (HTTP/SSE)..." -ForegroundColor Cyan
Write-Host "[MCP] Port: $Port" -ForegroundColor Cyan
Write-Host "[MCP] Host: $McpHost" -ForegroundColor Cyan
Write-Host "[MCP] Game dir: $env:DINO_GAME_DIR" -ForegroundColor Cyan

$arguments = @(
    "-m", "dinoforge_mcp.server",
    "--http",
    "--port", $Port.ToString(),
    "--host", $McpHost
)

if ($Detached) {
    $mcpProcess = Start-ProcessDetached -FilePath $pythonExe -ArgumentList $arguments -WorkingDirectory $mcpDir -PidFile $mcpPidFile -LogFile $mcpLogFile
    Start-Sleep -Seconds 1
    if (-not (Wait-ForTcpPort -ServerHost $McpHost -Port $Port -TimeoutSeconds 15)) {
        Write-Host "[MCP] Warning: port did not open within timeout. Check log: $mcpLogFile" -ForegroundColor Yellow
        exit 1
    }
    if (-not (Wait-ForHttpHealth -Url $HealthProbeUrl -TimeoutSeconds 15)) {
        Write-Host "[MCP] Warning: MCP process started but /health probe did not respond within timeout. Check log: $mcpLogFile" -ForegroundColor Yellow
        exit 1
    }

    Write-Host "[MCP] PID file: $mcpPidFile" -ForegroundColor Cyan
    Write-Host "[MCP] Log file: $mcpLogFile" -ForegroundColor Cyan
    Write-Host "[MCP] JSON-RPC endpoint: http://$McpHost`:$Port/messages" -ForegroundColor Cyan
    Write-Host "[MCP] SSE endpoint: http://$McpHost`:$Port/sse" -ForegroundColor Cyan
    Write-Host "[MCP] Health endpoint: $HealthProbeUrl" -ForegroundColor Cyan
    if ($effectiveWatch) {
        Start-WatcherDetached -RepoRoot $repoRoot -PidFile $watcherPidFile | Out-Null
    }
    exit 0
}

if ($Watch -or $effectiveWatch) {
    Write-Host "[MCP] WARNING: -Watch requires -Detached when combined with foreground MCP start." -ForegroundColor Yellow
    Write-Host "[MCP] Re-run with -Detached -Watch to keep both processes alive." -ForegroundColor Yellow
}

try {
    Push-Location $mcpDir
    & $pythonExe @arguments
}
catch {
    Write-Host "[MCP] Error starting server: $_" -ForegroundColor Red
    exit 1
}
finally {
    Pop-Location
}
