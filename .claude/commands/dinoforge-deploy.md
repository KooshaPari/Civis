# dinoforge-deploy

Quick deploy of the Runtime DLL to BepInEx without restarting the game.

**Usage**: `/dinoforge-deploy [--verify-hash] [--target-path <path>]`

**Arguments**:
- `--verify-hash`: After deploy, compute SHA256 hash and compare with expected (optional)
- `--target-path`: Deploy to a specific game directory instead of the default (optional, rare)

## Purpose

Deploys the latest compiled Runtime DLL to the game's BepInEx folder with optional hash verification.

## Steps

1. **Verify build is current**:
   ```
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release
   ```

2. **Deploy to BepInEx**:
   ```
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
   ```

3. **Get deployed DLL hash** (if `--verify-hash`):
   ```
   Get-FileHash "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll" -Algorithm SHA256
   ```

4. **Report**:
   - Build status (exit code, compiler warnings)
   - DLL file size and timestamp
   - SHA256 hash (if `--verify-hash` requested)
   - Expected vs actual hash match/mismatch (if verification enabled)

## Use When

- Quick iteration cycle: make code change → deploy → reload packs (without full game restart)
- Verifying a DLL deploy succeeded without relaunching the game
- Debugging why your code isn't running (is the DLL actually deployed?)
- Part of a larger debug/test loop where relaunch overhead is high

## Note

This does **not** relaunch the game. Use `/dinoforge-smoke` if you need a full cycle including restart.

## Time

~15 seconds (build + deploy + optional hash).
