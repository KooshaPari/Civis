# dinoforge-smoke

Run a full build-deploy-relaunch-verify smoke test end-to-end.

**Usage**: `/dinoforge-smoke`

## Purpose

Validates DINOForge is working correctly by building, deploying, relaunching the game, and verifying packs load.

## Steps

1. **Build**: Compile the Runtime DLL (netstandard2.0 target)
   ```
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release
   ```

2. **Deploy**: Copy to BepInEx and verify SHA256 hash
   ```
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
   Get-FileHash "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll" -Algorithm SHA256
   ```

3. **Relaunch**: Kill any running game and start fresh
   ```
   Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue
   Start-Sleep 3
   Start-Process -FilePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
   ```

4. **Verify**: Wait 8 seconds, then check via GameClient RPC:
   - Game process is running (MainWindowTitle != "Fatal error")
   - ECS World is ready (system count > 0)
   - Packs are loaded (check dinoforge_debug.log for ContentLoader entries)
   - No critical errors in LogOutput.log

5. **Report**:
   - Build status (exit code)
   - DLL timestamp and hash
   - Game process status (running / not running)
   - Pack load count from debug log
   - Any errors or warnings

## Use When

- Validating major changes before committing
- After pulling new commits and want to verify they integrate cleanly
- Debugging "did my change actually deploy?" questions
- Before opening a pull request
- Daily sanity check of the develop branch

## Time

~45 seconds total (build + deploy + relaunch + settle + verify).
