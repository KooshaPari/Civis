<#
.SYNOPSIS
    Bring up the full DINOForge dev stack (MCP server + playCUA) in one command.

.DESCRIPTION
    Starts both background services that the proof / verification workflow depends on,
    in the right order, and returns a summary. Idempotent: if a service is already
    running, the existing instance is kept and its PID is reported.

    Order:
      1. MCP server (scripts/start-mcp.ps1 -Action start -Detached)
      2. playCUA  (scripts/start-playcua.ps1 -Listen 127.0.0.1:9000)

    The script does NOT touch the DINO game process (that's a deliberate choice — the
    user controls game launches per CLAUDE.md). After this script succeeds, the user
    can run the first-external-receipt runbook (docs/setup/first-external-receipt-runbook.md).

.PARAMETER PlayCuaListen
    Override the playCUA listen address. Default 127.0.0.1:9000 — the address PlayCUABackend expects.

.PARAMETER McpUrl
    The MCP health endpoint to verify after start. Default http://127.0.0.1:8765/health.

.PARAMETER SkipPlayCua
    Skip starting playCUA. Useful if you only need the MCP server.

.EXAMPLE
    pwsh scripts/dev/full-stack-up.ps1
    Brings up both services on default ports.

.EXAMPLE
    pwsh scripts/dev/full-stack-up.ps1 -SkipPlayCua
    Brings up only the MCP server.
#>
[CmdletBinding()]
param(
    [string]$PlayCuaListen = "127.0.0.1:9000",
    [string]$McpUrl = "http://127.0.0.1:8765/health",
    [switch]$SkipPlayCua
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$results = @()

# 1. MCP server
Write-Host "[1/2] Starting MCP server..."
$mcpScript = Join-Path $repoRoot "scripts\start-mcp.ps1"
if (-not (Test-Path -LiteralPath $mcpScript)) {
    throw "MCP starter script not found: $mcpScript"
}
& $mcpScript -Action start -Detached | Out-Host

Start-Sleep -Seconds 2
try {
    $health = Invoke-RestMethod -Uri $McpUrl -Method Get -TimeoutSec 5 -ErrorAction Stop
    $results += [pscustomobject]@{ Service = "MCP"; Status = "UP"; Detail = $McpUrl }
} catch {
    $results += [pscustomobject]@{ Service = "MCP"; Status = "FAIL"; Detail = "Not reachable at $McpUrl after start" }
}

# 2. playCUA
if ($SkipPlayCua) {
    $results += [pscustomobject]@{ Service = "playCUA"; Status = "SKIPPED"; Detail = "-SkipPlayCua specified" }
} else {
    Write-Host "[2/2] Starting playCUA on $PlayCuaListen..."
    $playcuaScript = Join-Path $repoRoot "scripts\start-playcua.ps1"
    if (-not (Test-Path -LiteralPath $playcuaScript)) {
        $results += [pscustomobject]@{ Service = "playCUA"; Status = "MISSING"; Detail = "$playcuaScript not found" }
    } else {
        try {
            $output = & $playcuaScript -Listen $PlayCuaListen 2>&1
            $pidLine = $output | Where-Object { $_ -match "^PID=(\d+)$" } | Select-Object -First 1
            if ($pidLine) {
                $playcuaPid = [int]($pidLine -replace "^PID=", "")
                $results += [pscustomobject]@{ Service = "playCUA"; Status = "UP"; Detail = "PID=$playcuaPid on $PlayCuaListen" }
            } else {
                $results += [pscustomobject]@{ Service = "playCUA"; Status = "PARTIAL"; Detail = "Started but no PID emitted" }
            }
        } catch {
            $results += [pscustomobject]@{ Service = "playCUA"; Status = "FAIL"; Detail = "$_" }
        }
    }
}

Write-Host ""
$results | Format-Table -AutoSize

$failed = @($results | Where-Object { $_.Status -eq "FAIL" -or $_.Status -eq "MISSING" })
if ($failed.Count -eq 0) {
    Write-Host ""
    Write-Host "Stack is up. Run scripts/proof/preflight-runbook.ps1 to verify all proof prerequisites."
    exit 0
} else {
    Write-Host ""
    Write-Host "Stack startup had $($failed.Count) failure(s). Resolve and re-run."
    exit 1
}
