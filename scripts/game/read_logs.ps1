# Read and filter DINOForge logs
$debugLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
$bepLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log"

Write-Host "=== DINOFORGE DEBUG LOG (non-heartbeat, last 500 lines) ==="
$lines = Get-Content $debugLog -Tail 500
foreach ($line in $lines) {
    if ($line -notmatch "heartbeat") {
        Write-Host $line
    }
}

Write-Host ""
Write-Host "=== LOGOUTPUT.LOG ERRORS/WARNINGS (last 200 lines) ==="
$bepLines = Get-Content $bepLog -Tail 200
foreach ($line in $bepLines) {
    if ($line -match "Error|Warning|Exception|WARN|ERROR|\[Error") {
        Write-Host $line
    }
}
