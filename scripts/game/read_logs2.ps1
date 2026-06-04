# Read startup errors from dinoforge_debug.log
$debugLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"

Write-Host "=== DINOFORGE DEBUG LOG - ERROR/WARN lines (all) ==="
$lines = Get-Content $debugLog
foreach ($line in $lines) {
    if ($line -match "\[Error\]|\[Warn\]|ERROR|WARN|Exception|NRE|failed|Failed|null ref|NullRef") {
        Write-Host $line
    }
}

Write-Host ""
Write-Host "=== LOGOUTPUT.LOG - ERROR/WARN lines (all) ==="
$bepLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log"
$blines = Get-Content $bepLog
foreach ($line in $blines) {
    if ($line -match "^\[Error|^\[Warning") {
        Write-Host $line
    }
}
