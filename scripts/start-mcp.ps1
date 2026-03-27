#!/usr/bin/env pwsh
<#
.SYNOPSIS
Start DINOForge MCP server in HTTP/SSE mode (persistent, survives hot-reload).

.DESCRIPTION
Launches the FastMCP 3.1.1 server on localhost:8765 with HTTP/SSE transport.
This allows the server to stay running while you rebuild and redeploy the DINOForge Runtime DLL,
without interrupting Claude Code's MCP client connection.

Environment variables:
  DINO_GAME_DIR: Path to DINO game installation (default: G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option)
  BARE_CUA_NATIVE: Path to bare-cua-native.exe for screenshot capture
  DINOFORGE_MCP_DEBUG: Set to 1 for verbose logging

Endpoints:
  JSON-RPC: http://127.0.0.1:8765/messages
  SSE: http://127.0.0.1:8765/sse
  HMR: POST http://127.0.0.1:8765/hmr (trigger pack reload notification)

.EXAMPLE
./scripts/start-mcp.ps1
#>

[CmdletBinding()]
param(
    [int]$Port = 8765,
    [string]$Host = "127.0.0.1"
)

# Set environment variables if not already set
if (-not $env:DINO_GAME_DIR) {
    $env:DINO_GAME_DIR = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
}
if (-not $env:BARE_CUA_NATIVE) {
    $env:BARE_CUA_NATIVE = "C:\Users\koosh\bare-cua\target\release\bare-cua-native.exe"
}

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$mcpDir = Join-Path $repoRoot "src\Tools\DinoforgeMcp"

Write-Host "[MCP] Starting DINOForge MCP server in HTTP/SSE mode..." -ForegroundColor Cyan
Write-Host "[MCP] Port: $Port" -ForegroundColor Cyan
Write-Host "[MCP] Game dir: $env:DINO_GAME_DIR" -ForegroundColor Cyan

try {
    Push-Location $mcpDir
    python -m dinoforge_mcp.server --http --port $Port --host $Host
}
catch {
    Write-Host "[MCP] Error starting server: $_" -ForegroundColor Red
    exit 1
}
finally {
    Pop-Location
}
