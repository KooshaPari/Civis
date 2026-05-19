# MockSteamworksNet

A BepInEx plugin that mocks the Steamworks.NET managed wrapper for headless CI testing.

## Purpose

DINO uses Steamworks.NET to interact with the Steam client. In CI environments (headless, containerized, or automated test runners), Steam is typically unavailable. This plugin intercepts Steamworks.NET method calls at the managed IL level using Harmony patches and returns mock responses, allowing the game to start and run automated tests without requiring:

- A real Steam client installation
- A Goldberg emulator (which Steamworks.NET cannot bypass via raw DLL substitution)
- GUI overhead or window management

## What It Mocks

| Method | Return Value | Purpose |
|--------|--------------|---------|
| `SteamAPI.Init()` | `true` | Allows game to think Steam initialized successfully |
| `SteamAPI.IsSteamRunning()` | `true` | Prevents early-exit checks for missing Steam |
| `SteamUser.GetSteamID()` | `76561197960265728UL` (mock ID) | Provides a valid Steam ID for identity checks |
| `SteamApps.BIsSubscribedApp(uint)` | `true` | Fakes app subscription (allows DLC/base game checks) |
| `SteamFriends.GetPersonaName()` | `"MockUser"` | Provides a dummy username for UI/logs |

## Usage

### Development/Local Testing

Not recommended. If you need to bypass Steam locally, use the real Goldberg emulator (`C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe`).

### CI/Headless Automation

1. **Deploy to BepInEx:**
   ```bash
   # After building
   cp src/Tools/MockSteamworksNet/bin/Release/net8.0/MockSteamworksNet.dll \
      "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\"
   ```

2. **Launch game in headless mode:**
   ```powershell
   # Via hidden desktop (Windows)
   dotnet run --project src/Tools/DinoforgeMcp -- game_launch --hidden=true
   
   # Via MCP tool
   dino mcp game_launch --mode headless
   ```

3. **Plugin activates automatically:**
   - Patches are applied during BepInEx plugin loading (Awake())
   - Logs message: `MockSteamworksNet v1.0.0 loading (headless CI mock mode)...`
   - All subsequent Steamworks.NET calls return mocked values

4. **Verify in logs:**
   ```
   BepInEx\dinoforge_debug.log or BepInEx\LogOutput.log
   ```
   Look for:
   ```
   Patched SteamAPI.Init()
   Patched SteamAPI.IsSteamRunning()
   Patched SteamUser.GetSteamID()
   ...
   MockSteamworksNet patches applied successfully. Headless CI mode active.
   ```

## Architecture

### Harmony Patches

Uses [HarmonyLib](https://harmony.pardeike.net/) (included with BepInEx core) to apply **postfix patches** to Steamworks.NET methods:

- **Postfix** (not prefix): Allows original method to run, then overwrites return value
- **Static methods**: All Steamworks.NET surface APIs are static; patches target them directly
- **Zero imports**: Doesn't reference Steamworks.NET namespace at compile time (only via reflection at runtime)

### Method Resolution

Patches are applied dynamically via reflection:
```csharp
var method = typeof(SteamAPI).GetMethod("Init", BindingFlags.Public | BindingFlags.Static);
_harmony.Patch(method, postfix: new HarmonyMethod(...));
```

This allows the plugin to work even if Steamworks.NET is absent (graceful degradation).

## Limitations

1. **Mocking Only**: Does not implement actual Steam functionality. Callbacks, achievements, stats, or cloud save won't work.
2. **Surface APIs Only**: Deep integrations (friends lists, controller input, networking) are not mocked.
3. **Expansion**: To mock additional Steamworks.NET APIs, add more patches following the existing pattern.

## Building

```bash
dotnet build src/Tools/MockSteamworksNet/MockSteamworksNet.csproj -c Release
```

Requires:
- .NET 8.0 SDK
- BepInEx 5.4.23.5 (referenced from game install)
- HarmonyLib (auto-included with BepInEx)
- Steamworks.NET v13.0.0 (NuGet)

## Testing

To verify the plugin loads and patches correctly:

1. Deploy DLL to `BepInEx/plugins/`
2. Launch game with `-headless` or via hidden desktop
3. Check `BepInEx/dinoforge_debug.log` for:
   - No errors during patch application
   - Confirmation messages for each patched method
   - Game startup proceeds without Steam errors

## Future Work

- Add mocks for `SteamMatchmaking`, `SteamNetworking`, `SteamRemoteStorage` (cloud save)
- Implement callback stubs (currently silent)
- Add environment variable to enable/disable plugin at runtime
- Wire into CI workflow gates for automated verification

## References

- Steamworks.NET: https://github.com/rlabrecque/Steamworks.NET
- HarmonyLib: https://harmony.pardeike.net/
- BepInEx: https://docs.bepinex.dev/
