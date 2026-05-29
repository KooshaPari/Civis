# DINOForge PowerShell Module

The DINOForge PowerShell module provides native cmdlets that wrap the `dinoforge` CLI with full PowerShell semantics, making mod development feel native to Windows PowerShell.

## Features

- **Verb-Noun Naming**: All cmdlets follow PowerShell conventions (`Install-DINOForge`, `Deploy-DINOForgePack`, etc.)
- **Structured Output**: Returns `PSCustomObject` instead of raw text for programmatic consumption
- **Pipeline Support**: Chain commands together naturally
- **Automatic Game Detection**: Finds DINO installation via Steam registry
- **Error Handling**: Consistent error reporting and retry logic
- **Aliases**: Short aliases for frequent operations (`dino-status`, `dino-deploy`, etc.)

## Installation

### One-Line Installation (Recommended)

Open PowerShell and run:

```powershell
iwr https://raw.githubusercontent.com/KooshaPari/Dino/main/tools/Install-DINOForge.ps1 | iex
```

This installer will:
1. Install `DINOForge.Tools.Cli` as a global dotnet tool
2. Download and configure the PowerShell module
3. Verify/install BepInEx (if needed)
4. Deploy DINOForge runtime to your game installation

### Manual Installation

If you prefer to install manually:

```powershell
# 1. Create module directory
$ModulePath = Join-Path $env:USERPROFILE "Documents\PowerShell\Modules\DINOForge"
New-Item -ItemType Directory -Path $ModulePath -Force | Out-Null

# 2. Download module files
$repoBase = "https://raw.githubusercontent.com/KooshaPari/Dino/main/tools/PSModule"
Invoke-WebRequest -Uri "$repoBase/DINOForge.psm1" -OutFile "$ModulePath\DINOForge.psm1"
Invoke-WebRequest -Uri "$repoBase/DINOForge.psd1" -OutFile "$ModulePath\DINOForge.psd1"

# 3. Import module
Import-Module DINOForge -Force

# 4. Verify installation
Get-Command -Module DINOForge | Select-Object Name
```

### Install via PowerShell Gallery (Future)

Once published to PowerShell Gallery:

```powershell
Install-Module -Name DINOForge -Scope CurrentUser
Update-Module -Name DINOForge  # Updates to latest version
```

## Cmdlets

### Install-DINOForge
Installs DINOForge to a game installation.

```powershell
# Auto-detect game path
Install-DINOForge

# Explicit game path
Install-DINOForge -GamePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"

# Without confirmation
Install-DINOForge -Confirm:$false
```

**Parameters:**
- `-GamePath <string>`: Path to game directory (auto-detected if omitted)
- `-SkipBackup`: Skip backup creation before installation

**Output:**
```
GamePath    : G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option
Success     : True
Status      : Installed
Timestamp   : 5/28/2026 3:14:22 PM
Details     : {...}
```

### Get-DINOForgeStatus
Queries DINOForge runtime for current status.

```powershell
# Get status (auto-detect game)
Get-DINOForgeStatus

# Check specific installation
Get-DINOForgeStatus -GamePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"

# Alias
dino-status
```

**Output:**
```
GamePath    : G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option
Success     : True
Timestamp   : 5/28/2026 3:14:22 PM
Data        : {
                 LoadedPacks: [...]
                 EntityCount: 45776
                 SystemGroups: {...}
              }
```

### Deploy-DINOForgePack
Builds, validates, and deploys a pack to the game.

```powershell
# Deploy a pack (auto-detect game)
Deploy-DINOForgePack -PackName warfare-starwars

# Deploy with hot-reload (if game is running)
Deploy-DINOForgePack -PackName warfare-starwars -HotReload

# Skip validation
Deploy-DINOForgePack -PackName warfare-starwars -NoValidate

# Pipeline usage
"warfare-starwars", "economy-balanced" | Deploy-DINOForgePack

# Alias
dino-deploy -PackName warfare-starwars
```

