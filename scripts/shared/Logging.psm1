#Requires -Version 5.0
<#
.SYNOPSIS
Structured logging module for DINOForge automation scripts.
Provides JSON/JSONL logging with console output, suitable for audit trails and CI analysis.

.DESCRIPTION
This module exports functions for structured logging to JSONL format with:
- Timestamp (ISO 8601)
- Log level (INFO, WARN, ERROR, DEBUG)
- Message and structured context (as JSON object)
- Process ID and machine name
- Optional request/correlation ID for tracing operations

All logs are written to $env:TEMP\DINOForge\dinoforge.jsonl for aggregation and analysis.

.EXAMPLE
Import-Module ./scripts/shared/Logging.psm1 -Force

Write-LogInfo "Game instance created" @{ instanceCount = 2; outputDir = "G:\dino_boxes" }
Write-LogWarn "Sandbox cleanup incomplete" @{ remainingFiles = 12 }
Write-LogError "Failed to connect to pipe" @{ pipeName = "dinoforge-1"; retries = 3 }

$requestId = [guid]::NewGuid().ToString()
Write-LogInfo "Starting test run" @{ requestId = $requestId; testCount = 10 }
#>

# Ensure log directory exists
function Ensure-LogDirectory {
    $logDir = "$env:TEMP\DINOForge"
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir -Force | Out-Null
    }
    return $logDir
}

# Get the JSONL log file path
function Get-LogFilePath {
    param(
        [string]$LogDir = "$(Ensure-LogDirectory)"
    )
    return Join-Path $LogDir "dinoforge.jsonl"
}

<#
.SYNOPSIS
Write a structured JSON log entry to JSONL file and console.

.PARAMETER Level
Log level: INFO, WARN, ERROR, DEBUG

.PARAMETER Message
Human-readable log message

.PARAMETER Context
Hashtable of structured context data (converted to JSON object)

.PARAMETER LogFile
Path to JSONL log file (default: $env:TEMP\DINOForge\dinoforge.jsonl)

.PARAMETER RequestId
Optional correlation/request ID for tracing related operations

.EXAMPLE
Write-LogJson -Level INFO -Message "Starting process" -Context @{ timeout = 30000 }
#>
function Write-LogJson {
    param(
        [Parameter(Mandatory=$true)]
        [ValidateSet('DEBUG', 'INFO', 'WARN', 'ERROR')]
        [string]$Level,

        [Parameter(Mandatory=$true)]
        [string]$Message,

        [hashtable]$Context = @{},

        [string]$LogFile = "$(Get-LogFilePath)",

        [string]$RequestId
    )

    # Ensure log directory exists
    $logDir = Split-Path -Parent $LogFile
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir -Force | Out-Null
    }

    # Build log entry
    $logEntry = @{
        timestamp = [datetime]::UtcNow.ToString('o')
        level = $Level
        message = $Message
        context = $Context
        processId = $PID
        machineName = $env:COMPUTERNAME
    }

    # Add correlation ID if provided
    if ($RequestId) {
        $logEntry['requestId'] = $RequestId
    }

    # Convert to JSON and write to file
    $jsonLog = $logEntry | ConvertTo-Json -Compress
    Add-Content -Path $LogFile -Value $jsonLog -Encoding UTF8 -ErrorAction SilentlyContinue

    # Also output to console with color-coded level
    $timeStr = Get-Date -Format 'HH:mm:ss'
    $consoleColor = switch($Level) {
        'ERROR'   { 'Red' }
        'WARN'    { 'Yellow' }
        'INFO'    { 'Green' }
        'DEBUG'   { 'Cyan' }
        default   { 'White' }
    }

    Write-Host "[$timeStr] [$Level] $Message" -ForegroundColor $consoleColor

    # Optionally log context to console in debug mode
    if ($Level -eq 'DEBUG' -and $Context.Count -gt 0) {
        $contextStr = $Context | ConvertTo-Json -Compress
        Write-Host "         Context: $contextStr" -ForegroundColor DarkGray
    }
}

<#
.SYNOPSIS
Write an INFO level log entry.

.PARAMETER Message
Log message

.PARAMETER Context
Structured context data

.PARAMETER RequestId
Optional correlation ID

.EXAMPLE
Write-LogInfo "Starting operation" @{ timeout = 5000 }
#>
function Write-LogInfo {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Message,

        [hashtable]$Context = @{},

        [string]$RequestId
    )

    $params = @{ Level = 'INFO'; Message = $Message; Context = $Context }
    if ($RequestId) { $params['RequestId'] = $RequestId }
    Write-LogJson @params
}

<#
.SYNOPSIS
Write a WARN level log entry.

.PARAMETER Message
Log message

.PARAMETER Context
Structured context data

.PARAMETER RequestId
Optional correlation ID

.EXAMPLE
Write-LogWarn "Timeout occurred" @{ elapsedMs = 30000; expectedMs = 25000 }
#>
function Write-LogWarn {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Message,

        [hashtable]$Context = @{},

        [string]$RequestId
    )

    $params = @{ Level = 'WARN'; Message = $Message; Context = $Context }
    if ($RequestId) { $params['RequestId'] = $RequestId }
    Write-LogJson @params
}

<#
.SYNOPSIS
Write an ERROR level log entry.

.PARAMETER Message
Log message

.PARAMETER Context
Structured context data (typically includes error details)

.PARAMETER RequestId
Optional correlation ID

.EXAMPLE
Write-LogError "Failed to launch game" @{ pipeName = "test-1"; errorCode = -2147483648 }
#>
function Write-LogError {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Message,

        [hashtable]$Context = @{},

        [string]$RequestId
    )

    $params = @{ Level = 'ERROR'; Message = $Message; Context = $Context }
    if ($RequestId) { $params['RequestId'] = $RequestId }
    Write-LogJson @params
}

<#
.SYNOPSIS
Write a DEBUG level log entry.

.PARAMETER Message
Log message

.PARAMETER Context
Structured context data

.PARAMETER RequestId
Optional correlation ID

.EXAMPLE
Write-LogDebug "Checking pipe status" @{ pipeName = "dinoforge-main" }
#>
function Write-LogDebug {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Message,

        [hashtable]$Context = @{},

        [string]$RequestId
    )

    $params = @{ Level = 'DEBUG'; Message = $Message; Context = $Context }
    if ($RequestId) { $params['RequestId'] = $RequestId }
    Write-LogJson @params
}

<#
.SYNOPSIS
Get the path to the JSONL log file.

.EXAMPLE
$logPath = Get-LogFilePath
#>
function Get-LogFilePath {
    param(
        [string]$LogDir = "$(Ensure-LogDirectory)"
    )
    return Join-Path $LogDir "dinoforge.jsonl"
}

# Export public functions
Export-ModuleMember -Function @(
    'Write-LogJson',
    'Write-LogInfo',
    'Write-LogWarn',
    'Write-LogError',
    'Write-LogDebug',
    'Get-LogFilePath',
    'Ensure-LogDirectory'
)
