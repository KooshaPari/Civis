# macOS Deployment Guide

This guide covers installation of DINOForge on macOS (Intel and Apple Silicon).

## System Requirements

- **OS**: macOS 11 (Big Sur) or newer
- **Processor**: Intel or Apple Silicon (M1/M2/M3)
- **.NET Runtime**: .NET 11 (preview) — [.NET Downloads](https://dotnet.microsoft.com/download/dotnet/11.0)
- **Game**: Diplomacy is Not an Option (via Steam + Proton/Parallels)
- **Build Tools**: Xcode Command Line Tools (optional, for development)
- **Disk Space**: 2 GB (runtime + plugins)
- **RAM**: 8 GB minimum (12 GB recommended)

### Checking .NET Installation

Open Terminal and verify:

```bash
dotnet --info
```

Should show `.NET 11.0.100-preview` or newer.

## Installation Steps

### Step 1: Install .NET 11 Runtime

#### For Intel Macs

```bash
# Download .NET 11 SDK for macOS x64
wget https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/releases-index.json -O - | \
  grep "NET 11" | head -1

# Or download manually from:
# https://dotnet.microsoft.com/download/dotnet/11.0

# After download, run the installer
# Then verify
dotnet --version
```

#### For Apple Silicon (M1/M2/M3)

```bash
# Apple Silicon support for .NET 11 is in preview
# Download the arm64 version from:
# https://dotnet.microsoft.com/download/dotnet/11.0

# Look for "macOS Arm64"

# After installation, verify native execution:
dotnet --info

# Check architecture (should show arm64)
uname -m
# Expected: arm64
```

### Step 2: Install Game and Proton

macOS doesn't natively support Windows games. You have three options:

#### Option A: Steam + Proton (Recommended)

Most straightforward method using Steam's built-in compatibility layer:

```bash
# 1. Install Steam from: https://steampowered.com
# 2. In Steam preferences: Settings → Compatibility
# 3. Enable "Compatibility tool" and select latest Proton version
# 4. Install "Diplomacy is Not an Option" from Steam
# 5. Launch the game — Proton handles Windows compatibility automatically
```

#### Option B: Parallels Desktop (Full Windows VM)

For full Windows environment (more reliable but resource-intensive):

```bash
# 1. Purchase and install Parallels Desktop 18+
# 2. Create Windows 11 ARM64 VM (on Apple Silicon) or Windows 10 (on Intel)
# 3. Install .NET 11 in the VM (see Windows Deployment guide)
# 4. Install DINOForge in the VM as normal
# 5. Access from macOS via Parallels shared folders
```

#### Option C: UTM (Free, VM for Apple Silicon)

Open-source VM alternative:

```bash
# Install UTM from: https://mac.getutm.app/
# Create Windows 11 ARM64 VM with adequate resources
# Install Windows, .NET 11, and DINOForge as per Windows guide
```

### Step 3: Verify Game Installation

Check your game directory:

```bash
# If using Steam + Proton
ls -la ~/Library/Application\ Support/Steam/steamapps/common/Diplomacy\ is\ Not\ an\ Option/

# If using Parallels Desktop
# Game is inside the Windows VM, usually at:
# C:\Program Files\Steam\steamapps\common\Diplomacy is Not an Option
```

Ensure these exist:
- `Diplomacy is Not an Option.exe`
- `BepInEx/` directory

### Step 4: Install DINOForge

Clone or download the repository:

```bash
# Clone the repository
git clone https://github.com/KooshaPari/Dino.git ~/Dino
cd ~/Dino

# Or download release (Intel/Apple Silicon universal)
wget https://github.com/KooshaPari/Dino/releases/download/v0.14.0/DINOForge-v0.14.0.tar.gz
tar -xzf DINOForge-v0.14.0.tar.gz
cd Dino
```

Run the installer:

```bash
# Make script executable
chmod +x scripts/install/installer.sh

# Run macOS-specific installer
bash scripts/install/installer.sh --target macos

# Follow prompts:
# 1. Specify game path
# 2. Confirm installation
# 3. Deploy example packs (optional)
```

### Step 5: Configure DINOForge

```bash
# Create config directory
mkdir -p ~/.config/DINOForge

# Create config file
cat > ~/.config/DINOForge/config.json << 'EOF'
{
  "game_install_path": "/Users/username/Library/Application Support/Steam/steamapps/common/Diplomacy is Not an Option",
  "mods_directory": "BepInEx/dinoforge_packs",
  "enable_debug_mode": false,
  "mcp_server_port": 8765,
  "mcp_server_host": "127.0.0.1",
  "hot_reload_enabled": true
}
EOF

# Update the path with your actual game location
```

### Step 6: Deploy Packs

```bash
cd ~/Dino

# Build example packs
dotnet run --project src/Tools/PackCompiler -- build packs/example-balance
dotnet run --project src/Tools/PackCompiler -- build packs/warfare-modern

# Copy to game directory
GAME_PATH="/Users/$(whoami)/Library/Application Support/Steam/steamapps/common/Diplomacy is Not an Option"
mkdir -p "$GAME_PATH/BepInEx/dinoforge_packs"
cp -r packs/example-balance/dist/* "$GAME_PATH/BepInEx/dinoforge_packs/"
cp -r packs/warfare-modern/dist/* "$GAME_PATH/BepInEx/dinoforge_packs/"
```

## Launching the Game

### Via Steam (macOS native)

```bash
# Steam app is at: /Applications/Steam.app
# Simply launch via Finder or:
open /Applications/Steam.app

# Then launch "Diplomacy is Not an Option" from Steam library
```

### Via Command Line

If using Steam + Proton:

```bash
# Steam ID for DINO: 1370840
# Launch via Steam protocol
open "steam://run/1370840"

# Or use command line with Proton
GAME_PATH="$HOME/Library/Application Support/Steam/steamapps/common/Diplomacy is Not an Option"
~/.steam/steamapps/Proton*/protonrun "$GAME_PATH/Diplomacy is Not an Option.exe"
```

## Verification Checklist

### 1. .NET Installation

```bash
dotnet --version
# Expected: 11.0.100-preview.2.26159.112 or newer

# Verify architecture
dotnet --info | grep "RID:"
# Intel: osx-x64
# Apple Silicon: osx-arm64
```

### 2. Game Launches

```bash
# Wait for Steam to launch game, then check:
ps aux | grep "Diplomacy is Not an Option"
```

### 3. Check BepInEx Logs

```bash
GAME_PATH="$HOME/Library/Application Support/Steam/steamapps/common/Diplomacy is Not an Option"
tail -50 "$GAME_PATH/BepInEx/LogOutput.log" | grep -i dinoforge
```

### 4. MCP Server Connectivity

```bash
# After launching game, test MCP server
curl -X POST http://127.0.0.1:8765/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"game_status","params":{}}'

# Should return JSON response
```

## Troubleshooting

### ".NET 11 Runtime Not Found"

**Solution**:
1. Download from https://dotnet.microsoft.com/download/dotnet/11.0
2. Ensure you download the correct version:
   - **Intel Mac**: `macOS x64`
   - **Apple Silicon**: `macOS Arm64`
3. After installation: `dotnet --version`

### "Cannot Find Game Installation"

**Solution**:
```bash
# Find game path manually
find ~/ -name "Diplomacy is Not an Option.exe" -type f

# Steam game locations:
ls ~/Library/Application\ Support/Steam/steamapps/common/ | grep -i diplomacy
```

### "BepInEx Plugins Not Loading"

**Solution**:
1. Verify BepInEx directory exists and has correct permissions:
   ```bash
   ls -la "$GAME_PATH/BepInEx/"
   chmod -R u+rwX "$GAME_PATH/BepInEx"
   ```
2. Check if running via Proton (compatibility may vary)
3. Inspect BepInEx log for errors

### "Cannot Read/Write Game Files" (Permission Denied)

**Solution**:
```bash
# Fix permissions on game directory
GAME_PATH="$HOME/Library/Application Support/Steam/steamapps/common/Diplomacy is Not an Option"
chmod -R u+rwX "$GAME_PATH"

# Or if in shared location
sudo chmod -R u+rwX "$GAME_PATH"
```

### "MCP Server Port 8765 in Use"

**Solution**:
```bash
# Check what's using the port
lsof -i :8765

# Kill the process (if safe)
kill -9 <PID>

# Or configure alternate port in config.json
```

### Game Performance Issues

**Solution** (for Intel Macs):
```bash
# Disable esync if experiencing problems
export PROTON_NO_ESYNC=1

# Or use wined3d renderer
export PROTON_USE_WINED3D=1

# Relaunch game after setting env var
```

For Apple Silicon: Performance may be slower due to ARM64 emulation. Parallels Desktop provides better performance on M1/M2.

## macOS-Specific Notes

- **Gatekeeper**: macOS may quarantine downloaded files. If you see "can't be opened" error:
  ```bash
  # Remove quarantine attribute
  xattr -d com.apple.quarantine ~/Dino
  ```

- **Intel vs Apple Silicon**: .NET 11 supports both architectures natively. ARM64 version is preferred for Apple Silicon.

- **Parallels Desktop**: If using Parallels Desktop, follow the Windows deployment guide inside the VM. Game will run faster due to native Windows environment.

- **File Permissions**: Steam and Proton may have permission issues. Regularly check: `chmod -R u+rwX ~/Library/Application\ Support/Steam`

- **Disk Space**: Ensure sufficient free space for Steam, Proton, .NET runtime, and game files (minimum 10 GB free recommended).

## Known Limitations

- **BepInEx Support**: BepInEx ARM64 native version is in development. Current M1/M2 support relies on x64 emulation or Parallels Desktop.
- **Game Performance**: Windows games on macOS via Proton are slower than native Windows. Intel Macs handle better than Apple Silicon currently.
- **MCP Server**: The Python MCP server requires Python 3.9+ on macOS (usually pre-installed).

## Recommended Setup for macOS

1. **For best performance**: Use Parallels Desktop with Windows VM
2. **For convenience**: Use Steam + Proton (slower but simpler)
3. **For free option**: Use UTM + Windows 11 ARM64 VM (requires more setup)

## Next Steps

- [Explore example packs](/packs)
- [Create your first mod pack](/guide/creating-packs)
- [Review API examples](/guide/api-reference-interactive)
- [Windows deployment](/deploy/windows-deployment)
- [Linux deployment](/deploy/linux-deployment)
- [Troubleshooting guide](/deploy/troubleshooting)
