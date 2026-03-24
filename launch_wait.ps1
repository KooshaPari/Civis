Write-Host "Waiting 300 seconds for DINO to fully load..."
Start-Sleep -Seconds 300
Write-Host "Wait complete. Checking game status..."
Get-Process -Name "Diplomacy*" -ErrorAction SilentlyContinue | Select-Object Name, MainWindowTitle, CPU, Id
Write-Host "--- BepInEx LogOutput.log ---"
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log" -ErrorAction SilentlyContinue
Write-Host "--- DINOForge debug log (last 40 lines) ---"
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" -ErrorAction SilentlyContinue | Select-Object -Last 40
