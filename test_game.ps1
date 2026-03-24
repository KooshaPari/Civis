# Step 2: Verify game is dead
Start-Sleep 3
$p = Get-Process -Name 'Diplomacy is Not an Option' -ErrorAction SilentlyContinue
if ($p) {
    'STILL RUNNING'
} else {
    'DEAD'
}

# Step 3: Clear debug log
Clear-Content 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log' -ErrorAction SilentlyContinue
'Log cleared'

# Step 4: Launch game
Start-Process -FilePath 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe' -WorkingDirectory 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
'Launched'

# Step 5: Wait 25 seconds then check for OnGUI heartbeat
Start-Sleep 25
Write-Host "=== Checking for OnGUI/DebugOverlay ==="
Get-Content 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log' -ErrorAction SilentlyContinue | Select-String 'OnGUI|DebugOverlay'

# Step 6: Show last 10 lines
Write-Host "=== Last 10 lines of log ==="
Get-Content 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log' -ErrorAction SilentlyContinue | Select-Object -Last 10
