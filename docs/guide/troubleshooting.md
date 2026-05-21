---
title: Troubleshooting Common Issues
description: Diagnose and fix common problems when modding with DINOForge
---

# Troubleshooting Common Issues

This guide covers the most common problems you might encounter while creating, testing, and deploying DINOForge mod packs. Each section includes the symptom, likely cause, and step-by-step fixes.

## Game Launches but Mod Doesn't Load

### Symptom
- Game starts normally
- F10 menu doesn't appear (or shows no mods)
- `dinoforge_debug.log` shows no DINOForge entries

### Likely Causes
1. **Stale DLL deployment** — Old Runtime DLL still in memory
2. **Wrong BepInEx path** — DINOForge deployed to plugins/ instead of ecs_plugins/
3. **Runtime dependency missing** — BepInEx or core SDK assembly not found

### Fix

**Step 1: Kill all game instances and clear cache**

```powershell
# PowerShell
Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3
```

**Step 2: Verify DINOForge.Runtime.dll is in the correct path**

```powershell
# Should exist:
"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\ecs_plugins\DINOForge.Runtime.dll"

# Should NOT exist:
"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll"
```

**Step 3: Rebuild and redeploy**

```bash
# From repo root
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
```

**Step 4: Wait 12 seconds, then launch the game and check the log**

```powershell
# Wait for game to fully load
Start-Sleep -Seconds 12

# Check for DINOForge entries
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" -Tail 50
```

If you still see no DINOForge entries, check `BepInEx/LogOutput.log` for exception traces during plugin load.

---

## Pack Fails to Load — Validation Errors

### Symptom
- Pack loads partially or not at all
- `dinoforge_debug.log` shows schema validation errors
- Error message like: `Schema validation failed: Unit 'foo' missing required field 'health_points'`

### Likely Causes
1. **YAML/JSON syntax error** — Missing colons, bad indentation
2. **Missing required fields** — Unit/Building missing required schema properties
3. **Invalid enum value** — Faction name doesn't match known factions
4. **Circular or missing dependency** — Pack depends on non-existent faction/unit

### Fix

**Step 1: Validate your pack YAML before deploying**

```bash
# From repo root
dotnet run --project src/Tools/PackCompiler -- validate packs/<your-pack-id>
```

This will output detailed validation errors with line numbers.

**Step 2: Check the pack.yaml manifest**

```yaml
# Example: packs/my-pack/pack.yaml
id: my-pack
name: My Pack
version: 1.0.0
framework_version: ">=0.1.0 <1.0.0"
author: Your Name
type: content  # content | balance | ruleset | total_conversion

depends_on: []
conflicts_with: []

loads:
  factions: []
  units: []
  buildings: []
```

**Step 3: Verify required fields in your definitions**

Check against the canonical schemas at `C:\Users\koosh\Dino\schemas\`:

```bash
# View the unit schema
cat schemas/unit.schema.json | grep -A 5 '"required"'
```

**Step 4: Common field checks**

- **Units**: Must have `id`, `name`, `health_points`, `faction`, `visual_asset`
- **Buildings**: Must have `id`, `name`, `health_points`, `production_rate`
- **Factions**: Must have `id`, `name`, `color_primary`

**Step 5: Redeploy and check the log**

```bash
dotnet run --project src/Tools/PackCompiler -- build packs/<your-pack-id>
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
```

---

## F9/F10 Keys Not Responding

### Symptom
- Game is running with mod loaded
- F9 (debug overlay) or F10 (mod menu) do nothing
- No error in `dinoforge_debug.log`

### Likely Causes
1. **KeyInputSystem not initialized** — ECS system didn't start
2. **Game window not focused** — Win32 key capture requires window focus
3. **System group not running** — Simulation system group not active during gameplay
4. **Key handler not registered** — C# code isn't listening to Win32 key events

### Fix

**Step 1: Verify the game is in gameplay (not main menu)**

The KeyInputSystem only runs during active gameplay. If you're on the main menu, F9/F10 won't work.

**Step 2: Ensure game window has focus**

Click the game window to ensure it's the active foreground window, then press F9.

**Step 3: Check the debug log for KeyInputSystem entries**

```powershell
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" -Tail 100 | Select-String "KeyInputSystem|OnUpdate"
```

Look for lines like: `[KeyInputSystem] Initialized` or `[KeyInputSystem] OnUpdate tick: 42`

**Step 4: Verify the system group is running**

If KeyInputSystem doesn't appear in the log, the system group may not have started. Check:

```bash
# From game running, press F10 for mod menu
# If menu appears, system group is fine

