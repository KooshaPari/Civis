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

function Start-ProcessDetached {
    param(
        [string]$FilePath,
        [string[]]$ArgumentList,
        [string]$WorkingDirectory,
        [string]$PidFile,
        [string]$LogFile
    )

    $process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -WorkingDirectory $WorkingDirectory -PassThru -NoNewWindow -WindowStyle Hidden -RedirectStandardOutput $LogFile -RedirectStandardError $LogFile
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

    $watcher = Start-Process -FilePath "pwsh" -ArgumentList @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $watcherScript, "-Watch") -PassThru -NoNewWindow -WindowStyle Hidden
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
    if ($proc) {
        Write-Host "[MCP] Status: running" -ForegroundColor Green
        Write-Host "  PID: $($proc.Id)"
        Write-Host "  Name: $($proc.ProcessName)"
    }
    else {
        Write-Host "[MCP] Status: stopped" -ForegroundColor Yellow
    }

    $listener = Wait-ForTcpPort -ServerHost $McpHost -Port $Port -TimeoutSeconds 1
    $listenerColor = if ($listener) { "Green" } else { "Yellow" }
    Write-Host "  Port ${McpHost}:${Port} listener: $([string]$listener)" -ForegroundColor $listenerColor
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
    Write-Host "[MCP] Server already running on ${McpHost}:${Port}" -ForegroundColor Green
    exit 0
}

$current = Get-ScriptState -PidFile $mcpPidFile -ExpectedName "python|pwsh"
if ($Action -eq "start") {
    if ($current) {
        Write-Host "[MCP] Already running (PID=$($current.Id)). Use -Action restart or -Action stop." -ForegroundColor Yellow
        if ($Watch -and -not (Get-ScriptState -PidFile $watcherPidFile -ExpectedName "pwsh")) {
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

    Write-Host "[MCP] PID file: $mcpPidFile" -ForegroundColor Cyan
    Write-Host "[MCP] Log file: $mcpLogFile" -ForegroundColor Cyan
    Write-Host "[MCP] URL: http://$McpHost`:$Port/messages" -ForegroundColor Cyan
    if ($Watch) {
        Start-WatcherDetached -RepoRoot $repoRoot -PidFile $watcherPidFile | Out-Null
    }
    exit 0
}

if ($Watch) {
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
