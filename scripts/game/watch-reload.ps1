# watch-reload.ps1 - Continuous file watcher: auto-build/deploy/signal on code changes
# Usage: .\scripts\game\watch-reload.ps1
# Stop: Press Ctrl+C

$ErrorActionPreference = "Stop"

# Get repo root
$scriptDir = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSCommandPath))
$RepoRoot = Split-Path -Parent $scriptDir
$SrcDir = "$RepoRoot\src"

# Game paths
$GameDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$BepInExDir = "$GameDir\BepInEx"
$SignalFile = "$BepInExDir\DINOForge_SoftReload"
$RuntimeCsproj = "$RepoRoot\src\Runtime\DINOForge.Runtime.csproj"

# Track last build time to debounce
$global:lastBuild = [DateTime]::MinValue
$debounceMs = 1000

Write-Host "[watch-reload] Starting file watcher..." -ForegroundColor Cyan
Write-Host "[watch-reload] Watching: $SrcDir" -ForegroundColor Cyan
Write-Host "[watch-reload] Signal file: $SignalFile" -ForegroundColor Cyan
Write-Host "[watch-reload] Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# Create watcher
$watcher = New-Object System.IO.FileSystemWatcher
$watcher.Path = $SrcDir
$watcher.Filter = "*.cs"
$watcher.IncludeSubdirectories = $true
$watcher.NotifyFilter = [System.IO.NotifyFilters]::LastWrite

# Define build action
$buildAction = {
    $now = [DateTime]::UtcNow
    $elapsed = ($now - $global:lastBuild).TotalMilliseconds

    # Skip if too soon (debounce)
    if ($elapsed -lt $debounceMs) {
        return
    }

    $global:lastBuild = $now

    $fileName = $Event.SourceEventArgs.Name
    Write-Host "[watch-reload] Changed: $fileName" -ForegroundColor Yellow
    Write-Host "[watch-reload] Building Runtime..." -ForegroundColor Cyan

    # Build
    $buildOutput = dotnet build "$RuntimeCsproj" -c Release -p:DeployToGame=true 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Host "[watch-reload] Build succeeded" -ForegroundColor Green

        # Signal soft-reload
        Write-Host "[watch-reload] Sending HMR signal..." -ForegroundColor Cyan
        "" | Out-File -FilePath $SignalFile -Encoding ASCII -Force
        Write-Host "[watch-reload] HMR signal sent! Game will reload." -ForegroundColor Green
    } else {
        Write-Host "[watch-reload] Build FAILED" -ForegroundColor Red
        $buildOutput | Select-Object -Last 5 | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    }

    Write-Host ""
}

# Register event handlers
Register-ObjectEvent -InputObject $watcher -EventName "Changed" -Action $buildAction | Out-Null
Register-ObjectEvent -InputObject $watcher -EventName "Created" -Action $buildAction | Out-Null

# Start watching
$watcher.EnableRaisingEvents = $true

Write-Host "[watch-reload] Watcher active. Waiting for file changes..." -ForegroundColor Green
Write-Host ""

try {
    while ($true) { Start-Sleep -Seconds 1 }
} finally {
    $watcher.EnableRaisingEvents = $false
    $watcher.Dispose()
    Write-Host "[watch-reload] Stopped." -ForegroundColor Cyan
}
