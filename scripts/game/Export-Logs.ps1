#!/usr/bin/env powershell
<#
.SYNOPSIS
Export and aggregate JSONL logs from DINOForge automation runs.

.DESCRIPTION
This utility reads all JSONL log files from:
- $env:TEMP\DINOForge\dinoforge.jsonl (PowerShell automation logs)
- logs/ directory (C# tool logs)

It aggregates logs by request ID, allows filtering by level/date range,
and outputs a structured report (JSON or markdown).

.PARAMETER RequestId
Filter logs to a specific request ID (correlation ID)

.PARAMETER Level
Filter by log level: DEBUG, INFO, WARN, ERROR (default: all)

.PARAMETER StartTime
Filter logs after this timestamp (ISO 8601 format, e.g., 2026-04-12T10:30:00)

.PARAMETER EndTime
Filter logs before this timestamp (ISO 8601 format)

.PARAMETER Output
Output file path (default: logs/export-{timestamp}.json)
Use .md extension for markdown report

.PARAMETER Format
Output format: json, markdown (default: json)

.PARAMETER IncludeContext
Include full context objects in output (default: true for JSON, false for markdown)

.EXAMPLE
.\Export-Logs.ps1
Export all logs to logs/export-{timestamp}.json

.\Export-Logs.ps1 -RequestId "12345678-1234-1234-1234-123456789012" -Output logs/test-run.json
Export logs for a specific test run

.\Export-Logs.ps1 -Level ERROR -Format markdown -Output logs/errors.md
Export all ERROR level logs as markdown report

.\Export-Logs.ps1 -StartTime "2026-04-12T10:00:00" -EndTime "2026-04-12T11:00:00" -Output logs/hourly.json
Export logs from a specific time window

.NOTES
The export process:
1. Reads dinoforge.jsonl from $env:TEMP\DINOForge\
2. Reads all .jsonl files from logs/
3. Aggregates by requestId (correlation ID)
4. Applies filters (level, time range)
5. Outputs aggregated report
#>

param(
    [string]$RequestId,
    [ValidateSet('DEBUG', 'INFO', 'WARN', 'ERROR')]
    [string]$Level,
    [string]$StartTime,
    [string]$EndTime,
    [string]$Output,
    [ValidateSet('json', 'markdown')]
    [string]$Format = 'json',
    [switch]$IncludeContext = $true
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Determine output file if not specified
if (-not $Output) {
    $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
    $ext = if ($Format -eq 'markdown') { '.md' } else { '.json' }
    $Output = "logs/export-$timestamp$ext"
}

# Ensure output directory exists
$outputDir = Split-Path -Parent $Output
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
}

Write-Host "=== DINOForge Log Export ===" -ForegroundColor Cyan
Write-Host "Output format: $Format"
if ($RequestId) { Write-Host "Request ID filter: $RequestId" }
if ($Level) { Write-Host "Level filter: $Level" }
if ($StartTime) { Write-Host "Start time: $StartTime" }
if ($EndTime) { Write-Host "End time: $EndTime" }
Write-Host "Output path: $Output"
Write-Host ""

# Collect all JSONL files
$logFiles = @()

# Check PowerShell automation logs
$psLogFile = Join-Path $env:TEMP "DINOForge\dinoforge.jsonl"
if (Test-Path $psLogFile) {
    $logFiles += $psLogFile
    Write-Host "Found PowerShell logs: $psLogFile" -ForegroundColor Green
}

# Check C# tool logs directory
$logsDir = "logs"
if (Test-Path $logsDir) {
    $jsonlFiles = Get-ChildItem $logsDir -Filter "*.jsonl" -Recurse -ErrorAction SilentlyContinue
    if ($jsonlFiles) {
        $logFiles += @($jsonlFiles | ForEach-Object { $_.FullName })
        Write-Host "Found $($jsonlFiles.Count) C# tool log files" -ForegroundColor Green
    }
}

if ($logFiles.Count -eq 0) {
    Write-Host "No JSONL log files found" -ForegroundColor Yellow
    exit 0
}

# Read and parse all log entries
Write-Host ""
Write-Host "Reading log entries..." -ForegroundColor Cyan

$allLogs = @()
$errorCount = 0

foreach ($logFile in $logFiles) {
    if (-not (Test-Path $logFile)) {
        continue
    }

    $entries = 0
    try {
        Get-Content -Path $logFile -Encoding UTF8 | ForEach-Object {
            if ([string]::IsNullOrWhiteSpace($_)) {
                return
            }

            try {
                $entry = $_ | ConvertFrom-Json
                $allLogs += $entry
                $entries++
            } catch {
                Write-Warning "Failed to parse line in $logFile : $_"
                $errorCount++
            }
        }

        Write-Host "  Read $entries entries from $logFile" -ForegroundColor DarkGreen
    } catch {
        Write-Warning "Error reading $logFile : $_"
        $errorCount++
    }
}

Write-Host "Total entries read: $($allLogs.Count)" -ForegroundColor Green
if ($errorCount -gt 0) {
    Write-Host "Parse errors: $errorCount" -ForegroundColor Yellow
}

# Apply filters
$filtered = $allLogs

if ($RequestId) {
    $filtered = @($filtered | Where-Object { $_.requestId -eq $RequestId })
}

if ($Level) {
    $filtered = @($filtered | Where-Object { $_.level -eq $Level })
}

if ($StartTime) {
    try {
        $startTimeObj = [datetime]::Parse($StartTime)
        $filtered = @($filtered | Where-Object { [datetime]::Parse($_.timestamp) -ge $startTimeObj })
    } catch {
        Write-Error "Invalid StartTime format: $StartTime. Use ISO 8601 (e.g., 2026-04-12T10:30:00)"
    }
}

if ($EndTime) {
    try {
        $endTimeObj = [datetime]::Parse($EndTime)
        $filtered = @($filtered | Where-Object { [datetime]::Parse($_.timestamp) -le $endTimeObj })
    } catch {
        Write-Error "Invalid EndTime format: $EndTime. Use ISO 8601 (e.g., 2026-04-12T11:30:00)"
    }
}

Write-Host ""
Write-Host "Filtered entries: $($filtered.Count)" -ForegroundColor Green

# Build output
$output = @{}

if ($Format -eq 'json') {
    # JSON output: aggregate by requestId
    $byRequestId = @{}
    foreach ($entry in $filtered) {
        $rid = $entry.requestId ?? "uncorrelated"
        if (-not $byRequestId.ContainsKey($rid)) {
            $byRequestId[$rid] = @{
                requestId = $rid
                entries = @()
                summary = @{
                    totalEntries = 0
                    byLevel = @{}
                    startTime = $null
                    endTime = $null
                }
            }
        }

        $entryToAdd = @{
            timestamp = $entry.timestamp
            level = $entry.level
            message = $entry.message
        }

        if ($IncludeContext -and $entry.context) {
            $entryToAdd['context'] = $entry.context
        }

        $entryToAdd['processId'] = $entry.processId
        $entryToAdd['machineName'] = $entry.machineName

        $byRequestId[$rid].entries += $entryToAdd
        $byRequestId[$rid].summary.totalEntries++

        if (-not $byRequestId[$rid].summary.byLevel.ContainsKey($entry.level)) {
            $byRequestId[$rid].summary.byLevel[$entry.level] = 0
        }
        $byRequestId[$rid].summary.byLevel[$entry.level]++

        $ts = [datetime]::Parse($entry.timestamp)
        if ($null -eq $byRequestId[$rid].summary.startTime -or $ts -lt [datetime]::Parse($byRequestId[$rid].summary.startTime)) {
            $byRequestId[$rid].summary.startTime = $entry.timestamp
        }
        if ($null -eq $byRequestId[$rid].summary.endTime -or $ts -gt [datetime]::Parse($byRequestId[$rid].summary.endTime)) {
            $byRequestId[$rid].summary.endTime = $entry.timestamp
        }
    }

    $output = @{
        exportTime = [datetime]::UtcNow.ToString('o')
        totalEntries = $filtered.Count
        requestGroups = $byRequestId.Values | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    }

    $output | ConvertTo-Json -Depth 10 | Out-File -FilePath $Output -Encoding UTF8
}
else {
    # Markdown output: human-readable report
    $md = @()
    $md += "# DINOForge Automation Logs Export"
    $md += ""
    $md += "**Export Time**: $(Get-Date -Format 'o')"
    $md += "**Total Entries**: $($filtered.Count)"
    $md += ""

    if ($RequestId) {
        $md += "**Request ID Filter**: `$RequestId"
        $md += ""
    }

    # Group by request ID
    $byRequestId = $filtered | Group-Object -Property { $_.requestId ?? "uncorrelated" } -AsHashTable

    foreach ($rid in ($byRequestId.Keys | Sort-Object)) {
        $entries = @($byRequestId[$rid])
        $md += "## Request: $rid"
        $md += ""

        # Summary
        $byLevel = $entries | Group-Object -Property level -AsHashTable
        $md += "**Summary**:"
        $md += "- Total entries: $($entries.Count)"
        foreach ($level in ('ERROR', 'WARN', 'INFO', 'DEBUG')) {
            if ($byLevel.ContainsKey($level)) {
                $md += "- $level: $($byLevel[$level].Count)"
            }
        }
        $md += ""

        # Log entries
        $md += "**Log Entries**:"
        $md += ""
        $md += "| Timestamp | Level | Message | Machine |"
        $md += "|-----------|-------|---------|---------|"

        foreach ($entry in ($entries | Sort-Object timestamp)) {
            $msg = $entry.message -replace '\|', '\\|' -replace '\n', ' '
            $md += "| $($entry.timestamp) | $($entry.level) | $msg | $($entry.machineName) |"
        }
        $md += ""
    }

    $md | Out-File -FilePath $Output -Encoding UTF8

    Write-Host ""
    Write-Host "Generated markdown report" -ForegroundColor Green
}

Write-Host ""
Write-Host "Export complete: $Output" -ForegroundColor Green
Write-Host ""
