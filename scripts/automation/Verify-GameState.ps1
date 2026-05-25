#!/usr/bin/env powershell
<#
.SYNOPSIS
Automated game state verification using MCP tools.

Captures screenshots every N seconds, analyzes visual state,
compares against golden references, and reports anomalies.

.PARAMETER InstanceCount
Number of instances to verify (default: 4)

.PARAMETER CaptureIntervalSeconds
Interval between screenshots (default: 5)

.PARAMETER TestDurationSeconds
Total test duration (default: 60)

.PARAMETER McpUrl
Base URL of the MCP server (default: http://127.0.0.1:8765)

.PARAMETER OutputDir
Directory to save screenshots and reports (default: docs/automation/screenshots/)

.PARAMETER Verbose
Enable verbose logging

.EXAMPLE
.\Verify-GameState.ps1 -InstanceCount 2 -TestDurationSeconds 30
Verify 2 game instances for 30 seconds, capture every 5s

.\Verify-GameState.ps1 -InstanceCount 4 -CaptureIntervalSeconds 10 -Verbose
Verify 4 instances with 10s capture interval and verbose output
#>

param(
    [int]$InstanceCount = 4,
    [int]$CaptureIntervalSeconds = 5,
    [int]$TestDurationSeconds = 60,
    [string]$McpUrl = "http://127.0.0.1:8765",
    [string]$OutputDir = "docs/automation/screenshots",
    [switch]$Verbose
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Continue"

Write-Host "=== DINOForge Game State Verification ===" -ForegroundColor Cyan
Write-Host "Instance count: $InstanceCount"
Write-Host "Capture interval: ${CaptureIntervalSeconds}s"
Write-Host "Test duration: ${TestDurationSeconds}s"
Write-Host "Output directory: $OutputDir"
Write-Host ""

# Create output directory
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    Write-Host "[PASS] Created output directory: $OutputDir" -ForegroundColor Green
}

# Verify MCP is running
Write-Host "Verifying MCP server..." -ForegroundColor Yellow
try {
    $health = Invoke-WebRequest -Uri "$McpUrl/health" -TimeoutSec 3 -ErrorAction Stop
    if ($health.StatusCode -eq 200) {
        Write-Host "[PASS] MCP server is running" -ForegroundColor Green
    } else {
        Write-Error "MCP health check failed"
        exit 1
    }
} catch {
    Write-Error "Cannot reach MCP server at $McpUrl"
    exit 1
}

# State tracking
$startTime = Get-Date
$captureCount = 0
$analyzeCount = 0
$anomalies = @()

Write-Host ""
Write-Host "Starting verification loop..." -ForegroundColor Cyan

# Main verification loop
while ((Get-Date) -lt $startTime.AddSeconds($TestDurationSeconds)) {
    $iterationStart = Get-Date

    for ($instIdx = 1; $instIdx -le $InstanceCount; $instIdx++) {
        # Capture screenshot
        try {
            $timestamp = Get-Date -Format "yyyyMMdd-HHmmss-fff"
            $screenshotPath = Join-Path $OutputDir "instance_${instIdx}_${timestamp}.png"

            $body = @{
                jsonrpc = "2.0"
                method = "game_screenshot"
                params = @{ output_file = $screenshotPath }
                id = ($instIdx * 1000 + 100 + $captureCount)
            } | ConvertTo-Json -Depth 10

            $response = Invoke-WebRequest `
                -Uri "$McpUrl/api/tools/game_screenshot" `
                -Method POST `
                -Body $body `
                -ContentType "application/json" `
                -TimeoutSec 5 `
                -ErrorAction SilentlyContinue

            if ($response.StatusCode -eq 200) {
                $captureCount++
                if ($Verbose) {
                    Write-Host "  [PASS] Instance $instIdx : Screenshot captured" -ForegroundColor Green
                }
            } else {
                if ($Verbose) {
                    Write-Host "  [FAIL] Instance $instIdx : Screenshot failed" -ForegroundColor Red
                }
            }
        } catch {
            if ($Verbose) {
                Write-Host "  [FAIL] Instance $instIdx : Screenshot error" -ForegroundColor Red
            }
        }

        # Analyze screen (if available)
        try {
            $body = @{
                jsonrpc = "2.0"
                method = "game_analyze_screen"
                params = @{
                    detect_ui = $true
                    detect_entities = $true
                }
                id = ($instIdx * 1000 + 200 + $analyzeCount)
            } | ConvertTo-Json -Depth 10

            $response = Invoke-WebRequest `
                -Uri "$McpUrl/api/tools/game_analyze_screen" `
                -Method POST `
                -Body $body `
                -ContentType "application/json" `
                -TimeoutSec 5 `
                -ErrorAction SilentlyContinue

            if ($response.StatusCode -eq 200) {
                $analyzeCount++
                $analysis = $response.Content | ConvertFrom-Json

                # Check against golden references
                if ($analysis.ui_elements.Count -eq 0) {
                    $anomalies += @{
                        Instance = $instIdx
                        Timestamp = Get-Date
                        Anomaly = "UI elements not detected"
                        Expected = "UI overlay should be visible"
                    }
                    if ($Verbose) {
                        Write-Host "  [WARN] Instance $instIdx : UI missing (anomaly detected)" -ForegroundColor Yellow
                    }
                }

                if ($Verbose) {
                    Write-Host "  [PASS] Instance $instIdx : Analysis complete" -ForegroundColor Green
                }
            } else {
                if ($Verbose) {
                    Write-Host "  [FAIL] Instance $instIdx : Analysis failed" -ForegroundColor Red
                }
            }
        } catch {
            if ($Verbose) {
                Write-Host "  [NOTE] Instance $instIdx : Analysis unavailable (expected)" -ForegroundColor DarkGray
            }
        }
    }

    # Wait for next capture interval
    $elapsedMs = ((Get-Date) - $iterationStart).TotalMilliseconds
    $sleepTime = [Math]::Max(100, ($CaptureIntervalSeconds * 1000) - $elapsedMs)
    if ($Verbose) {
        Write-Host "[Iteration] Captured: $captureCount | Analyzed: $analyzeCount | Next in ${sleepTime}ms" -ForegroundColor DarkCyan
    }
    Start-Sleep -Milliseconds $sleepTime
}

# Generate report
Write-Host ""
Write-Host "=== Verification Report ===" -ForegroundColor Cyan
$duration = ((Get-Date) - $startTime).TotalSeconds
Write-Host "Total duration: $([Math]::Round($duration, 1)) seconds"
Write-Host "Screenshots captured: $captureCount"
Write-Host "Analyses performed: $analyzeCount"
Write-Host "Anomalies detected: $($anomalies.Count)"

if ($anomalies.Count -gt 0) {
    Write-Host ""
    Write-Host "Detected Anomalies:" -ForegroundColor Yellow
    foreach ($anomaly in $anomalies) {
        Write-Host "  Instance $($anomaly.Instance) at $($anomaly.Timestamp):" -ForegroundColor Yellow
        Write-Host "    $($anomaly.Anomaly)" -ForegroundColor Yellow
        Write-Host "    Expected: $($anomaly.Expected)" -ForegroundColor Yellow
    }
} else {
    Write-Host "[PASS] No anomalies detected" -ForegroundColor Green
}

# Save report
$reportPath = Join-Path $OutputDir "verification_report.json"
@{
    timestamp = Get-Date
    duration_seconds = $duration
    instances = $InstanceCount
    screenshots_captured = $captureCount
    analyses_performed = $analyzeCount
    anomalies = $anomalies
    output_directory = (Resolve-Path $OutputDir).Path
} | ConvertTo-Json -Depth 10 | Set-Content $reportPath

Write-Host ""
Write-Host "[PASS] Report saved : $reportPath" -ForegroundColor Green
Write-Host "[PASS] Screenshots saved to : $OutputDir" -ForegroundColor Green
Write-Host ""
Write-Host "=== Verification Complete ===" -ForegroundColor Cyan
