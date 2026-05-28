#Requires -Version 7.0
param(
    [string]$EnvFile = (Join-Path $PSScriptRoot 'omniroute.env'),
    [string]$BaseUrl,
    [int]$TimeoutSec = 30
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path -LiteralPath $EnvFile)) {
    Write-Error "Missing $EnvFile — copy scripts/omniroute.env.example"
}

Get-Content -LiteralPath $EnvFile | ForEach-Object {
    if ($_ -match '^\s*([^#][^=]+)=(.*)$') {
        Set-Item -Path "env:$($matches[1].Trim())" -Value $matches[2].Trim()
    }
}

if (-not $env:OMNROUTE_BASE_URL -or -not $env:OMNROUTE_API_KEY) {
    Write-Error 'OMNROUTE_BASE_URL and OMNROUTE_API_KEY must be set'
}

if ($BaseUrl) { $env:OMNROUTE_BASE_URL = $BaseUrl }
$base = $env:OMNROUTE_BASE_URL.TrimEnd('/')
$modelsUrl = if ($base -match '/v1$') { "$base/models" } else { "$base/v1/models" }
$headers = @{ Authorization = "Bearer $env:OMNROUTE_API_KEY" }

Write-Host "GET $modelsUrl"
$response = Invoke-RestMethod -Uri $modelsUrl -Headers $headers -TimeoutSec $TimeoutSec
$count = @($response.data).Count
Write-Host "OK — $count models"
