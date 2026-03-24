Write-Host "=== GAME PROCESS STATUS ==="
Get-Process -Name "Diplomacy*" -ErrorAction SilentlyContinue | Select-Object Name, MainWindowTitle, CPU, Id

Write-Host ""
Write-Host "=== dinoforge_debug.log LAST 80 LINES ==="
$debugLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
if (Test-Path $debugLog) {
    $content = Get-Content $debugLog
    Write-Host "Total lines: $($content.Count)"
    $content | Select-Object -Last 80
} else {
    Write-Host "DEBUG LOG NOT FOUND"
}

Write-Host ""
Write-Host "=== LogOutput.log LAST 60 LINES ==="
$bepLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log"
if (Test-Path $bepLog) {
    $content = Get-Content $bepLog
    Write-Host "Total lines: $($content.Count)"
    $content | Select-Object -Last 60
} else {
    Write-Host "BEPINEX LOG NOT FOUND"
}

Write-Host ""
Write-Host "=== ALL SCENE TRANSITIONS ==="
if (Test-Path $debugLog) {
    Get-Content $debugLog | Select-String -Pattern "OnActiveSceneChanged|OnSceneLoaded|OnSceneUnloaded|Scene changed"
}
