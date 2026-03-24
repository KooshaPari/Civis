# /launch-game

Kill any existing game instance, then launch fresh for testing.

## What this does
1. Kills all running game processes
2. Clears the debug log
3. Launches the game exe directly
4. Waits for BepInEx + DINOForge to initialize

## Why "another instance" is no longer a problem
`boot.config` has `single-instance=0` (was `single-instance=` which Unity treated as truthy).
Unity's native single-instance check is now permanently disabled. No mutex bypass needed.

## Steps

### 1. Kill existing processes
```powershell
Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "UnityCrashHandler64" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

# Verify clean
$remaining = Get-Process | Where-Object { $_.Name -like "*Diplomacy*" }
if ($remaining) { $remaining | Stop-Process -Force; Start-Sleep -Seconds 2 }
```

### 2. Clear debug log
```powershell
Clear-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" -ErrorAction SilentlyContinue
```

### 3. Launch directly
```powershell
Start-Process `
  -FilePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe" `
  -WorkingDirectory "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
```

### 4. Wait for initialization
```powershell
$debugLog = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
$timeout = 30; $elapsed = 0
while ($elapsed -lt $timeout) {
    Start-Sleep -Seconds 2; $elapsed += 2
    if ((Get-Content $debugLog -ErrorAction SilentlyContinue) -match "Awake completed") {
        Write-Host "DINOForge loaded"
        break
    }
}
```

## Flags
- `--wait-for-world` — also wait for ECS world creation (`KeyInputSystem.OnCreate` in log)
- `--record` — capture to `%TEMP%\dinoforge_proof.mp4` using ffmpeg

## Notes
- `boot.config` fix file: `Diplomacy is Not an Option_Data\boot.config` — `single-instance=0`
- If the game is updated via Steam, Steam may restore `boot.config` with `single-instance=` — reapply the fix if the error returns
