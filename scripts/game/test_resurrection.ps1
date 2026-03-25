# Kill all instances
taskkill /F /IM "Diplomacy is Not an Option.exe" 2>$null | Out-Null
Start-Sleep -Seconds 3

# Verify no processes remain
Write-Host "=== Checking for running processes ==="
Get-Process -Name "Diplomacy*" -ErrorAction SilentlyContinue | Select-Object Name

# Clear the debug log
$logPath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
if (Test-Path $logPath) {
    Clear-Content $logPath
    Write-Host "Debug log cleared"
}

# Launch the game
$gameDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$gameExe = "$gameDir\Diplomacy is Not an Option.exe"
Write-Host "Launching game from: $gameDir"
Start-Process -FilePath $gameExe -WorkingDirectory $gameDir

# Wait for game to run
Write-Host "Waiting 120 seconds for game to run..."
Start-Sleep -Seconds 120

# Get final status
Write-Host "`n=== FINAL STATUS AFTER 120s ==="
Get-Process -Name "Diplomacy*" -ErrorAction SilentlyContinue | Select-Object Name, MainWindowTitle, CPU

Write-Host "`n=== LAST 50 LINES OF DEBUG LOG ==="
if (Test-Path $logPath) {
    Get-Content $logPath | Select-Object -Last 50
} else {
    Write-Host "Debug log not found at: $logPath"
}
