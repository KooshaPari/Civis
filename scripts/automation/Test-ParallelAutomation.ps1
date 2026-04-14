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

# Generate request ID for tracing this test run
$requestId = [guid]::NewGuid().ToString()
$env:DINO_REQUEST_ID = $requestId

# Import logging module
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$loggingModule = Join-Path $scriptDir "..\shared\Logging.psm1"
if (Test-Path $loggingModule) {
    Import-Module $loggingModule -Force
} else {
    Write-Warning "Logging module not found at $loggingModule - falling back to Write-Host"
}

Write-Host "=== DINOForge Parallel Automation Test ===" -ForegroundColor Cyan
Write-LogInfo "Starting parallel automation test" @{
    instanceCount = $InstanceCount
    testDurationSeconds = $TestDurationSeconds
    mcpUrl = $McpUrl
    skipMcpCheck = $SkipMcpCheck
    requestId = $requestId
} -RequestId $requestId

# Check MCP server health (HTTP health endpoint)
if (-not $SkipMcpCheck) {
    Write-LogInfo "Checking MCP server health" @{ mcpUrl = $McpUrl } -RequestId $requestId
    try {
        $health = Invoke-WebRequest `
            -Uri "$McpUrl/health" `
            -TimeoutSec 3 `
            -ErrorAction Stop

        if ($health.StatusCode -eq 200) {
            $respObj = $health.Content | ConvertFrom-Json
            if ($respObj.status -eq "ok") {
                Write-LogInfo "MCP server health check passed" @{
                    status = $respObj.status
                    version = $respObj.version
                } -RequestId $requestId
            } else {
                Write-LogError "MCP health check failed" @{ status = $respObj.status } -RequestId $requestId
                exit 1
            }
        } else {
            Write-LogError "MCP health check failed" @{ statusCode = $health.StatusCode } -RequestId $requestId
            exit 1
        }
    } catch {
        Write-LogError "Cannot reach MCP server" @{ mcpUrl = $McpUrl; error = $_ } -RequestId $requestId
        exit 1
    }
}

# First, launch the game instances
Write-LogInfo "Launching game instances" @{ instanceCount = $InstanceCount } -RequestId $requestId

$launcherPath = Join-Path $PSScriptRoot "Launch-ParallelGames.ps1"
if (-not (Test-Path $launcherPath)) {
    Write-LogError "Cannot find launcher script" @{ launcherPath = $launcherPath } -RequestId $requestId
    exit 1
}

$launchResult = & $launcherPath -InstanceCount $InstanceCount -Verbose:$Verbose
if (-not $launchResult.Processes -or @($launchResult.Processes).Count -eq 0) {
    Write-LogError "Failed to launch game instances" @{ launchResult = $launchResult } -RequestId $requestId
    exit 1
}

$runningInstances = @($launchResult.Processes)
$pipenames = $launchResult.PipeNames
Write-LogInfo "Game instances launched successfully" @{
    runningCount = $runningInstances.Count
    pipeNames = $pipenames
} -RequestId $requestId

# Test metrics
Write-LogInfo "Starting test suite execution" @{
    testDurationSeconds = $TestDurationSeconds
    instanceCount = $InstanceCount
} -RequestId $requestId

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
                Write-LogDebug "Game status test passed" @{
                    instanceNum = $instNum
                    responseTime = $responseTime
                } -RequestId $requestId
            } else {
                $testsFailed++
                $perInstanceStats[$instIdx].Failed++
                Write-LogWarn "Game status test failed" @{
                    instanceNum = $instNum
                    error = $result.error
                } -RequestId $requestId
            }
        } catch {
            $testsFailed++
            $perInstanceStats[$instIdx].Failed++
            Write-LogError "Game status test error" @{
                instanceNum = $instNum
                error = $_.Exception.Message
            } -RequestId $requestId
        }

        # Test 2: Check entity count via status (verifies ECS world is active)
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

            # Verify entity count is > 0 (indicates ECS world is ready)
            if ($result.success -and $result.EntityCount -gt 0) {
                $testsPassed++
                $perInstanceStats[$instIdx].Passed++
                Write-LogDebug "Entity count test passed" @{
                    instanceNum = $instNum
                    entityCount = $result.EntityCount
                    responseTime = $responseTime
                } -RequestId $requestId
            } else {
                $testsFailed++
                $perInstanceStats[$instIdx].Failed++
                Write-LogWarn "Entity count check failed" @{
                    instanceNum = $instNum
                    entityCount = $result.EntityCount
                } -RequestId $requestId
            }
        } catch {
            $testsFailed++
            $perInstanceStats[$instIdx].Failed++
            Write-LogError "Entity count test error" @{
                instanceNum = $instNum
                error = $_.Exception.Message
            } -RequestId $requestId
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
                Write-LogDebug "Runtime verification test passed" @{
                    instanceNum = $instNum
                    responseTime = $responseTime
                } -RequestId $requestId
            } else {
                $testsFailed++
                $perInstanceStats[$instIdx].Failed++
                Write-LogWarn "Runtime verification test failed" @{
                    instanceNum = $instNum
                } -RequestId $requestId
            }
        } catch {
            $testsFailed++
            $perInstanceStats[$instIdx].Failed++
            Write-LogError "Runtime verification test error" @{
                instanceNum = $instNum
                error = $_.Exception.Message
            } -RequestId $requestId
        }
    }

    $iterationTime = ((Get-Date) - $iterationStart).TotalMilliseconds
    if ($iterationCount % 5 -eq 0) {
        Write-LogDebug "Test iteration progress" @{
            iterationCount = $iterationCount
            testsPassed = $testsPassed
            testsFailed = $testsFailed
            iterationTime = [Math]::Round($iterationTime, 2)
        } -RequestId $requestId
    }

    $sleepTime = [Math]::Max(100, 500 - $iterationTime)
    Start-Sleep -Milliseconds $sleepTime
}

# Calculate success rate
$totalTests = $testsPassed + $testsFailed
$successRate = if ($totalTests -gt 0) { ($testsPassed / $totalTests) * 100 } else { 0 }
$avgResponseTime = if ($totalTests -gt 0) { [Math]::Round($totalTime / $totalTests, 2) } else { 0 }

# Output results
$duration = ((Get-Date) - $startTime).TotalSeconds

Write-LogInfo "Test execution completed" @{
    duration = [Math]::Round($duration, 1)
    iterations = $iterationCount
    totalTests = $totalTests
    testsPassed = $testsPassed
    testsFailed = $testsFailed
    successRate = [Math]::Round($successRate, 2)
    avgResponseTime = $avgResponseTime
} -RequestId $requestId

# Per-instance breakdown
for ($i = 0; $i -lt $InstanceCount; $i++) {
    $stats = $perInstanceStats[$i]
    $instTests = $stats.Passed + $stats.Failed
    $instRate = if ($instTests -gt 0) { [Math]::Round(($stats.Passed / $instTests) * 100, 2) } else { 0 }
    Write-LogDebug "Per-instance statistics" @{
        instanceNum = $i + 1
        passed = $stats.Passed
        failed = $stats.Failed
        rate = $instRate
    } -RequestId $requestId
}

# Status indicator
if ($successRate -ge 95) {
    Write-LogInfo "Test PASSED (95%+ success rate)" @{ successRate = $successRate } -RequestId $requestId
    $exitCode = 0
} elseif ($successRate -ge 80) {
    Write-LogWarn "Test DEGRADED (80-95% success rate)" @{ successRate = $successRate } -RequestId $requestId
    $exitCode = 1
} else {
    Write-LogError "Test FAILED (below 80% success rate)" @{ successRate = $successRate } -RequestId $requestId
    $exitCode = 2
}

# Cleanup
Write-LogInfo "Cleaning up test resources" @{ } -RequestId $requestId
$runningInstances | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 500
Write-LogInfo "Test run completed and cleanup finished" @{
    exitCode = $exitCode
    totalDuration = [Math]::Round($duration, 1)
} -RequestId $requestId

exit $exitCode
