# Troubleshooting Guide

Comprehensive solutions for common DINOForge installation and runtime issues across all platforms.

## Installation Issues

### ".NET 11 Runtime Not Found"

**Symptom**: Game crashes with error message containing "System.Runtime" or "NET 11"

**Root Cause**: .NET 11 preview SDK/runtime not installed or not in PATH

**Solutions**:

1. **Install .NET 11 Preview**:
   ```bash
   # Windows (PowerShell, Administrator)
   # Download from https://dotnet.microsoft.com/download/dotnet/11.0
   # Or use Chocolatey:
   choco install dotnet-11.0-preview

   # Linux (Ubuntu/Debian)
   sudo apt-get install dotnet-sdk-11.0

   # Linux (Fedora)
   sudo dnf install dotnet-sdk-11.0

   # macOS
   # Download installer from https://dotnet.microsoft.com/download/dotnet/11.0
   ```

2. **Verify Installation**:
   ```bash
   dotnet --version
   # Should output: 11.0.100-preview.2.26159.112 or newer
   ```

3. **Update PATH (if needed)**:
   - Windows: `C:\Program Files\dotnet\` should be in PATH
   - Linux/macOS: `/usr/local/share/dotnet` or `/usr/bin/dotnet`

4. **Check Multiple .NET Versions**:
   ```bash
   dotnet --info
   # Look for .NET 11 in the list
   ```

### "BepInEx Plugins Not Loading"

**Symptom**: BepInEx log shows "DINOForge.Runtime.dll could not be loaded" or "Failed to load plugin"

**Root Cause**: 
- BepInEx version mismatch
- Missing Visual C++ Redistributables
- DLL corruption or inaccessible file
- BepInEx plugins directory issue

**Solutions**:

1. **Verify BepInEx Version**:
   ```bash
   # Check BepInEx.cfg in game directory
   grep "BepInExVersion" BepInEx/BepInEx.cfg
   # Should be 5.4.23.5 or newer
   ```

2. **Install Visual C++ Redistributables**:
   - Windows: [Download Microsoft Visual C++ 2015-2022](https://support.microsoft.com/en-us/help/2977003)
   - Run installer, select "Repair"

3. **Check File Permissions** (Linux/macOS):
   ```bash
   chmod -R u+rwX BepInEx/
   ls -la BepInEx/plugins/DINOForge.Runtime.dll
   ```

4. **Verify Plugin Directory Structure**:
   ```bash
   # Windows
   dir BepInEx\plugins\

   # Linux/macOS
   ls -la BepInEx/plugins/

   # Should contain: DINOForge.Runtime.dll
   ```

5. **Clear BepInEx Cache** (Nuclear option):
   ```bash
   # Backup first
   cp -r BepInEx BepInEx.backup

   # Delete cache
   rm -rf BepInEx/cache

   # Restart game to rebuild cache
   ```

### "Installation Path Not Found" or "Game Directory Invalid"

**Symptom**: Installer can't find game directory or shows "path does not exist"

**Root Cause**: Steam installation path not detected, or using alternate game path

**Solutions**:

1. **Manually Specify Path**:
   ```bash
   # Windows (PowerShell)
   $InstallPath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
   # Verify it exists
   Test-Path $InstallPath
   ```

2. **Find Correct Path**:
   - Windows: Open Steam → Library → Right-click DINO → Manage → Browse local files
   - Linux: `find ~/.steam -name "Diplomacy is Not an Option" -type d`
   - macOS: `find ~/Library -name "Diplomacy is Not an Option" -type d`

3. **Check Spaces in Path** (Windows):
   - If path has spaces (e.g., "Program Files"), ensure it's quoted in config
   - Use `%ProgramFiles%` environment variable when possible

## Runtime Issues

### "MCP Server Connection Refused"

**Symptom**: "127.0.0.1:8765 connection refused" or "Cannot connect to MCP server"

**Root Cause**: 
- Game not running
- MCP server didn't start
- Port 8765 in use or blocked
- Firewall blocking connection

**Solutions**:

1. **Verify Game Is Running**:
   ```bash
   # Windows (PowerShell)
   Get-Process | Select-String "Diplomacy"

   # Linux/macOS
   ps aux | grep -i diplomacy
   ```

2. **Check Port Availability**:
   ```bash
   # Windows (PowerShell)
   netstat -ano | Select-String 8765

   # Linux
   lsof -i :8765

   # If something is using it, kill it:
   # Windows: Get-Process -Id <PID> | Stop-Process
   # Linux: sudo kill -9 <PID>
   ```

3. **Configure Alternate Port**:
   ```json
   // config.json
   {
     "mcp_server_port": 8766,
     "mcp_server_host": "127.0.0.1"
   }
   ```

4. **Open Firewall** (Windows):
   ```powershell
   # Allow port through firewall
   New-NetFirewallRule -DisplayName "DINOForge MCP" `
       -Direction Inbound -LocalPort 8765 -Protocol TCP -Action Allow

   # Verify rule was added
   Get-NetFirewallRule -DisplayName "DINOForge MCP"
   ```

