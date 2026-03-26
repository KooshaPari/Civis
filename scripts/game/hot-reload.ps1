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
