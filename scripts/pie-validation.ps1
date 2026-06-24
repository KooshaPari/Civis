#Requires -Version 5.1
<#
.SYNOPSIS
  Agent-runnable Unreal PIE prep: backends, protocol smoke, terrain HTTP, offline UE preflight, human PIE checklist.

.DESCRIPTION
  Does not drive the UE Editor GUI. Starts civ-server / civ-watch only when ports 3000 / 9090 are free.

.EXIT CODES
  0  All automated checks passed
  1  A step failed
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ServerPort = if ($env:CIV_SERVER_PORT) { [int]$env:CIV_SERVER_PORT } else { 3000 }
$WatchPort = if ($env:CIV_WATCH_PORT) { [int]$env:CIV_WATCH_PORT } else { 9090 }
$StartedBackends = @()

function Test-PortListening([int] $Port) {
    $conn = @(Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue |
        Where-Object { $_.LocalAddress -eq '127.0.0.1' -or $_.LocalAddress -eq '0.0.0.0' -or $_.LocalAddress -eq '::' })
    return $conn.Length -gt 0
}

function Start-BackendIfNeeded([string] $Name, [int] $Port, [string] $Package) {
    if (Test-PortListening $Port) {
        Write-Host "==> $Name already listening on port $Port" -ForegroundColor DarkGray
        return
    }
    Write-Host "==> Starting $Name (cargo run -p $Package) on port $Port" -ForegroundColor Cyan
    $job = Start-Job -Name $Name -ScriptBlock {
        param($Root, $Pkg)
        Set-Location $Root
        cargo run -p $Pkg -q 2>&1
    } -ArgumentList $RepoRoot, $Package
    $script:StartedBackends += [pscustomobject]@{ Name = $Name; Port = $Port; Job = $job }
}

function Wait-BackendReady([int] $Port, [int] $TimeoutSec = 90) {
    $deadline = (Get-Date).AddSeconds($TimeoutSec)
    while ((Get-Date) -lt $deadline) {
        if (Test-PortListening $Port) { return }
        Start-Sleep -Milliseconds 500
    }
    throw "Port $Port did not enter Listen state within ${TimeoutSec}s"
}

function Invoke-Step([string] $Label, [scriptblock] $Action) {
    Write-Host ''
    Write-Host "==> $Label" -ForegroundColor Cyan
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed (exit $LASTEXITCODE)"
    }
}

function Show-PieChecklist() {
    Write-Host ''
    Write-Host '==> Play in Editor — human steps (from fr-unreal-agent-playbook.md)' -ForegroundColor Magenta
    Write-Host 'Prerequisites: civ-server + civ-watch running (this script may have started them).' -ForegroundColor DarkGray
    @(
        'Open clients/unreal-show/CivShow.uproject in UE Editor — project loads without compile errors.'
        'Edit -> Project Settings -> Maps & Modes — default Game Mode = CivShowGameMode (or set on World Settings).'
        'Press Play (PIE) — no crash; terrain mesh appears from civ-watch HTTP.'
        'Confirm WS attach — UCivWsClient to ws://127.0.0.1:3000/ws?tick_format=binary; sim.snapshot and civ_pins arrive.'
        'Pins — spawn on server (RPC or another client); cylinder civilians appear at normalized positions.'
        'Day/night — toggle sim day phase or wait for is_day flip; directional light + ambient follow ApplyDayNight.'
        'Job colors — spawn civilians with Citizen.job when wired; verify CivisJobColors tint path does not crash.'
    ) | ForEach-Object -Begin { $i = 1 } -Process {
        Write-Host "  $i. $_"
        $i++
    }
    Write-Host ''
    Write-Host 'Quick smoke: terrain within ~1 s; civilian pin after spawn RPC.' -ForegroundColor DarkGray
}

Push-Location $RepoRoot
try {
    Start-BackendIfNeeded 'civ-server' $ServerPort 'civ-server'
    Start-BackendIfNeeded 'civ-watch' $WatchPort 'civ-watch'

    if ($StartedBackends.Count -gt 0) {
        foreach ($b in $StartedBackends) {
            Wait-BackendReady $b.Port
            Write-Host "    $($b.Name) ready on port $($b.Port)" -ForegroundColor Green
        }
        Write-Host ''
        Write-Host 'NOTE: Stop background backends when PIE is done (Stop-Job / Remove-Job or close their windows):' -ForegroundColor Yellow
        foreach ($b in $StartedBackends) {
            Write-Host "  Stop-Job -Name $($b.Name); Remove-Job -Name $($b.Name)" -ForegroundColor Yellow
        }
    }

    Invoke-Step 'civ-server WS spawn pin smoke' {
        & cargo test -p civ-server --test ws_smoke ws_jsonrpc_spawn_civilian_pin --quiet
    }

    Invoke-Step 'civ-watch GET /terrain' {
        $uri = "http://127.0.0.1:${WatchPort}/terrain"
        try {
            $resp = Invoke-WebRequest -Uri $uri -Method Get -TimeoutSec 5 -UseBasicParsing
        }
        catch {
            if ($_.Exception.Response) {
                $status = [int]$_.Exception.Response.StatusCode
                throw "GET $uri returned $status (expected 200)"
            }
            throw "GET $uri failed: $($_.Exception.Message)"
        }
        if ($resp.StatusCode -ne 200) {
            throw "GET $uri returned $($resp.StatusCode) (expected 200)"
        }
        Write-Host "    $($resp.StatusCode) OK ($($resp.RawContentLength) bytes)" -ForegroundColor Green
    }

    $verify = Join-Path $RepoRoot 'clients\unreal-show\scripts\verify-unreal-ready.ps1'
    Invoke-Step 'Unreal offline preflight (verify-unreal-ready.ps1)' {
        & powershell -NoProfile -ExecutionPolicy Bypass -File $verify
    }

    Show-PieChecklist

    Write-Host ''
    Write-Host '==> pie-validation passed' -ForegroundColor Green
    exit 0
}
catch {
    Write-Host ''
    Write-Host "==> pie-validation FAILED: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
finally {
    Pop-Location
}
