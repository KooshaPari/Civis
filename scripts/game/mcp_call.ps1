$ErrorActionPreference = "Stop"
param(
    [Parameter(Mandatory=$true)]
    [string]$Method,
    [string]$Arguments = "{}"
)

$body = @{
    jsonrpc = "2.0"
    id = 1
    method = $Method
    params = ConvertFrom-Json $Arguments
} | ConvertTo-Json -Compress

$response = Invoke-RestMethod -Uri "http://127.0.0.1:8765/messages" -Method POST -ContentType "application/json" -Body $body -TimeoutSec 30
$response | ConvertTo-Json -Depth 10
