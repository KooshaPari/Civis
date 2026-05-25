$gameExe = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$gameDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"

# Kill any existing game
Get-Process -EA SilentlyContinue | Where-Object { $_.ProcessName -match 'Diplomacy|UnityCrash' } | ForEach-Object { try { $_.Kill() } catch {} }
Start-Sleep 5

Write-Host "Launching directly..."
$game = Start-Process -FilePath $gameExe -WorkingDirectory $gameDir -PassThru
Write-Host "PID=$($game.Id)"

# Wait 3 seconds for DLLs to load
Start-Sleep 3

# Check if winhttp.dll is loaded
try {
    $modules = (Get-Process -Id $game.Id -EA SilentlyContinue).Modules
    $winhttpModule = $modules | Where-Object { $_.ModuleName -like '*winhttp*' }
    if ($winhttpModule) {
        Write-Host "FOUND winhttp.dll: $($winhttpModule.FileName)"
    } else {
        Write-Host "winhttp.dll NOT FOUND in process modules"
    }
    Write-Host "Total modules: $($modules.Count)"
    $modules | Where-Object { $_.ModuleName -match 'BepInEx|doorstop|mono|unity' } | ForEach-Object { Write-Host "  $($_.ModuleName)" }
} catch { Write-Host "Cannot read modules: $_" }

# Command line
$cmdLine = (Get-CimInstance Win32_Process -Filter "ProcessId=$($game.Id)" -EA SilentlyContinue).CommandLine
Write-Host "Command line: $cmdLine"

# Kill immediately (don't wait for full launch)
Start-Sleep 2
try { $game.Kill() } catch {}
Get-Process -EA SilentlyContinue | Where-Object { $_.ProcessName -match 'Diplomacy|UnityCrash' } | ForEach-Object { try { $_.Kill() } catch {} }
Write-Host "Done"