5. **Check BepInEx Log**:
   ```bash
   # Windows
   type "BepInEx\dinoforge_debug.log" | tail -50

   # Linux/macOS
   tail -50 BepInEx/dinoforge_debug.log | grep -i "mcp\|server"
   ```

### "Game Crashes on Mod Load"

**Symptom**: Game crashes immediately after mod loads, or during gameplay

**Root Cause**: 
- Incompatible pack dependencies
- Missing framework version
- Schema validation failure
- Mod conflict with another mod

**Solutions**:

1. **Check Pack Dependencies**:
   ```bash
   # Validate pack manifest
   dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack

   # Check pack.yaml for correct versions
   grep "framework_version\|depends_on" packs/my-pack/pack.yaml
   ```

2. **Verify Framework Version**:
   ```bash
   # In pack.yaml, check:
   framework_version: ">=0.14.0 <1.0.0"  # Should match installed DINOForge version
   ```

3. **Disable Conflicting Mods**:
   ```bash
   # Temporarily remove all packs except one
   mkdir BepInEx/dinoforge_packs_backup
   mv BepInEx/dinoforge_packs/* BepInEx/dinoforge_packs_backup/

   # Add packs back one at a time to identify culprit
   ```

4. **Check Game Log for Details**:
   ```bash
   # Windows
   type "BepInEx\LogOutput.log" | tail -100

   # Linux/macOS
   tail -100 BepInEx/LogOutput.log | grep -i "error\|exception"
   ```

5. **Verify Pack Integrity**:
   ```bash
   # Check for missing files
   ls -la packs/my-pack/
   # Should contain: pack.yaml, definitions/, assets/ (if applicable)
   ```

### "DLL Lock" or "File Already in Use"

**Symptom**: "The process cannot access the file because it is being used by another process" when deploying updates

**Root Cause**: Game or other process has DLL locked in memory

**Solutions**:

1. **Kill Game Process**:
   ```bash
   # Windows (PowerShell)
   Stop-Process -Name "Diplomacy is Not an Option" -Force

   # Linux
   pkill -9 "Diplomacy is Not an Option"

   # macOS
   pkill -9 "Diplomacy"
   ```

2. **Use Deployment with Game Closed**:
   ```bash
   # Close game completely before:
   dotnet build -p:DeployToGame=true
   ```

3. **Unload Assembly** (Advanced):
   ```csharp
   // In MCP server: force unload before redeployment
   if (AppDomain.CurrentDomain != null)
   {
       GC.Collect();
       GC.WaitForPendingFinalizers();
   }
   ```

4. **Restart Computer** (Last resort):
   - Some file locks persist despite process termination
   - Reboot ensures all handles are released

### "Pack Failed to Load" or "Missing Definition"

**Symptom**: Game log shows "Pack 'X' failed to load" or "Definition 'Y' not found"

**Root Cause**: 
- Invalid YAML syntax in pack.yaml
- Missing schema files
- Incorrect definition references
- Pack not in correct directory

**Solutions**:

1. **Validate Pack.yaml Syntax**:
   ```bash
   # Use online YAML validator or:
   dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack

   # Check for common issues:
   # - Incorrect indentation (YAML uses spaces, not tabs)
   # - Missing colons after keys
   # - Unclosed strings
   ```

2. **Verify Definitions Exist**:
   ```bash
   # Check units defined in pack.yaml exist in definitions
   grep -n "id:" packs/my-pack/definitions/units.yaml

   # Make sure all IDs match between manifest and definitions
   ```

3. **Check Pack Directory Structure**:
   ```bash
   # Expected structure:
   packs/my-pack/
     pack.yaml
     definitions/
       units.yaml
       buildings.yaml
       factions.yaml
     assets/  (if visual assets)
   ```

