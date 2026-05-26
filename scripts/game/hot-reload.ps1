#!/usr/bin/env pwsh
# DINOForge HMR: build Runtime DLL, deploy, signal game to soft-reload UI + packs
param([switch]$Watch)

$repoRoot = "C:\Users\koosh\Dino"
$bepinexDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx"
$signalFile = "$bepinexDir\DINOForge_HotReload"

function Invoke-HotReload {
    Write-Host "[HMR] Building..." -ForegroundColor Cyan
    dotnet build "$repoRoot\src\Runtime\DINOForge.Runtime.csproj" -c Release -p:DeployToGame=true -v quiet 2>&1 | Select-String -NotMatch "^$"
    if ($LASTEXITCODE -eq 0) {
        "" | Set-Content $signalFile
        Write-Host "[HMR] Deployed + signaled. Game reloading UI + packs." -ForegroundColor Green

        # Notify MCP server of reload (if running in HTTP mode)
        try {
            $response = Invoke-RestMethod -Uri "http://127.0.0.1:8765/hmr" -Method POST -TimeoutSec 2 -ErrorAction SilentlyContinue
            if ($response.success) {
                Write-Host "[HMR] MCP server notified of reload (pack caches cleared)" -ForegroundColor Green
            }
        }
        catch {
            # MCP server not running in HTTP mode — that's OK, just log it
            Write-Host "[HMR] MCP server not running in HTTP mode (or not accessible) — pack caches may be stale" -ForegroundColor Yellow
        }
    } else {
        Write-Host "[HMR] Build FAILED" -ForegroundColor Red
    }
}

if ($Watch) {
    $w = New-Object System.IO.FileSystemWatcher "$repoRoot\src" "*.cs"
    $w.IncludeSubdirectories = $true; $w.EnableRaisingEvents = $true
    $action = { Start-Sleep -Milliseconds 500; Invoke-HotReload }
    Register-ObjectEvent $w Changed -Action $action | Out-Null
    Write-Host "[HMR] Watch mode on src/**/*.cs — Ctrl+C to stop" -ForegroundColor Yellow
    while ($true) { Start-Sleep 1 }
} else { Invoke-HotReload }
