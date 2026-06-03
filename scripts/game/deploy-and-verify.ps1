#!/usr/bin/env pwsh
<#
.SYNOPSIS
  Build, deploy, relaunch DINO and wait for swap verification.
  Handles the DLL-lock problem by killing game before copy.
  Usage: ./deploy-and-verify.ps1 [-SkipBuild] [-SkipNavigate]
#>
param(
    [switch]$SkipBuild,
    [switch]$SkipNavigate
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$REPO       = "C:\Users\koosh\Dino"
$SRC_DLL    = "$REPO\src\Runtime\bin\Release\netstandard2.0\DINOForge.Runtime.dll"
$GAME_DIR   = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$DEST_DLL   = "$GAME_DIR\BepInEx\plugins\DINOForge.Runtime.dll"
$GAME_EXE   = "$GAME_DIR\Diplomacy is Not an Option.exe"
$LOG        = "$GAME_DIR\BepInEx\dinoforge_debug.log"
$MCP        = "http://127.0.0.1:8765"

function Invoke-Mcp {
    param([string]$Tool, [hashtable]$Params = @{})
    $body = @{ jsonrpc="2.0"; method="tools/call"; id=1;
               params=@{ name=$Tool; arguments=$Params } } | ConvertTo-Json -Depth 5
    # FastMCP HTTP mode uses SSE; call via the Python helper instead
    $py = @"
import urllib.request, json, sys
req = urllib.request.Request(
    '$MCP/mcp',
    data=json.dumps({'jsonrpc':'2.0','method':'tools/call','id':1,
                     'params':{'name':'$Tool','arguments':$($Params | ConvertTo-Json -Compress)}}).encode(),
    headers={'Content-Type':'application/json','Accept':'application/json, text/event-stream'},
    method='POST')
try:
    with urllib.request.urlopen(req, timeout=60) as r:
        for line in r:
            l = line.decode().strip()
            if l.startswith('data:'):
                print(l[5:].strip())
                break
except Exception as e:
    print(json.dumps({'error': str(e)}))
"@
    $result = python3 -c $py 2>&1
    return $result
}

function Kill-Game {
    Write-Host "  Killing game..."
    Get-Process | Where-Object { $_.Name -like "*Diplomacy*" } |
        Stop-Process -Force -ErrorAction SilentlyContinue
    # Wait until DLL is released (up to 15s)
    $deadline = [DateTime]::Now.AddSeconds(15)
    while ([DateTime]::Now -lt $deadline) {
        Start-Sleep -Milliseconds 500
        try {
            $stream = [System.IO.File]::Open($DEST_DLL, 'Open', 'ReadWrite', 'None')
            $stream.Close()
            Write-Host "  DLL unlocked ✅"
            return $true
        } catch { }
    }
    Write-Host "  ❌ DLL still locked after 15s"
    return $false
}

# ── 1. Build ─────────────────────────────────────────────────────────────────
if (-not $SkipBuild) {
    Write-Host "`n[1/5] Building Runtime..."
    Push-Location $REPO
    $result = dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release `
        -p:TargetFramework=netstandard2.0 2>&1 | Select-String -Pattern "Error\(s\)|Warning\(s\)|Time Elapsed"
    Pop-Location
    if ($LASTEXITCODE -ne 0) { Write-Host "❌ Build failed"; exit 1 }
    Write-Host "  Build OK: $($result -join ' | ')"
}

# ── 2. Kill + Deploy ──────────────────────────────────────────────────────────
Write-Host "`n[2/5] Deploying DLL..."
$unlocked = Kill-Game
if (-not $unlocked) { exit 1 }

Copy-Item -Path $SRC_DLL -Destination $DEST_DLL -Force
$srcH = (Get-FileHash $SRC_DLL).Hash
$dstH = (Get-FileHash $DEST_DLL).Hash
if ($srcH -ne $dstH) { Write-Host "❌ Hash mismatch after copy"; exit 1 }
Write-Host "  Deployed $dstH ✅"

# ── 3. Relaunch ───────────────────────────────────────────────────────────────
Write-Host "`n[3/5] Relaunching game..."
Start-Process -FilePath $GAME_EXE -WorkingDirectory $GAME_DIR
Start-Sleep -Seconds 8
$proc = Get-Process | Where-Object { $_.Name -like "*Diplomacy*" } | Select-Object -First 1
if (-not $proc) { Write-Host "❌ Game did not start"; exit 1 }
Write-Host "  Game PID $($proc.Id) ✅"

# ── 4. Wait for entity population ─────────────────────────────────────────────
Write-Host "`n[4/5] Waiting for entity population (need ≥1000 entities)..."
if (-not $SkipNavigate) {
    # Give game time to reach main menu, then load autosave
    Start-Sleep -Seconds 20
    Write-Host "  Triggering gameplay via MCP game_navigate_to..."
    $nav = Invoke-Mcp -Tool "game_navigate_to" -Params @{ state="gameplay" }
    Write-Host "  Nav result: $nav"
}

# Poll log for swap completion (up to 3 min)
$deadline = [DateTime]::Now.AddSeconds(180)
$swapResult = $null
while ([DateTime]::Now -lt $deadline) {
    Start-Sleep -Seconds 3
    $recent = Get-Content $LOG -Tail 200 -ErrorAction SilentlyContinue |
        Where-Object { $_ -match "batch complete" } | Select-Object -Last 1
    if ($recent -match "batch complete") {
        $swapResult = $recent
        break
    }
}

# ── 5. Report ─────────────────────────────────────────────────────────────────
Write-Host "`n[5/5] Results:"
if ($swapResult) {
    Write-Host "  $swapResult"
    if ($swapResult -match "(\d+) succeeded") {
        $n = [int]$Matches[1]
        if ($n -gt 0) {
            Write-Host "  ✅ $n swaps confirmed in log. User must verify visually."
        } else {
            Write-Host "  ❌ 0 succeeded — check for errors above"
            Get-Content $LOG -Tail 50 | Where-Object { $_ -match "failed|error|exception" } | Select-Object -Last 10
        }
    }
} else {
    Write-Host "  ❌ No swap batch completed within timeout — still at main menu?"
    Get-Content $LOG -Tail 10 | ForEach-Object { Write-Host "  $_" }
}