4. **Check File Encoding** (Windows):
   - Ensure YAML files are UTF-8, not UTF-16 or with BOM
   - Use Visual Studio Code: right-click → "Reopen with Encoding" → UTF-8

5. **Check Pack Load Order**:
   ```bash
   # Dependencies are loaded in order
   # If Pack B depends on Pack A, ensure Pack A exists and loads first
   ```

## Platform-Specific Issues

### Windows-Specific

#### "Access Denied" When Writing to Game Directory

**Solution**:
```powershell
# Run installer as Administrator
# Or modify folder permissions:
$GamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"

# Grant permissions
icacls "$GamePath" /grant:r "$env:USERNAME`:F"
```

#### Long Path Issues (MAX_PATH)

**Solution**:
```powershell
# Enable long paths in Windows 10/11
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
    -Name "LongPathsEnabled" -Value 1 -Force
```

### Linux-Specific

#### "Permission Denied" on Game Directory

**Solution**:
```bash
# Fix permissions
chmod -R u+rwX ~/.steam/steamapps/common/Diplomacy\ is\ Not\ an\ Option/

# Or as root if needed
sudo chown -R $USER ~/.steam/steamapps/common/Diplomacy\ is\ Not\ an\ Option/
```

#### File Case Sensitivity Issues

**Solution** (Linux is case-sensitive):
```bash
# Ensure all filenames match exactly, including capitalization
# Wrong: pack.YAML
# Correct: pack.yaml

# Find case issues:
find . -name "*Pack.yaml" -o -name "*pack.YAML"
```

#### Wine/Proton Issues

**Solution**:
```bash
# Update Wine/Proton to latest version
proton-update

# Or manually upgrade Wine
sudo apt-get install --only-upgrade wine

# Set environment variables if issues persist:
export PROTON_NO_ESYNC=1
export PROTON_USE_WINED3D=1
```

### macOS-Specific

#### "Cannot Open" due to Gatekeeper (Downloaded files)

**Solution**:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ~/Dino
xattr -d com.apple.quarantine ~/.config/DINOForge
```

#### Permissions Issues in `/Library`

**Solution**:
```bash
# macOS system folders need sudo
sudo chmod -R u+rwX ~/Library/Application\ Support/Steam/steamapps/

# Or use Parallels Desktop for better compatibility
```

#### Apple Silicon (M1/M2) Performance

**Solution**:
- Use Parallels Desktop with Windows VM for best performance
- Or wait for native ARM64 BepInEx support
- Current ARM64 .NET 11 works but is slower via emulation

## Diagnostic Commands

### Comprehensive Health Check

**Windows (PowerShell)**:
```powershell
# Check everything
Write-Host "=== DINOForge Health Check ===" -ForegroundColor Green
Write-Host "1. .NET Version:"
dotnet --version

Write-Host "2. Game Process:"
Get-Process | Select-String "Diplomacy" | Format-Table

Write-Host "3. BepInEx Files:"
Get-ChildItem "BepInEx\plugins\" -Filter "DINOForge*"

Write-Host "4. MCP Port:"
netstat -ano | Select-String 8765

Write-Host "5. Recent Errors (last 10):"
Get-Content "BepInEx\LogOutput.log" -Tail 10
```

**Linux/macOS (Bash)**:
```bash
#!/bin/bash
echo "=== DINOForge Health Check ==="
echo "1. .NET Version:"
dotnet --version

echo "2. Game Process:"
ps aux | grep -i diplomacy

echo "3. BepInEx Files:"
ls -la BepInEx/plugins/DINOForge*

echo "4. MCP Port:"
lsof -i :8765

echo "5. Recent Errors:"
tail -10 BepInEx/LogOutput.log | grep -i error
```

## Getting Help

If troubleshooting steps don't resolve your issue:

1. **Check GitHub Issues**: [DINOForge Issues](https://github.com/KooshaPari/Dino/issues)
2. **Review Documentation**: [DINOForge Docs](/guide/getting-started)
3. **Submit Diagnostics**: Include:
   - OS and version
   - .NET version (`dotnet --version`)
   - BepInEx log (last 50 lines)
   - DINOForge debug log (if available)
   - Exact error message and steps to reproduce

## Next Steps

- [Windows deployment](/deploy/windows-deployment)
- [Linux deployment](/deploy/linux-deployment)
- [macOS deployment](/deploy/macos-deployment)
- [Docker deployment](/deploy/docker-deployment)