# If menu doesn't appear, try:
# 1. Exit to main menu and re-enter gameplay
# 2. Verify no mods are conflicting (try disabling other mods)
# 3. Check BepInEx/LogOutput.log for system load errors
```

---

## Hot-Reload Not Triggering

### Symptom
- Changed a pack YAML file
- Expected live reload, but changes don't appear in-game
- No reload indication in `dinoforge_debug.log`

### Likely Causes
1. **HMR signal file not created** — DINOForge isn't watching for changes
2. **Pack file not saved** — Changes not committed to disk
3. **HMR watcher crashed** — File monitor exited silently
4. **File outside watched directory** — Pack not in `BepInEx/dinoforge_packs/`

### Fix

**Step 1: Verify the pack is in the correct directory**

```powershell
# Check that your pack exists here:
dir "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_packs\<pack-id>"
```

**Step 2: Manually trigger a reload using the HMR signal file**

```powershell
# Create the signal file (this tells DINOForge to reload)
$filePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\DINOForge_HotReload"
[System.IO.File]::WriteAllText($filePath, [datetime]::Now.ToString())
```

Wait 2-3 seconds and check `dinoforge_debug.log` for reload entries.

**Step 3: Check for file watcher errors in the log**

```powershell
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" -Tail 50 | Select-String -i "hotreload|filewatcher|reload"
```

**Step 4: Restart the game and try again**

Hot-reload is optional. If it fails, you can always restart the game to pick up changes.

---

## Build Errors: TFM/.NET Version Mismatch

### Symptom
- `dotnet build` fails with error like:
  - `error NETSDK1045: The current .NET SDK does not support targeting .NET 11.0`
  - `Framework version 'net11.0' not found`

### Likely Cause
- Local .NET SDK is outdated — doesn't include .NET 11 preview

### Fix

**Step 1: Check your current .NET version**

```powershell
dotnet --version
```

You need: `11.0.100-preview.2.26159.112` or later.

**Step 2: Install .NET 11 preview**

```powershell
# Download from https://dotnet.microsoft.com/download/dotnet/11.0
# Run the installer and select "Install latest .NET 11 preview"

# OR, if you have dotnet-install script:
# Windows:
.\dotnet-install.ps1 -Channel 11.0 -Version latest -Quality preview

# Linux/macOS:
./dotnet-install.sh --channel 11.0 --version latest --quality preview
```

**Step 3: Verify installation**

```powershell
dotnet --version
# Should output: 11.0.100-preview.2.26159.112 or later
```

**Step 4: Rebuild**

```bash
dotnet clean src/DINOForge.sln
dotnet build src/DINOForge.sln -c Release
```

**Reference**: See [.NET Version Policy in CLAUDE.md](https://github.com/KooshaPari/Dino/blob/main/CLAUDE.md#net-version-policy-mandatory--do-not-change-without-checking)

---

## Tests Hang or TestHost Crashes

### Symptom
- `dotnet test` hangs and doesn't complete after 30+ seconds
- `testhost.exe` crashes with exit code 139/134
- Tests work locally but fail in CI

### Likely Causes
1. **Background thread deadlock** — Code calling `Resources.FindObjectsOfTypeAll` off main thread
2. **Memory pressure** — Too many tests running in parallel, exhausting heap
3. **Async task not completing** — Test waits forever for a mock response
4. **System.Reflection hang** — ECS type discovery hangs on reflection

### Fix

**Step 1: Run a single test to isolate the problem**

```bash
# Run just one test
dotnet test src/Tests/DINOForge.Tests.csproj -k "NameOfTest" --verbosity normal
```

**Step 2: Disable parallel test execution**

```bash
# Run tests serially (slower but helps identify deadlocks)
dotnet test src/Tests/DINOForge.Tests.csproj -x --logger:console --verbosity detailed
```

**Step 3: Check for background thread reflection calls**

If the hang is in integration tests, look for:

```csharp
// BAD: Off-thread reflection
Task.Run(() =>
{
    var objs = Resources.FindObjectsOfTypeAll<MyComponent>();  // DEADLOCK!
});

