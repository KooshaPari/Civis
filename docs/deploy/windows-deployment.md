# Windows Deployment Guide

This guide covers installation and configuration of DINOForge on Windows 10/11.

## System Requirements

Before proceeding, ensure your system meets these requirements:

- **OS**: Windows 10 (build 19041+) or Windows 11
- **.NET Runtime**: .NET 11 (preview) — see [.NET Downloads](https://dotnet.microsoft.com/download/dotnet/11.0)
- **BepInEx**: Version 5.4.23.5 or later (standard GitHub release)
- **Visual C++ Redistributables**: 2015-2022 (required by BepInEx and Unity)
- **Game**: Diplomacy is Not an Option (Steam, fully updated)
- **Disk Space**: 2 GB (runtime + plugins + packs)
- **RAM**: 8 GB minimum (12 GB recommended for testing)

### Checking .NET Installation

Open PowerShell and verify .NET is installed:

```powershell
dotnet --info
```

Look for `.NET 11.0.100-preview` or later in the output. If not present, download from https://dotnet.microsoft.com/download/dotnet/11.0.

## Installation Steps

### Step 1: Verify Game Installation

Locate your DINO installation directory. By default:

```
G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\
```

Check that these exist:
- `Diplomacy is Not an Option.exe` (the game executable)
- `BepInEx\` folder
- `BepInEx\plugins\` folder

If BepInEx is missing, download and extract [BepInEx 5.4.23.5](https://github.com/BepInEx/BepInEx/releases/tag/v5.4.23) to the game directory.

### Step 2: Install DINOForge Runtime

Using PowerShell (run as Administrator):

```powershell
# Set variables
$GamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$InstallerUrl = "https://github.com/KooshaPari/Dino/releases/download/v0.14.0/DINOForge-Installer-windows.zip"
$TempDir = "$env:TEMP\DINOForge-Install"

# Create temp directory
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

# Download installer
Invoke-WebRequest -Uri $InstallerUrl -OutFile "$TempDir\installer.zip"

# Extract and run
Expand-Archive -Path "$TempDir\installer.zip" -DestinationPath $TempDir
& "$TempDir\DINOForge-Install.exe" -GamePath $GamePath

# Cleanup
Remove-Item -Path $TempDir -Recurse -Force
```

Or use the interactive installer:

1. Download the latest installer from [Releases](https://github.com/KooshaPari/Dino/releases)
2. Run `DINOForge-Installer.exe`
3. Follow the on-screen prompts
4. Select your game installation directory
5. Click "Install"

### Step 3: Configure Game Path (Optional)

If the installer didn't auto-detect your game path, manually configure it:

```powershell
# Edit the DINOForge config file
$ConfigPath = "$env:LOCALAPPDATA\DINOForge\config.json"

# Update with your game path
$config = @{
    game_install_path = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
    mods_directory = "mods"
    enable_debug_mode = $false
    mcp_server_port = 8765
}

$config | ConvertTo-Json | Set-Content -Path $ConfigPath
```

### Step 4: Deploy Example Packs (Optional)

To deploy example content packs:

```powershell
cd "C:\Users\<YourUsername>\Dino"

# Deploy example packs
dotnet run --project src/Tools/PackCompiler -- build packs/example-balance
dotnet run --project src/Tools/PackCompiler -- build packs/warfare-modern

# Copy to game directory
Copy-Item -Path "packs/example-balance/dist/*" -Destination "$GamePath/BepInEx/dinoforge_packs/" -Recurse
Copy-Item -Path "packs/warfare-modern/dist/*" -Destination "$GamePath/BepInEx/dinoforge_packs/" -Recurse
```

## Verification Checklist

After installation, verify everything is working:

### 1. Check DINOForge Plugin Is Loaded

Open PowerShell:

```powershell
$LogPath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log"
Get-Content -Path $LogPath -Tail 50 | Select-String "DINOForge"
```

You should see lines like:
```
[Info   : BepInEx] Loading [DINOForge.Runtime v0.14.0]
[Info   : BepInEx] Loaded plugin DINOForge.Runtime
```

### 2. Verify .NET Runtime

```powershell
dotnet --version
# Should output: 11.0.100-preview.2.26159.112 (or newer)
```

### 3. Check BepInEx Plugins Directory

```powershell
Get-ChildItem "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\" -Filter "*.dll"
```

Should list:
- `DINOForge.Runtime.dll`
- Other BepInEx plugins

### 4. Test MCP Server

The MCP server should be running on port 8765:

```powershell
# Launch the game first, then test connectivity
$TestRequest = @{
    jsonrpc = "2.0"
    id = 1
    method = "game_status"
    params = @{}
}

$Response = Invoke-WebRequest `
    -Uri "http://127.0.0.1:8765/rpc" `
    -Method POST `
    -Body ($TestRequest | ConvertTo-Json) `
    -ContentType "application/json" `
    -ErrorAction SilentlyContinue

if ($Response.StatusCode -eq 200) {
    Write-Host "MCP Server is responding!" -ForegroundColor Green
    $Response.Content | ConvertFrom-Json
} else {
    Write-Host "MCP Server not responding. Check game logs." -ForegroundColor Red
}
```

## Configuration Options

DINOForge settings are stored in: `%LOCALAPPDATA%\DINOForge\config.json`

### Common Configuration

```json
{
  "game_install_path": "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option",
  "mods_directory": "BepInEx/dinoforge_packs",
  "enable_debug_mode": false,
  "enable_telemetry": true,
  "mcp_server_port": 8765,
  "mcp_server_host": "127.0.0.1",
  "hot_reload_enabled": true,
  "log_level": "Info",
  "ui_overlay_enabled": true,
  "overlay_key_toggle": "F10"
}
```

## Troubleshooting

### Error: ".NET 11 Runtime Not Found"

**Symptom**: Game crashes with "System.Runtime.InteropServices error"

**Solution**:
1. Download [.NET 11 preview](https://dotnet.microsoft.com/download/dotnet/11.0)
2. Run the installer
3. Restart your computer
4. Verify: `dotnet --version` in PowerShell

### Error: "DLL Not Found" or "Module Load Failed"

**Symptom**: BepInEx log shows `DINOForge.Runtime.dll could not be loaded`

**Solution**:
1. Check Visual C++ Redistributables: [Download 2015-2022](https://support.microsoft.com/en-us/help/2977003)
2. Verify BepInEx is at version 5.4.23.5+
3. Ensure DLL is in: `BepInEx/plugins/DINOForge.Runtime.dll`
4. Check Windows Defender isn't quarantining the DLL:
   ```powershell
   # Whitelist DINOForge directory
   Add-MpPreference -ExclusionPath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx"
   ```

### Error: "MCP Server Connection Refused"

**Symptom**: `127.0.0.1:8765 refused connection`

**Solution**:
1. Verify the game is running
2. Check Windows Firewall isn't blocking port 8765:
   ```powershell
   # Allow port through firewall
   New-NetFirewallRule -DisplayName "DINOForge MCP" `
       -Direction Inbound -LocalPort 8765 -Protocol TCP -Action Allow
   ```
3. Check for port conflicts: `netstat -ano | Select-String 8765`
4. Restart the game and wait 5 seconds for MCP to initialize

### Error: "Pack Failed to Load"

**Symptom**: Game log shows `Failed to load pack 'my-pack'`

**Solution**:
1. Validate pack manifest: `dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack`
2. Check for dependency conflicts
3. Verify pack is in correct directory: `BepInEx/dinoforge_packs/`
4. Check pack.yaml syntax (YAML format errors)

### Game Crashes on Startup

**Symptom**: "Fatal error" dialog immediately after launch

**Solution**:
1. Disable all DINOForge packs temporarily:
   ```powershell
   Remove-Item "G:\...\BepInEx\dinoforge_packs\*" -Recurse
   ```
2. Launch game to verify it starts
3. Re-add packs one at a time to identify problematic pack
4. Check pack dependencies and compatibility

## Windows-Specific Notes

- **User Account Control (UAC)**: The installer requires administrator privileges. If you see UAC prompts, click "Yes" to allow installation.
- **Windows Defender**: Sometimes flags game modifications. You may need to add exceptions for the game directory.
- **Antivirus**: Some antivirus software blocks file modifications in `Program Files`. Disable scans temporarily during deployment, or use alternate Steam library paths.
- **Multiple Game Instances**: To run two game instances simultaneously (for testing), set `GameInstallPath` to different directories. See [Game Launch Protocol](https://github.com/KooshaPari/Dino/blob/main/CLAUDE.md#game-launch-protocol-via-subagent).

## Next Steps

- [Explore example packs](/packs)
- [Create your first mod pack](/guide/creating-packs)
- [Review API examples](/guide/api-reference-interactive)
- [Linux deployment](/deploy/linux-deployment)
- [macOS deployment](/deploy/macos-deployment)
- [Troubleshooting guide](/deploy/troubleshooting)
