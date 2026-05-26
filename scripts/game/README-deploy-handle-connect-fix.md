# deploy-handle-connect-fix.ps1

**Crisis & Fix**: Iteration 142 identified that `GameBridgeServer.HandleConnect()` was never invoked when players joined, causing `InvalidOperationException: Object reference not set to an instance of an object` (null bridge). The fix re-wires the RPC handler in the Bridge/Server layer.

## Prerequisites

- **Git branch**: `fix/handle-connect-iter142` must be merged into your current branch (or main)
- **Game install**: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\` (customizable via `-GameInstallPath` parameter)
- **Administrator access**: Required to kill running game process
- **No game running**: Script will auto-kill; respect the 3-second grace period

## Usage

### Basic Deploy
```powershell
pwsh scripts/game/deploy-handle-connect-fix.ps1
```

### Custom Game Path
```powershell
pwsh scripts/game/deploy-handle-connect-fix.ps1 -GameInstallPath "D:\Games\DINO"
```

### Verify Deployed DLL Only (No Build)
```powershell
pwsh scripts/game/deploy-handle-connect-fix.ps1 -VerifyOnly
```

### Skip Auto-Kill (if game already stopped)
```powershell
pwsh scripts/game/deploy-handle-connect-fix.ps1 -SkipKill
```

## What It Does

1. **Kill the game** — Forces `Diplomacy is Not an Option` process to exit (releases DLL file lock)
2. **Build with auto-deploy** — Runs `dotnet build ... -p:DeployToGame=true` which copies the Runtime DLL into `BepInEx/plugins/`
3. **Verify deployment** — Confirms the DLL exists and is fresh (LastWriteTime is recent)
4. **Sanity check** — Scans the DLL binary for `HandleConnect` symbol (confirms the fix is present)

## What It Does NOT Do

- **Launch the game** — Per CLAUDE.md governance, the script stops before launch. You must launch manually via Steam.
- **Tail logs** — Script prints the log path; you manually run `Get-Content ... -Wait` to monitor
- **Commit/push** — No git operations. Script is deployment-only.
- **Test the fix** — You must verify in-game: join a multiplayer game and confirm no crash on `HandleConnect`

## Troubleshooting

### Build fails with "DLL locked"
- Game process is still running. Check Task Manager or use `Get-Process -Name 'Diplomacy*'`
- Re-run script (auto-kill should catch it)

### "HandleConnect symbol NOT found" warning
- May indicate the fix branch hasn't merged yet, or the build is stale
- Re-run with `-SkipKill` after manually pulling latest

### Deployed DLL is old (age > 5 min)
- Build may have skipped deployment. Check `Directory.Build.props` for `GameInstallPath`
- Manually copy: `Copy-Item src/Runtime/bin/Release/net11.0/DINOForge.Runtime.dll "$GameInstallPath\BepInEx\plugins\"`

## Success Criteria

After deploying and launching the game:
1. Game starts without crashing
2. Tail log (`dinoforge_debug.log`) shows `HandleConnect` invoked (no `InvalidOperationException`)
3. Players can join/leave without "Object reference not set" errors

## Reference

- **Fix details**: `fix/handle-connect-iter142` branch (iter-142 task)
- **Log path**: `$GameInstallPath\BepInEx\dinoforge_debug.log`
- **Deploy governance**: CLAUDE.md § Deploying Fixes
