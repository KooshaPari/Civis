<#
.SYNOPSIS
    Manage the Civis local dev stack (process-compose).

.PARAMETER Action
    up | down | status | logs

.PARAMETER Service
    Optional service name for `logs`.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('up', 'down', 'status', 'logs')]
    [string]$Action,

    [string]$Service = ''
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot
$ComposeFile = Join-Path $RepoRoot 'process-compose.yaml'
$LogDir = Join-Path $RepoRoot '.process-compose/logs'
$Port = 18080

if (-not (Get-Command process-compose -ErrorAction SilentlyContinue)) {
    Write-Host "[dev] process-compose not found in PATH." -ForegroundColor Red
    Write-Host "[dev] Install: https://f1bonacc1.github.io/process-compose/" -ForegroundColor Yellow
    exit 1
}

function Get-Running {
    try {
        $r = & process-compose process list --port $Port 2>&1
        return ($LASTEXITCODE -eq 0)
    } catch { return $false }
}

switch ($Action) {
    'up' {
        New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
        if (Get-Running) {
            Write-Host "[dev] Stack already running on port $Port." -ForegroundColor Yellow
            & process-compose process list --port $Port
            exit 0
        }
        Write-Host "[dev] Starting backing services (PG, DragonFly, NATS, MinIO, civ-watch)..." -ForegroundColor Cyan
        Push-Location $RepoRoot
        try {
            Start-Process -FilePath 'process-compose' `
                -ArgumentList @('up', '-f', $ComposeFile, '--port', $Port, '--tui=false', '--detached-with-tui=false', '--keep-tui') `
                -NoNewWindow -PassThru | Out-Null
        } finally { Pop-Location }

        # Wait up to 30s for the API to come up
        $deadline = (Get-Date).AddSeconds(30)
        while ((Get-Date) -lt $deadline) {
            if (Get-Running) {
                Write-Host "[dev] Stack ready on port $Port." -ForegroundColor Green
                & process-compose process list --port $Port
                exit 0
            }
            Start-Sleep -Milliseconds 500
        }
        Write-Host "[dev] Timed out waiting for process-compose API." -ForegroundColor Red
        exit 1
    }
    'down' {
        if (-not (Get-Running)) {
            Write-Host "[dev] Stack not running." -ForegroundColor Yellow
            exit 0
        }
        Write-Host "[dev] Stopping stack..." -ForegroundColor Cyan
        & process-compose down --port $Port
        exit $LASTEXITCODE
    }
    'status' {
        if (-not (Get-Running)) {
            Write-Host "[dev] Stack not running on port $Port." -ForegroundColor Yellow
            exit 1
        }
        & process-compose process list --port $Port
        exit $LASTEXITCODE
    }
    'logs' {
        if ($Service) {
            $f = Join-Path $LogDir "$Service.log"
            if (-not (Test-Path $f)) { Write-Host "[dev] No log: $f" -ForegroundColor Red; exit 1 }
            Get-Content -Path $f -Wait -Tail 50
        } else {
            Get-ChildItem $LogDir -Filter '*.log' | ForEach-Object {
                Write-Host "=== $($_.Name) ===" -ForegroundColor Cyan
                Get-Content $_.FullName -Tail 20
            }
        }
    }
}
