# Structured Logging (JSON/JSONL) for DINOForge Automation

This document describes the structured logging infrastructure implemented across the DINOForge game automation stack (PowerShell scripts and C# tools).

## Overview

All automation operations now produce **structured JSON logs** to enable:
- Audit trails and compliance tracking
- Machine-readable log analysis and parsing
- Correlation of related operations across process boundaries
- CI/CD integration and automated failure analysis
- Debugging with precise operation timing and context

## Architecture

### Components

1. **PowerShell Logging Module** (`scripts/shared/Logging.psm1`)
   - Core logging functions (Write-LogInfo, Write-LogWarn, Write-LogError, Write-LogDebug)
   - JSONL file writer with automatic directory creation
   - Console output with color-coded levels
   - Correlation ID support (RequestId field)

2. **C# Serilog Integration** (GameClient.cs, GameControlCli)
   - Structured logging via Serilog library
   - Dual output: console + JSONL file sink
   - Request/correlation ID enrichment from `DINO_REQUEST_ID` environment variable
   - Automatic process and machine name tracking

3. **Log Export Utility** (`scripts/game/Export-Logs.ps1`)
   - Read and aggregate JSONL logs from multiple sources
   - Filter by level, date range, request ID
   - Output as JSON (structured) or Markdown (human-readable)

## Usage

### PowerShell Scripts

Import the logging module at the start of any script:

```powershell
# Generate unique request ID for tracing
$requestId = [guid]::NewGuid().ToString()
$env:DINO_REQUEST_ID = $requestId

# Import logging module
Import-Module "$PSScriptRoot/shared/Logging.psm1" -Force

# Use logging functions
Write-LogInfo "Starting operation" @{
    param1 = "value1"
    param2 = 42
} -RequestId $requestId

Write-LogWarn "Something unexpected" @{ threshold = 100; actual = 95 } -RequestId $requestId

Write-LogError "Operation failed" @{ error = $_.Exception.Message } -RequestId $requestId

Write-LogDebug "Detailed info" @{ variableX = $x; count = $list.Count } -RequestId $requestId
```

### C# Tools (GameClient, etc.)

Serilog is configured automatically in `GameClient.InitializeLogger()`:

```csharp
// Read correlation ID from environment variable
var requestId = Environment.GetEnvironmentVariable("DINO_REQUEST_ID") ?? "no-request-id";

// Log structured data
_logger.Information("Operation started", new { operationId = "123", timeout = 5000 });
_logger.Warning("Timeout exceeded", new { elapsedMs = 5100 });
_logger.Error(ex, "Operation failed", new { retryCount = 3 });
```

The C# logger automatically enriches all messages with:
- `ProcessName`: Name of the C# executable (e.g., GameControlCli)
- `ProcessId`: Process ID
- `MachineName`: Machine hostname
- `RequestId`: Correlation ID from env var

### Log File Locations

**PowerShell logs:**
- `$env:TEMP\DINOForge\dinoforge.jsonl`
  - Single cumulative JSONL file (one entry per line)
  - Entries are appended; never overwritten

**C# logs:**
- `logs/dinoforge-YYYY-MM-DD.jsonl`
  - Daily rolling files (one file per day)
  - Located in the working directory where the C# tool is run

### Log Export and Analysis

Export logs after a test run:

```powershell
# Export all logs to JSON report
.\scripts\game\Export-Logs.ps1 -Output logs/test-run.json

# Export logs for a specific request ID
.\scripts\game\Export-Logs.ps1 -RequestId "12345678-1234-1234-1234-123456789012" -Output logs/specific-run.json

# Export only ERROR level logs as markdown
.\scripts\game\Export-Logs.ps1 -Level ERROR -Format markdown -Output logs/errors.md

# Export logs from a time range
.\scripts\game\Export-Logs.ps1 `
    -StartTime "2026-04-12T10:00:00" `
    -EndTime "2026-04-12T11:00:00" `
    -Output logs/hourly.json
```

## Log Entry Structure

### PowerShell Entry Format

```json
{
  "timestamp": "2026-04-12T15:30:45.1234567Z",
  "level": "INFO",
  "message": "Game instance created",
  "context": {
    "instanceCount": 2,
    "outputDir": "G:\\dino_boxes"
  },
  "processId": 12345,
  "machineName": "WORKSTATION",
  "requestId": "a1b2c3d4-a1b2-a1b2-a1b2-a1b2c3d4a1b2"
}
```

### C# Entry Format (Serilog JSON)

```json
{
  "Timestamp": "2026-04-12T15:30:45.1234567Z",
  "Level": "Information",
  "MessageTemplate": "Request '{Method}' completed successfully in {ElapsedMs}ms",
  "Properties": {
    "Method": "status",
    "ElapsedMs": 245,
    "ProcessName": "GameControlCli",
    "ProcessId": 5678,
    "MachineName": "WORKSTATION",
    "RequestId": "a1b2c3d4-a1b2-a1b2-a1b2-a1b2c3d4a1b2"
  }
}
```

## Correlation ID (Request Tracing)

All operations are traced via **request/correlation IDs**. Each automation script generates a unique GUID:

```powershell
$requestId = [guid]::NewGuid().ToString()
$env:DINO_REQUEST_ID = $requestId
```

This ID is:
1. Passed to all PowerShell Write-Log* calls via `-RequestId` parameter
2. Available to child C# processes via the `DINO_REQUEST_ID` environment variable
3. Automatically included in all Serilog logs
4. Used by Export-Logs.ps1 to group related entries

**Benefit:** You can trace a single test run across all components:

```powershell
# All entries with this RequestId form a complete audit trail
.\scripts\game\Export-Logs.ps1 -RequestId "a1b2c3d4-a1b2-a1b2-a1b2-a1b2c3d4a1b2" | jq .
```

## Log Levels

Four levels are used consistently:

| Level | Use Case | Example |
|-------|----------|---------|
| DEBUG | Detailed diagnostic info | Variable values, function entry/exit, state changes |
| INFO | Normal operation milestones | Game launched, test iteration complete, request succeeded |
| WARN | Unexpected but recoverable | Retry attempt, timeout occurred, degraded success rate |
| ERROR | Failures requiring intervention | Connection failed, server error, all retries exhausted |

## CI/CD Integration

### GitHub Actions

Update `.github/workflows/game-automation.yml` to capture logs:

```yaml
- name: Run game automation tests
  run: |
    pwsh -Command "& ./scripts/automation/Test-ParallelAutomation.ps1"

- name: Export logs
  if: always()
  run: |
    pwsh -Command "& ./scripts/game/Export-Logs.ps1 -Output logs/test-run.json"

- name: Upload logs
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: automation-logs
    path: logs/
```

### Log Analysis in CI

Parse exported JSON logs for failure reporting:

```powershell
# Count errors by type
$logs = Get-Content logs/test-run.json | ConvertFrom-Json
$logs.requestGroups.entries | 
    Where-Object { $_.level -eq "ERROR" } | 
    Group-Object { $_.message } | 
    Select-Object @{ Name = "Error"; Expression = { $_.Name } }, 
                  @{ Name = "Count"; Expression = { $_.Count } }
```

## Best Practices

### When to Log

- **Always log operation boundaries** (start, success, error)
- **Log parameters and context** that help understand failures
- **Log timing information** for performance analysis
- **Log retry/retry attempts** to understand resilience
- **Never log sensitive data** (passwords, API keys, PII)

### Context Objects

Keep context hashtables focused and structured:

```powershell
# Good: Clear, structured context
Write-LogInfo "Instance launched" @{
    instanceNum = 1
    pid = 2345
    pipeName = "dinoforge-main"
} -RequestId $requestId

# Avoid: Unstructured, hard to query
Write-LogInfo "Launched instance 1 with PID 2345 on pipe dinoforge-main"
```

### Timing Information

Include elapsed time for long operations:

```csharp
var sw = Stopwatch.StartNew();
DoWork();
sw.Stop();
_logger.Information("Work completed in {ElapsedMs}ms", sw.ElapsedMilliseconds);
```

### Request ID Propagation

Always set and propagate the request ID:

```powershell
$requestId = [guid]::NewGuid().ToString()
$env:DINO_REQUEST_ID = $requestId

# All child processes inherit $env:DINO_REQUEST_ID
$process = Start-Process -FilePath "some-tool.exe" -PassThru
```

## Troubleshooting

### Logs not appearing

1. Verify log directory exists: `$env:TEMP\DINOForge\` or `logs/`
2. Check file permissions: Script must have write access
3. Verify Logging module is imported: `Get-Module Logging | Select-Object Name, Version`
4. Check Serilog initialization: Ensure GameClient is instantiated before logging

### RequestId not populated

1. Set env var before calling child processes: `$env:DINO_REQUEST_ID = $requestId`
2. Verify C# tools read from environment: Check InitializeLogger() implementation
3. Use Export-Logs to verify: Filter logs and check RequestId field

### Export-Logs failing

1. Ensure log files are valid JSON (one JSON object per line)
2. Check for Unicode BOM or encoding issues: Use UTF-8
3. Verify output directory is writable
4. Check timestamps are ISO 8601 format (with 'Z' suffix or ±HH:MM)

## References

- [Serilog Documentation](https://serilog.net/)
- [JSON Logging Best Practices](https://github.com/brynbellomy/structured-log-formatter)
- [Keep a Changelog](https://keepachangelog.com/)
- [ISO 8601 Date Format](https://en.wikipedia.org/wiki/ISO_8601)