// GOOD: Main thread only
var objs = Resources.FindObjectsOfTypeAll<MyComponent>();
```

See `project_dino_runtime_execution_model.md` in docs/sessions for full details on thread safety.

**Step 4: Increase test timeout and memory**

```bash
# Give tests more time
dotnet test src/Tests/DINOForge.Tests.csproj -x --logger:console --verbosity normal --maxcpucount=1
```

**Step 5: Check BepInEx/LogOutput.log for crash signatures**

```powershell
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log" -Tail 100
```

---

## MCP Server Connection Issues

### Symptom
- Claude Code / IDE can't connect to MCP bridge
- Error: `Connection refused` or `ECONNREFUSED 127.0.0.1:8765`
- Game automation tools won't run (`/game-test`, `/launch-game` fail)

### Likely Causes
1. **MCP server not started** — Python FastMCP server not running
2. **Port 8765 in use** — Another process claimed the port
3. **Virtual environment not activated** — Python dependencies missing

### Fix

**Step 1: Start the MCP server**

```powershell
# From repo root
cd src/Tools/DinoforgeMcp
python -m dinoforge_mcp.server

# Should output:
# [INFO] uvicorn: Uvicorn running on http://127.0.0.1:8765
```

**Step 2: Check if port 8765 is already in use**

```powershell
# Windows: Check what's listening on 8765
netstat -ano | Select-String "8765"

# If something is there, kill it:
Get-Process | Where-Object { $_.Port -eq 8765 } | Stop-Process -Force
```

**Step 3: Verify dependencies are installed**

```bash
cd src/Tools/DinoforgeMcp
pip install -r requirements.txt
```

**Step 4: Test the connection**

```bash
# From another terminal
curl http://127.0.0.1:8765/health

# Should return: {"status": "ok", "version": "0.24.0"}
```

**Step 5: Keep the server running during testing**

The MCP server must stay alive for game automation to work. Consider running it in a separate terminal or in detached mode:

```powershell
# Start detached (runs in background)
& scripts/start-mcp.ps1 -Action start -Detached
```

---

## Goldberg/Steamworks Emulator Questions

### Symptom
- "Can I use Steamless DINO with Goldberg emulator?"
- "Should I disable Steam for mod testing?"

### Answer

**Short**: DINOForge is **Steam-agnostic**. You can use Goldberg, Steamless, or Steam directly.

**Details**:

- DINOForge runs inside BepInEx, which runs inside the game after Unity initializes — completely independent of Steam's login/DRM layer
- Asset swaps, pack loading, ECS queries, and all mod features work identically with or without Steam
- For **testing without Steam**: Use Goldberg (`DLL + .ini` config) — BepInEx will still load normally
- For **CI/CD headless testing**: Stream the game directly via Parsec/RDP, or use the playCUA backend (see MCP Bridge guide)

**No changes needed to packs or C# code** — Steam presence is transparent to DINOForge.

---

## Still Stuck?

If your issue isn't covered above:

1. **Check the debug log**: `BepInEx/dinoforge_debug.log` (detailed) and `BepInEx/LogOutput.log` (exception traces)
2. **Search the issue tracker**: [github.com/KooshaPari/Dino/issues](https://github.com/KooshaPari/Dino/issues)
3. **Consult the reference docs**:
   - [CLAUDE.md - Governance & Architecture](https://github.com/KooshaPari/Dino/blob/main/CLAUDE.md)
   - [Modding DX Reference](/reference/modding-dx-reference)
   - [ECS Bridge Layer](/concepts/ecs-bridge)
4. **File a new issue** with your `dinoforge_debug.log` and pack YAML attached
