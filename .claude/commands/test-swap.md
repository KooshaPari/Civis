# test-swap

End-to-end asset swap test: build → deploy → launch → wait → verify results in debug log.

**Usage**: `/test-swap [--pack <pack-id>] [--no-launch]`

**Arguments**: $ARGUMENTS

## Purpose

Automates the full loop for testing visual asset swaps: builds the Runtime DLL, deploys it to the game, optionally launches a fresh game instance, waits for the swap window, then reads and reports results.

## Steps

1. **Build & deploy**
   ```bash
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
   ```
   Report build success/failure. Abort on build error.

2. **Check what bundles exist** (for the target pack, default `warfare-starwars`):
   ```bash
   ls "G:\SteamLibrary\...\BepInEx\dinoforge_packs\<pack-id>\assets\bundles\"
   ```
   Report how many bundle files are present.

3. **Launch game** (skip if `--no-launch` or game already running via MCP):
   Launch `"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"` as background process.

4. **Wait for world + swap window**:
   Poll `BepInEx/dinoforge_debug.log` every 5s for up to 90s for:
   - `AssetSwapSystem.OnCreate` → world initialized
   - `AssetSwapSystem: RenderMesh entities present` → swap firing
   - `swap complete` or `batch complete` → results ready

5. **Read and report results**:
   - Count `swap complete` entries → succeeded
   - Count `live swap pending` entries → awaiting entities
   - List any `swap exception` entries
   - Check `TrySwapRenderMeshFromBundle: swapped N/M entities` counts
   - Report final summary

6. **Take screenshot** if MCP bridge is connected:
   Call `game_screenshot` and display the result path.

## Success Criteria

- At least 1 `swap complete` in the log
- `swapped N/M entities` shows N > 0
- No uncaught exceptions