**Parameters:**
- `-PackName <string>`: Name of pack to deploy (mandatory)
- `-GamePath <string>`: Game installation path (auto-detected if omitted)
- `-NoValidate`: Skip schema validation
- `-HotReload`: Trigger hot-reload if game is running

**Output:**
```
PackName      : warfare-starwars
GamePath      : G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option
Success       : True
Status        : Deployed
HotReloaded   : True
Timestamp     : 5/28/2026 3:14:22 PM
Details       : {...}
```

### New-DINOForgePack
Scaffolds a new mod pack with templates.

```powershell
# Create basic content pack
New-DINOForgePack -PackName my-custom-mod

# Create with specific type
New-DINOForgePack -PackName warcraft-theme -PackType total_conversion

# Custom author
New-DINOForgePack -PackName my-mod -Author "MyName" -OutputPath "D:\Mods"

# Alias
dino-new -PackName my-custom-mod
```

**Parameters:**
- `-PackName <string>`: Pack name in kebab-case (mandatory)
- `-PackType <string>`: Type - content, balance, ruleset, scenario, total_conversion, utility (default: content)
- `-Author <string>`: Author name (default: current username)
- `-OutputPath <string>`: Output directory (default: packs/)

**Output:**
```
PackName   : my-custom-mod
PackType   : content
Author     : MyName
OutputPath : packs
Success    : True
Timestamp  : 5/28/2026 3:14:22 PM
Details    : {...}
```

### Invoke-DINOForgeSmoke
Runs smoke tests and integration tests.

```powershell
# Run smoke tests (default)
Invoke-DINOForgeSmoke

# Run integration tests
Invoke-DINOForgeSmoke -Scenario integration

# Run all tests
Invoke-DINOForgeSmoke -Scenario all
```

**Parameters:**
- `-Scenario <string>`: smoke, integration, or all (default: smoke)
- `-GamePath <string>`: Game path (auto-detected if omitted)

**Output:**
```
Scenario   : smoke
Success    : True
Output     : [test output...]
Timestamp  : 5/28/2026 3:14:22 PM
```

### Update-DINOForge
Updates DINOForge to the latest version.

```powershell
# Update to latest stable
Update-DINOForge

# Include pre-release versions
Update-DINOForge -PreRelease

# Without confirmation
Update-DINOForge -Confirm:$false
```

**Output:**
```
Success   : True
Status    : Updated
Output    : Tool updated successfully
Timestamp : 5/28/2026 3:14:22 PM
```

### Get-DINOForgeMetrics
Retrieves performance metrics and diagnostics.

```powershell
# Get all metrics
Get-DINOForgeMetrics

# Get specific metric type
Get-DINOForgeMetrics -MetricType performance
Get-DINOForgeMetrics -MetricType memory
Get-DINOForgeMetrics -MetricType packs

# Alias
dino-metrics
```

**Parameters:**
- `-MetricType <string>`: all, performance, packs, entities, memory (default: all)
- `-GamePath <string>`: Game path (auto-detected if omitted)

**Output:**
```
MetricType : performance
Success    : True
GamePath   : G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option
Timestamp  : 5/28/2026 3:14:22 PM
Data       : {
               FrameTime: 16.2ms
               EntityCount: 45776
               PackLoadTime: 234ms
             }
```

### Get-DINOForgeHelp
Displays help for dinoforge CLI commands.

```powershell
# List all commands
Get-DINOForgeHelp

# Get help for specific command
Get-DINOForgeHelp -Command pack
Get-DINOForgeHelp -Command deploy
```

## Aliases

For convenience, short aliases are available:

| Alias | Cmdlet |
|-------|--------|
| `dino-status` | `Get-DINOForgeStatus` |
| `dino-deploy` | `Deploy-DINOForgePack` |
| `dino-new` | `New-DINOForgePack` |
| `dino-metrics` | `Get-DINOForgeMetrics` |

Example:
```powershell
dino-status
dino-deploy -PackName my-mod -HotReload
dino-new -PackName my-pack
```

## Examples

### Basic Workflow

