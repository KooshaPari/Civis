# Linux Deployment Guide

This guide covers installation and configuration of DINOForge on Linux (Ubuntu 20.04+, Fedora 36+, Arch, etc.).

## System Requirements

- **OS**: Ubuntu 20.04+, Fedora 36+, Arch, Debian 11+, or compatible
- **.NET Runtime**: .NET 11 (preview) — [.NET Downloads](https://dotnet.microsoft.com/download/dotnet/11.0)
- **Game Runtime**: Wine 7.0+ or Proton GE (for Steam Proton)
- **Build Essentials**: gcc, g++, make (for native compilation)
- **Game**: Diplomacy is Not an Option (Steam + Proton)
- **Disk Space**: 2 GB (runtime + plugins + packs)
- **RAM**: 8 GB minimum (12 GB recommended)

### Checking .NET Installation

Open a terminal and verify:

```bash
dotnet --info
```

Should show `.NET 11.0.100-preview` or newer. If not installed, proceed to Step 1.

## Installation Steps

### Step 1: Install .NET 11 Runtime

**Ubuntu/Debian:**

```bash
# Add Microsoft package repository
wget https://packages.microsoft.com/config/ubuntu/22.04/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
sudo dpkg -i packages-microsoft-prod.deb
rm packages-microsoft-prod.deb

# Update package index
sudo apt-get update

# Install .NET 11 preview SDK
sudo apt-get install -y dotnet-sdk-11.0

# Verify installation
dotnet --version
```

**Fedora:**

```bash
# Install .NET 11 preview SDK
sudo dnf install dotnet-sdk-11.0

# Verify installation
dotnet --version
```

**Arch:**

```bash
# Use AUR or manual build
yay -S dotnet-sdk-11-bin

# Verify installation
dotnet --version
```

### Step 2: Install Wine/Proton (for Game Execution)

The game runs via Wine or Proton. Choose one approach:

#### Option A: Use Steam Proton (Recommended)

If you have Steam with Proton installed:

```bash
# Steam automatically provides Wine/Proton
# Your game will run via Proton automatically when launched via Steam
# No additional setup needed!
```

#### Option B: Install Standalone Wine

```bash
# Ubuntu/Debian
sudo apt-get install -y wine wine32 wine64

# Fedora
sudo dnf install wine

# Arch
sudo pacman -S wine

# Verify installation
wine --version
```

Requires Wine 7.0 or newer. For newer versions, use WineHQ staging builds.

### Step 3: Verify Game Installation

Locate your DINO installation directory:

```bash
# If using Steam with Proton (Linux native):
ls -la ~/.steam/steamapps/common/Diplomacy\ is\ Not\ an\ Option/

# Or find it:
find ~/.steam -name "Diplomacy is Not an Option" -type d
find ~/.local/share/Steam -name "Diplomacy is Not an Option" -type d

# If using Windows Steam via Wine:
~/.wine/drive_c/Program\ Files\ \(x86\)/Steam/steamapps/common/Diplomacy\ is\ Not\ an\ Option/
```

Verify these files exist:
- `Diplomacy is Not an Option.exe`
- `BepInEx/` directory
- `BepInEx/plugins/` directory

### Step 4: Install DINOForge Runtime

Clone the repository (or download release):

```bash
# Clone the DINOForge repository
git clone https://github.com/KooshaPari/Dino.git ~/Dino
cd ~/Dino

# Or download the latest release
wget https://github.com/KooshaPari/Dino/releases/download/v0.14.0/DINOForge-v0.14.0.tar.gz
tar -xzf DINOForge-v0.14.0.tar.gz
cd Dino
```

Run the Linux installer script:

```bash
# Make script executable
chmod +x scripts/install/installer.sh

# Run installer
bash scripts/install/installer.sh

# Follow the prompts:
# 1. Specify game installation path
# 2. Confirm BepInEx installation
# 3. Deploy example packs (optional)
```

### Step 5: Configure Game Path

Edit the DINOForge config:

```bash
# Create config directory
mkdir -p ~/.config/DINOForge

# Create config file
cat > ~/.config/DINOForge/config.json << 'EOF'
{
  "game_install_path": "/home/username/.steam/steamapps/common/Diplomacy is Not an Option",
  "mods_directory": "BepInEx/dinoforge_packs",
  "enable_debug_mode": false,
  "enable_telemetry": true,
  "mcp_server_port": 8765,
  "mcp_server_host": "127.0.0.1",
  "hot_reload_enabled": true,
  "log_level": "Info"
}
EOF

# Update the path with your actual game directory
```

### Step 6: Deploy Example Packs

```bash
cd ~/Dino

# Build example packs
dotnet run --project src/Tools/PackCompiler -- build packs/example-balance
dotnet run --project src/Tools/PackCompiler -- build packs/warfare-modern

# Get your actual game path
GAME_PATH=$(grep "game_install_path" ~/.config/DINOForge/config.json | cut -d'"' -f4)

# Copy packs to game directory
mkdir -p "$GAME_PATH/BepInEx/dinoforge_packs"
cp -r packs/example-balance/dist/* "$GAME_PATH/BepInEx/dinoforge_packs/"
cp -r packs/warfare-modern/dist/* "$GAME_PATH/BepInEx/dinoforge_packs/"
```

## Launching the Game

### Via Steam (Recommended)

If using Steam with Proton:

```bash
# Launch from Steam normally
# Proton automatically handles Windows compatibility
```

### Via Command Line (Wine/Proton)

```bash
# Using Proton (recommended for Linux)
GAME_PATH="/path/to/Diplomacy is Not an Option"
proton run "$GAME_PATH/Diplomacy is Not an Option.exe"

# Or using standalone Wine
wine "$GAME_PATH/Diplomacy is Not an Option.exe"
```

### Via Wine Configuration

Some games benefit from specific Wine prefix settings:

```bash
# Create a dedicated Wine prefix for the game
export WINEPREFIX=~/.wine-dino
wine wineboot --init

# Set Windows version
WINEARCH=win64 wine winecfg

# In the window that appears:
# Go to "Windows Version" tab
# Select "Windows 10"
# Click "Apply" then "OK"
```

## Verification Checklist

### 1. Check .NET Installation

```bash
dotnet --version
# Expected: 11.0.100-preview.2.26159.112 or newer
```

### 2. Verify Game Launches

```bash
# Navigate to game directory
cd "/path/to/Diplomacy is Not an Option"

# Launch the game
wine ./Diplomacy\ is\ Not\ an\ Option.exe &

# Wait 5 seconds, then check if process is running
sleep 5
pgrep -f "Diplomacy is Not an Option"
```

### 3. Check BepInEx Logs

After game startup, check the BepInEx log:

```bash
tail -50 "/path/to/game/BepInEx/LogOutput.log" | grep -i dinoforge
```

Should show:
```
[Info   : BepInEx] Loading [DINOForge.Runtime v0.14.0]
[Info   : BepInEx] Loaded plugin DINOForge.Runtime
```

### 4. Verify MCP Server Port

After launching game:

```bash
# Check if port 8765 is open
lsof -i :8765

# Test connectivity
curl -X POST http://127.0.0.1:8765/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"game_status","params":{}}'
```

## Proton Configuration

If using Steam with Proton GE or official Proton:

```bash
# Get Proton version
proton --version

# Set custom Proton version for a game
# In Steam: right-click game → Properties → Compatibility → Force Compatibility Tool → Select Proton version

# Or via command line with custom Wine settings
PROTON_NO_ESYNC=1 PROTON_USE_WINED3D=1 proton run game.exe
```

## Common Proton/Wine Flags

If you encounter performance or compatibility issues:

```bash
# Disable esync (for some games)
export PROTON_NO_ESYNC=1

# Use wined3d instead of DXVK (older GPU support)
export PROTON_USE_WINED3D=1

# Enable debug logging
export WINEDEBUG=+all

# Run with these flags
wine "./Diplomacy is Not an Option.exe"
```

## Troubleshooting

### ".NET 11 Runtime Not Found"

**Solution**:
1. Verify installation: `dotnet --version`
2. If not found, reinstall following Step 1 for your distro
3. Ensure `/usr/bin/dotnet` is in your PATH: `which dotnet`

### "Wine: Command Not Found"

**Solution**:
```bash
# Reinstall Wine for your distro
sudo apt-get install -y wine wine32 wine64

# Or use Proton instead (via Steam)
```

### Game Fails to Launch via Wine

**Solution**:
1. Verify Wine version: `wine --version` (should be 7.0+)
2. Check Wine prefix is initialized: `wine winecfg`
3. Try updating Wine: `sudo apt upgrade wine`
4. If stuck, use Steam + Proton instead

### MCP Server Port Blocked or Unavailable

**Solution**:
```bash
# Check what's using port 8765
sudo lsof -i :8765

# If blocked, try alternate port in config
# Or kill the process using it (if safe)
sudo kill -9 <PID>
```

### Pack Fails to Load

**Solution**:
1. Verify pack path has correct permissions:
   ```bash
   chmod -R 755 "$GAME_PATH/BepInEx/dinoforge_packs"
   ```
2. Check for symlink issues: `ls -la "$GAME_PATH/BepInEx"`
3. Validate pack syntax:
   ```bash
   cd ~/Dino
   dotnet run --project src/Tools/PackCompiler -- validate packs/your-pack
   ```

## Linux-Specific Notes

- **File Permissions**: Ensure the game directory is readable/writable: `chmod -r u+rwX "$GAME_PATH"`
- **Symlinks**: Wine sometimes has issues with symlinks. Use direct paths or copies instead.
- **Case Sensitivity**: Linux is case-sensitive. Ensure file references match exactly (including capitalization).
- **Path Separators**: Use forward slashes `/` in configs, not backslashes `\`
- **Snap/Flatpak**: If running Steam via Snap/Flatpak, adjust paths accordingly (e.g., `~/snap/steam/common/steamapps/`)

## Next Steps

- [Explore example packs](/packs)
- [Create your first mod pack](/guide/creating-packs)
- [Review API examples](/guide/api-reference-interactive)
- [Windows deployment](/deploy/windows-deployment)
- [macOS deployment](/deploy/macos-deployment)
- [Troubleshooting guide](/deploy/troubleshooting)
