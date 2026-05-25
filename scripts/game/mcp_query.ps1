$ErrorActionPreference = "Stop"
$ToolName = $args[0]
if (-not $ToolName) { $ToolName = "game_status" }
$ArgumentsJson = if ($args[1]) { $args[1] } else { "{}" }
$MaxRetries = if ($args[2]) { [int]$args[2] } else { 12 }
$RetryDelaySec = if ($args[3]) { [int]$args[3] } else { 5 }

$body = @{
    jsonrpc = "2.0"
    id = 1
    method = "tools/call"
    params = @{
        name = $ToolName
        arguments = (ConvertFrom-Json $ArgumentsJson)
    }
} | ConvertTo-Json -Compress

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Calling MCP tool: $ToolName"

for ($i = 0; $i -lt $MaxRetries; $i++) {
    try {
        $response = Invoke-RestMethod -Uri "http://127.0.0.1:8765/messages" -Method POST -ContentType "application/json" -Body $body -TimeoutSec 15
        $responseJson = $response | ConvertTo-Json -Depth 10
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Response: $responseJson"
        return
    }
    catch {
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Retry $i/$MaxRetries failed: $($_.Exception.Message)"
        if ($i -lt ($MaxRetries - 1)) {
            Start-Sleep -Seconds $RetryDelaySec
        }
    }
}

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] All retries exhausted."