```powershell
# 1. Create a new pack
New-DINOForgePack -PackName my-awesome-mod -PackType balance

# 2. Edit your pack files (in packs/my-awesome-mod/)

# 3. Deploy to game
Deploy-DINOForgePack -PackName my-awesome-mod

# 4. Check status
Get-DINOForgeStatus

# 5. View metrics
Get-DINOForgeMetrics -MetricType packs
```

### Advanced Scenarios

```powershell
# Deploy multiple packs
"warfare-starwars", "economy-balanced" | Deploy-DINOForgePack -HotReload

# Check deployment status and metrics
$status = Get-DINOForgeStatus
if ($status.Success) {
    Get-DINOForgeMetrics | Select-Object -ExpandProperty Data
}

# Run tests before deployment
Invoke-DINOForgeSmoke -Scenario integration
if ($?) {
    Deploy-DINOForgePack -PackName my-mod
}

# Update module and CLI
Update-DINOForge -PreRelease
```

## Troubleshooting

### Module not found after installation

Ensure PowerShell can find the module:

```powershell
# Check module directories
$env:PSModulePath

# Manually import from path
Import-Module "C:\Users\YourUsername\Documents\PowerShell\Modules\DINOForge"

# Verify installation
Get-Module DINOForge
```

### Game path not detected

Provide explicit path:

```powershell
$gamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
Deploy-DINOForgePack -PackName my-mod -GamePath $gamePath
```

### BepInEx not found

Install BepInEx manually:

```powershell
# Download from GitHub
iwr "https://github.com/BepInEx/BepInEx/releases/download/v5.4.23.5/BepInEx_x64_5.4.23.5.zip" -OutFile "$env:TEMP\BepInEx.zip"

# Extract to game directory
Expand-Archive "$env:TEMP\BepInEx.zip" -DestinationPath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"

# Verify installation
Test-Path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx"
```

### Cmdlets not working

Verify CLI installation:

```powershell
# Check if dotnet tool is installed
dotnet tool list -g | Select-String DINOForge

# Reinstall if needed
dotnet tool update -g DINOForge.Tools.Cli --prerelease
```

## Development

### Module Structure

```
tools/PSModule/
├── DINOForge.psm1      # Module implementation
├── DINOForge.psd1      # Module manifest
└── README.md           # Module documentation

tools/
└── Install-DINOForge.ps1  # One-line installer
```

### Testing the Module Locally

```powershell
# In tools/PSModule directory
$modulePath = Get-Location
Import-Module "$modulePath\DINOForge.psm1" -Force -Verbose

# Test a cmdlet
Get-DINOForgeStatus -Verbose

# Reload after edits
Remove-Module DINOForge
Import-Module "$modulePath\DINOForge.psm1" -Force
```

### Publishing to PowerShell Gallery

```powershell
# Create NuGet API key at https://www.powershellgallery.com/users/account/ApiKeys
$apiKey = Read-Host "Enter NuGet API key"

# Publish module
Publish-Module -Path "tools/PSModule" -NuGetApiKey $apiKey -Verbose

# Verify publication
Find-Module -Name DINOForge
```

## Version Compatibility

| PowerShell | Support | Notes |
|------------|---------|-------|
| 5.1 (Desktop) | ✅ | Windows default, tested |
| 7.0+ (Core) | ✅ | Cross-platform, recommended |
| 4.0 and earlier | ❌ | Not supported |

## System Requirements

- **PowerShell**: 5.1 or later
- **.NET CLI**: .NET 8.0 or later (via global tool)
- **OS**: Windows, macOS, or Linux
- **Disk Space**: ~500 MB for installation

## See Also

- [DINOForge CLI Documentation](../README.md)
- [Installation Guide](./README.md)
- [Getting Started](./GETTING_STARTED.md)
- [Pack Development Guide](../packs/README.md)

## Support

- **Issues**: [GitHub Issues](https://github.com/KooshaPari/Dino/issues)
- **Discussions**: [GitHub Discussions](https://github.com/KooshaPari/Dino/discussions)
- **Documentation**: [kooshapari.github.io/Dino](https://kooshapari.github.io/Dino/)
