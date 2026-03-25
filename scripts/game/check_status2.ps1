Write-Host "=== GAME PROCESS STATUS ==="
Get-Process -Name "Diplomacy*" -ErrorAction SilentlyContinue | Select-Object Name, MainWindowTitle, CPU, Id

Write-Host ""
Write-Host "=== FULL dinoforge_debug.log ==="
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "=== SCENE-RELATED lines from LogOutput.log ==="
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log" -ErrorAction SilentlyContinue | Select-String -Pattern "Scene|scene|NativeMenu|Inject|canvas|Canvas|Settings|Options|Mods|button|Button|PrimeCanvas|InitialGame|main menu|MainMenu" | Select-Object -Last 80
