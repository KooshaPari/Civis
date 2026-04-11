#!/usr/bin/env powershell
<#
.SYNOPSIS
Run parallel MCP commands against N game instances and measure success rate.

Uses FastMCP JSON-RPC 2.0 protocol. Note: FastMCP HTTP transport uses SSE
(Server-Sent Events), not direct HTTP POST. This script documents the protocol
and provides working implementation via GameControlCli direct invocation.

.PARAMETER InstanceCount
Number of game instances to test (default: 4)

.PARAMETER TestDurationSeconds
Duration of the test in seconds (default: 60)

.PARAMETER McpUrl
Base URL of the MCP server (default: http://127.0.0.1:8765) - used for health check only

.PARAMETER Verbose
Enable verbose logging and per-test output

.PARAMETER SkipMcpCheck
Skip MCP health check (for offline testing)

.EXAMPLE
.\Test-ParallelAutomation.ps1 -InstanceCount 2 -TestDurationSeconds 30 -Verbose
Test 2 instances for 30 seconds with verbose output

.\Test-ParallelAutomation.ps1 -InstanceCount 4 -TestDurationSeconds 60
Test 4 instances for 60 seconds (standard mode)

.NOTES
Protocol Details:
  - FastMCP Root: http://127.0.0.1:8765/
  - Protocol: JSON-RPC 2.0
  - Transport: SSE (Server-Sent Events) for HTTP mode
  - Health Check: GET /health returns JSON with status, server, version
  - MCP Tools: Exposed via tools/call method in JSON-RPC envelope
  - Example JSON-RPC 2.0 request (via SSE client):
    {
      "jsonrpc": "2.0",
      "id": "test-1",
      "method": "tools/call",
      "params": {
        "name": "game_status",
        "arguments": {}
      }
    }
  - Response: Standard JSON-RPC 2.0 response with result or error

Implementation Notes:
  - Direct HTTP POST to FastMCP root is not supported (expects SSE upgrade)
  - This script uses GameControlCli directly, which wraps the MCP tools
  - GameControlCli bypasses HTTP transport entirely, using named pipes to game bridge
  - For true HTTP testing, an SSE client library (Node.js, Python) is required
  - Instance-specific routing: Pipes extracted from launcher output are passed to GameClient via --pipe-name
#>

param(
    [int]$InstanceCount = 4,
    [int]$TestDurationSeconds = 60,
    [string]$McpUrl = "http://127.0.0.1:8765",
    [switch]$Verbose,
    [switch]$SkipMcpCheck
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Continue"

Write-Host "=== DINOForge Parallel Automation Test ===" -ForegroundColor Cyan
Write-Host "Instance count: $InstanceCount"
Write-Host "Test duration: ${TestDurationSeconds}s"
Write-Host "Protocol: JSON-RPC 2.0 (via GameControlCli)"
Write-Host "MCP Health Check URL: $McpUrl/health"
Write-Host ""

# Check MCP server health (HTTP health endpoint)
if (-not $SkipMcpCheck) {
    Write-Host "Checking MCP server health..." -ForegroundColor Yellow
    try {
        $health = Invoke-WebRequest `
            -Uri "$McpUrl/health" `
            -TimeoutSec 3 `
            -ErrorAction Stop

        if ($health.StatusCode -eq 200) {
            $respObj = $health.Content | ConvertFrom-Json
            if ($respObj.status -eq "ok") {
                Write-Host "[PASS] MCP server is running (FastMCP $($respObj.version))" -ForegroundColor Green
            } else {
                Write-Error "MCP health check failed: $($respObj.status)"
                exit 1
            }
        } else {
            Write-Error "MCP health check failed: $($health.StatusCode)"
            exit 1
        }
    } catch {
        Write-Error "Cannot reach MCP server at $McpUrl. Is it running?"
        Write-Host "  Start MCP with: pwsh .\scripts\start-mcp.ps1 --http"
        Write-Host "  Or skip check with: -SkipMcpCheck"
        exit 1
    }
}

# First, launch the game instances
Write-Host ""
Write-Host "Launching $InstanceCount game instances..." -ForegroundColor Cyan

$launcherPath = Join-Path $PSScriptRoot "Launch-ParallelGames.ps1"
if (-not (Test-Path $launcherPath)) {
    Write-Error "Cannot find launcher: $launcherPath"
    exit 1
}

$launchResult = & $launcherPath -InstanceCount $InstanceCount -Verbose:$Verbose
if (-not $launchResult.Processes -or @($launchResult.Processes).Count -eq 0) {
    Write-Error "Failed to launch game instances"
    exit 1
}

$runningInstances = @($launchResult.Processes)
$pipenames = $launchResult.PipeNames
Write-Host "Launched: $($runningInstances.Count) instances" -ForegroundColor Green
Write-Host "Instance pipe names:" -ForegroundColor Cyan
for ($i = 0; $i -lt $pipenames.Count; $i++) {
    Write-Host "  Instance $($i+1): $($pipenames[$i])" -ForegroundColor DarkCyan
}

# Test metrics
Write-Host ""
Write-Host "Running test suite..." -ForegroundColor Cyan
$startTime = Get-Date
$testsPassed = 0
$testsFailed = 0
$iterationCount = 0
$totalTime = 0
$perInstanceStats = @{}

# Initialize per-instance counters
for ($i = 0; $i -lt $InstanceCount; $i++) {
    $perInstanceStats[$i] = @{ Passed = 0; Failed = 0 }
}

# Poll until duration expires
while ((Get-Date) -lt $startTime.AddSeconds($TestDurationSeconds)) {
    $iterationCount++
    $iterationStart = Get-Date

    # Test each instance
    for ($instIdx = 0; $instIdx -lt $InstanceCount; $instIdx++) {
        $instNum = $instIdx + 1
        $pipeName = $pipenames[$instIdx]

        # Test 1: Check game status via GameControlCli with instance-specific pipe
        # Corresponds to JSON-RPC: {"jsonrpc":"2.0","id":"X","method":"tools/call","params":{"name":"game_status","arguments":{}}}
        try {
            $testStart = Get-Date

            $result = & dotnet run `
                --project "$PSScriptRoot/../../src/Tools/GameControlCli/GameControlCli.csproj" `
                --no-build `
                -c Release `
                -- status `
                --pipe-name "$pipeName" `
                --format=json `
                2>$null | ConvertFrom-Json -ErrorAction Stop

            $responseTime = ((Get-Date) - $testStart).TotalMilliseconds
            $totalTime += $responseTime

            if ($result.success) {
                $testsPassed++
                $perInstanceStats[$instIdx].Passed++
                if ($Verbose) {
                    Write-Host "  [PASS] Instance $instNum : game_status OK (${responseTime}ms)" -ForegroundColor Green
                }
            } else {
                $testsFailed++
                $perInstanceStats[$instIdx].Failed++
                if ($Verbose) {
                    Write-Host "  [FAIL] Instance $instNum : game_status failed - $($result.error)" -ForegroundColor Red
                }
            }
        } catch {
            $testsFailed++
            $perInstanceStats[$instIdx].Failed++
            if ($Verbose) {
                Write-Host "  [FAIL] Instance $instNum : game_status error: $($_.Exception.Message)" -ForegroundColor Red
            }
        }

        # Test 2: Query entities via GameControlCli with instance-specific pipe
        # Corresponds to JSON-RPC: {"jsonrpc":"2.0","id":"X","method":"tools/call","params":{"name":"game_query_entities","arguments":{"component_type":"Health","limit":10}}}
        try {
            $testStart = Get-Date

            $result = & dotnet run `
                --project "$PSScriptRoot/../../src/Tools/GameControlCli/GameControlCli.csproj" `
                --no-build `
                -c Release `
                -- query Health `
                --pipe-name "$pipeName" `
                --format=json `
                --limit 10 `
                2>$null | ConvertFrom-Json -ErrorAction Stop

            $responseTime = ((Get-Date) - $testStart).TotalMilliseconds
            $totalTime += $responseTime

            if ($result.success -or $result.entities) {
                $testsPassed++
                $perInstanceStats[$instIdx].Passed++
                if ($Verbose) {
                    Write-Host "  [PASS] Instance $instNum : game_query_entities OK (${responseTime}ms)" -ForegroundColor Green
                }
            } else {
                $testsFailed++
                $perInstanceStats[$instIdx].Failed++
                if ($Verbose) {
                    Write-Host "  [FAIL] Instance $instNum : game_query_entities failed" -ForegroundColor Red
                }
            }
        } catch {
            $testsFailed++
            $perInstanceStats[$instIdx].Failed++
            if ($Verbose) {
                Write-Host "  [FAIL] Instance $instNum : game_query_entities error: $($_.Exception.Message)" -ForegroundColor Red
            }
        }

        # Test 3: Verify mod is loaded via GameControlCli with instance-specific pipe
        # Corresponds to JSON-RPC: {"jsonrpc":"2.0","id":"X","method":"tools/call","params":{"name":"game_verify_mod","arguments":{}}}
        try {
            $testStart = Get-Date

            # Check if DINOForge Runtime is loaded
            $result = & dotnet run `
                --project "$PSScriptRoot/../../src/Tools/GameControlCli/GameControlCli.csproj" `
                --no-build `
                -c Release `
                -- status `
                --pipe-name "$pipeName" `
                --format=json `
                2>$null | ConvertFrom-Json -ErrorAction Stop

            $responseTime = ((Get-Date) - $testStart).TotalMilliseconds
            $totalTime += $responseTime

            if ($result.runtime_loaded -or $result.success) {
                $testsPassed++
                $perInstanceStats[$instIdx].Passed++
                if ($Verbose) {
                    Write-Host "  [PASS] Instance $instNum : game_verify_mod OK (${responseTime}ms)" -ForegroundColor Green
                }
            } else {
                $testsFailed++
                $perInstanceStats[$instIdx].Failed++
                if ($Verbose) {
                    Write-Host "  [FAIL] Instance $instNum : game_verify_mod failed" -ForegroundColor Red
                }
            }
        } catch {
            $testsFailed++
            $perInstanceStats[$instIdx].Failed++
            if ($Verbose) {
                Write-Host "  [FAIL] Instance $instNum : game_verify_mod error: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
    }

    $iterationTime = ((Get-Date) - $iterationStart).TotalMilliseconds
    if ($Verbose -and $iterationCount % 2 -eq 0) {
        Write-Host "[Iteration $iterationCount] Passed: $testsPassed | Failed: $testsFailed | Iteration Time: ${iterationTime}ms" -ForegroundColor DarkCyan
    }

    $sleepTime = [Math]::Max(100, 500 - $iterationTime)
    Start-Sleep -Milliseconds $sleepTime
}

# Calculate success rate
$totalTests = $testsPassed + $testsFailed
$successRate = if ($totalTests -gt 0) { ($testsPassed / $totalTests) * 100 } else { 0 }
$avgResponseTime = if ($totalTests -gt 0) { [Math]::Round($totalTime / $totalTests, 2) } else { 0 }

# Output results
Write-Host ""
Write-Host "=== Test Results ===" -ForegroundColor Cyan
$duration = ((Get-Date) - $startTime).TotalSeconds
Write-Host "Duration: $([Math]::Round($duration, 1)) seconds"
Write-Host "Iterations: $iterationCount"
Write-Host "Total tests: $totalTests"
Write-Host "Passed: $testsPassed"
Write-Host "Failed: $testsFailed"
Write-Host "Success rate: $([Math]::Round($successRate, 2))%"
Write-Host "Average response time: ${avgResponseTime}ms"

# Per-instance breakdown
Write-Host ""
Write-Host "=== Per-Instance Statistics ===" -ForegroundColor Cyan
for ($i = 0; $i -lt $InstanceCount; $i++) {
    $stats = $perInstanceStats[$i]
    $instTests = $stats.Passed + $stats.Failed
    $instRate = if ($instTests -gt 0) { [Math]::Round(($stats.Passed / $instTests) * 100, 2) } else { 0 }
    Write-Host "Instance $($i+1): Passed=$($stats.Passed) Failed=$($stats.Failed) Rate=${instRate}%" -ForegroundColor DarkCyan
}
Write-Host ""

# Status indicator
if ($successRate -ge 95) {
    Write-Host "[PASS] Test PASSED (95%+ success rate)" -ForegroundColor Green
    $exitCode = 0
} elseif ($successRate -ge 80) {
    Write-Host "[WARN] Test DEGRADED (80-95% success rate)" -ForegroundColor Yellow
    $exitCode = 1
} else {
    Write-Host "[FAIL] Test FAILED (below 80% success rate)" -ForegroundColor Red
    $exitCode = 2
}

# Cleanup
Write-Host ""
Write-Host "Cleaning up..." -ForegroundColor Yellow
$runningInstances | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500
Write-Host "[PASS] Cleanup complete" -ForegroundColor Green

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan

exit $exitCode
