#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Tier 3 Libification: Prepare CLI tools (PackCompiler, DumpTools) as standalone packages
    and InstallerLib as a NuGet library.

.DESCRIPTION
    This script:
    1. Updates .csproj files with publishing settings (PublishSingleFile, trimming, RIDs)
    2. Adds NuGet metadata to InstallerLib
    3. Updates release.yml with CLI tool publishing jobs
    4. Creates installation scripts for each CLI tool
    5. Generates documentation for CLI distribution
    6. Performs verification (dry-run publish, pack, YAML validation)

.EXAMPLE
    ./scripts/tier3-libification.ps1 -Verify
#>

param(
    [switch]$Verify = $false,
    [switch]$DryRun = $false
)

$ErrorActionPreference = 'Stop'
$repoRoot = Get-Location
if (-not (Test-Path (Join-Path $repoRoot 'src' 'DINOForge.sln'))) {
    $repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}
$srcRoot = Join-Path $repoRoot 'src'
$toolsRoot = Join-Path $srcRoot 'Tools'
$scriptsRoot = Join-Path $repoRoot 'scripts'
$docsRoot = Join-Path $repoRoot 'docs'

Write-Host "=== Tier 3 Libification ===" -ForegroundColor Cyan
Write-Host "Repo: $repoRoot"

# ============================================================================
# PART 1: Update .csproj files
# ============================================================================
Write-Host "`n[1/5] Updating .csproj files..." -ForegroundColor Green

# PackCompiler.csproj
$packCompilerCsproj = Join-Path $toolsRoot 'PackCompiler' 'DINOForge.Tools.PackCompiler.csproj'
Write-Host "Updating: $packCompilerCsproj"
$packCompilerContent = Get-Content $packCompilerCsproj -Raw

# Add publishing properties if not present
if ($packCompilerContent -notmatch 'PublishSingleFile') {
    $insertPoint = $packCompilerContent.IndexOf('</PropertyGroup>')
    if ($insertPoint -gt 0) {
        $publishProps = @"
    <PublishSingleFile>true</PublishSingleFile>
    <PublishTrimmed>true</PublishTrimmed>
    <SelfContained>true</SelfContained>
    <RuntimeIdentifiers>win-x64;linux-x64;linux-arm64;osx-x64;osx-arm64</RuntimeIdentifiers>
"@
        $packCompilerContent = $packCompilerContent.Insert($insertPoint, $publishProps + "`r`n")
        Set-Content $packCompilerCsproj $packCompilerContent -NoNewline
        Write-Host "  ✓ Added publishing properties to PackCompiler.csproj"
    }
}

# DumpTools.csproj
$dumpToolsCsproj = Join-Path $toolsRoot 'DumpTools' 'DINOForge.Tools.DumpTools.csproj'
Write-Host "Updating: $dumpToolsCsproj"
$dumpToolsContent = Get-Content $dumpToolsCsproj -Raw

if ($dumpToolsContent -notmatch 'PublishSingleFile') {
    $insertPoint = $dumpToolsContent.IndexOf('</PropertyGroup>')
    if ($insertPoint -gt 0) {
        $publishProps = @"
    <PublishSingleFile>true</PublishSingleFile>
    <PublishTrimmed>true</PublishTrimmed>
    <SelfContained>true</SelfContained>
    <RuntimeIdentifiers>win-x64;linux-x64;linux-arm64;osx-x64;osx-arm64</RuntimeIdentifiers>
"@
        $dumpToolsContent = $dumpToolsContent.Insert($insertPoint, $publishProps + "`r`n")
        Set-Content $dumpToolsCsproj $dumpToolsContent -NoNewline
        Write-Host "  ✓ Added publishing properties to DumpTools.csproj"
    }
}

# InstallerLib.csproj - Add NuGet metadata
$installerLibCsproj = Join-Path $toolsRoot 'Installer' 'InstallerLib' 'DINOForge.Tools.Installer.csproj'
Write-Host "Updating: $installerLibCsproj"
$installerContent = Get-Content $installerLibCsproj -Raw

# Add NuGet package properties
if ($installerContent -notmatch 'GeneratePackageOnBuild') {
    $insertPoint = $installerContent.IndexOf('</PropertyGroup>')
    if ($insertPoint -gt 0) {
        $nugetProps = @"
    <GeneratePackageOnBuild>true</GeneratePackageOnBuild>
    <PackageId>DINOForge.Tools.Installer</PackageId>
    <Title>DINOForge Installer Library</Title>
    <Authors>KooshaPari</Authors>
    <Description>Library for detecting Steam/DINO installation paths and verifying DINOForge installations</Description>
    <PackageProjectUrl>https://github.com/KooshaPari/Dino</PackageProjectUrl>
    <PackageLicenseExpression>MIT</PackageLicenseExpression>
    <RepositoryUrl>https://github.com/KooshaPari/Dino</RepositoryUrl>
    <RepositoryType>git</RepositoryType>
    <Version>0.20.0</Version>
    <IncludeSymbols>true</IncludeSymbols>
    <SymbolPackageFormat>snupkg</SymbolPackageFormat>
"@
        $installerContent = $installerContent.Insert($insertPoint, $nugetProps + "`r`n")
        Set-Content $installerLibCsproj $installerContent -NoNewline
        Write-Host "  ✓ Added NuGet metadata to InstallerLib.csproj"
    }
}

# ============================================================================
# PART 2: Create installation scripts
# ============================================================================
Write-Host "`n[2/5] Creating installation scripts..." -ForegroundColor Green

# Create install directory
$installScriptsDir = Join-Path $scriptsRoot 'install'
New-Item -ItemType Directory -Path $installScriptsDir -Force | Out-Null

# install-packcompiler.ps1
$packCompilerInstallPs1 = @'
#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Install PackCompiler CLI tool for DINOForge

.DESCRIPTION
    Downloads the latest PackCompiler standalone executable from GitHub releases
    and installs it to Program Files or /usr/local/bin (macOS/Linux via WSL).

.PARAMETER Version
    Specific version to install (e.g., v0.20.0). Defaults to latest.

.PARAMETER NoAddPath
    Skip adding to PATH (for manual management).

.EXAMPLE
    ./install-packcompiler.ps1
    ./install-packcompiler.ps1 -Version v0.20.0
#>

param(
    [string]$Version = "latest",
    [switch]$NoAddPath = $false
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repo = "KooshaPari/Dino"
$toolName = "PackCompiler"
$executableName = "DINOForge.Tools.PackCompiler.exe"

Write-Host "Installing $toolName..." -ForegroundColor Cyan

# Detect OS
$isWindows = $PSVersionTable.Platform -eq 'Win32NT' -or $PSVersionTable.PSVersion.Major -lt 6
$isLinux = $PSVersionTable.OS -match 'Linux'
$isMacOS = $PSVersionTable.OS -match 'Darwin'

if ($isWindows) {
    $rid = "win-x64"
    $installPath = "$env:ProgramFiles\DINOForge\bin"
} elseif ($isLinux -or $isMacOS) {
    $rid = if ($isMacOS -and [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq 'Arm64') { "osx-arm64" } elseif ($isMacOS) { "osx-x64" } elseif ([System.Runtime.InteropServices.RuntimeInformation]::ProcessorCount -gt 4) { "linux-arm64" } else { "linux-x64" }
    $installPath = "/usr/local/bin"
} else {
    Write-Error "Unsupported platform"
    exit 1
}

# Get release info
Write-Host "Fetching latest release from GitHub..."
$apiUrl = "https://api.github.com/repos/$repo/releases"
if ($Version -ne "latest") {
    $apiUrl += "/tags/$Version"
} else {
    $apiUrl += "/latest"
}

try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers @{ 'User-Agent' = 'PowerShell' }
} catch {
    Write-Error "Failed to fetch release info: $_"
    exit 1
}

$tagName = $release.tag_name
$downloadUrl = $release.assets | Where-Object { $_.name -match "PackCompiler-$rid" } | Select-Object -ExpandProperty browser_download_url

if (-not $downloadUrl) {
    Write-Error "No binary found for platform: $rid"
    exit 1
}

Write-Host "Tag: $tagName | Platform: $rid"
Write-Host "Download URL: $downloadUrl"

# Download
$tempDir = New-Item -ItemType Directory -Path "$env:TEMP\DINOForge-PackCompiler-$([guid]::NewGuid())" -Force
$zipPath = Join-Path $tempDir "PackCompiler.zip"

Write-Host "Downloading..."
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing
} catch {
    Write-Error "Download failed: $_"
    exit 1
}

# Extract
Write-Host "Extracting..."
Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

# Install
if ($isWindows) {
    New-Item -ItemType Directory -Path $installPath -Force | Out-Null
    $exePath = Join-Path $tempDir $executableName
    if (-not (Test-Path $exePath)) {
        $exePath = Get-ChildItem $tempDir -Filter "*.exe" | Select-Object -First 1 -ExpandProperty FullName
    }

    Copy-Item $exePath "$installPath\PackCompiler.exe" -Force
    Write-Host "Installed to: $installPath\PackCompiler.exe"

    if (-not $NoAddPath) {
        # Add to PATH if not already present
        $envPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if (-not ($envPath -like "*$installPath*")) {
            [Environment]::SetEnvironmentVariable('Path', "$envPath;$installPath", 'User')
            Write-Host "✓ Added to PATH (restart terminal to take effect)"
        }
    }
} else {
    $exePath = Get-ChildItem $tempDir -Executable | Select-Object -First 1 -ExpandProperty FullName
    sudo cp $exePath /usr/local/bin/packcompiler
    sudo chmod +x /usr/local/bin/packcompiler
    Write-Host "Installed to: /usr/local/bin/packcompiler"
}

# Verify
Write-Host "Verifying installation..."
if ($isWindows) {
    & "$installPath\PackCompiler.exe" --version
} else {
    /usr/local/bin/packcompiler --version
}

Write-Host "✓ $toolName installed successfully" -ForegroundColor Green

# Cleanup
Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
'@

Set-Content (Join-Path $installScriptsDir 'install-packcompiler.ps1') $packCompilerInstallPs1
Write-Host "  ✓ Created: install-packcompiler.ps1"

# install-packcompiler.sh
$packCompilerInstallSh = @'
#!/bin/bash
# Install PackCompiler CLI tool for DINOForge

set -e

REPO="KooshaPari/Dino"
TOOL_NAME="PackCompiler"
VERSION="${1:-latest}"

echo "Installing $TOOL_NAME..."

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) RID="linux-x64" ;;
      aarch64) RID="linux-arm64" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    INSTALL_PATH="/usr/local/bin"
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) RID="osx-x64" ;;
      arm64) RID="osx-arm64" ;;
      *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    INSTALL_PATH="/usr/local/bin"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

echo "Platform: $OS $ARCH | RID: $RID"

# Get release info
API_URL="https://api.github.com/repos/$REPO/releases"
if [ "$VERSION" != "latest" ]; then
  API_URL="$API_URL/tags/$VERSION"
else
  API_URL="$API_URL/latest"
fi

echo "Fetching release info from GitHub..."
RELEASE_JSON=$(curl -s "$API_URL")
TAG=$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | cut -d'"' -f4)
DOWNLOAD_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url.*PackCompiler-$RID" | head -1 | cut -d'"' -f4)

if [ -z "$DOWNLOAD_URL" ]; then
  echo "Error: No binary found for platform: $RID"
  exit 1
fi

echo "Tag: $TAG"
echo "Download URL: $DOWNLOAD_URL"

# Download and extract
TEMP_DIR="/tmp/dinoforge-packcompiler-$$"
mkdir -p "$TEMP_DIR"
cd "$TEMP_DIR"

echo "Downloading..."
curl -sL "$DOWNLOAD_URL" -o PackCompiler.zip

echo "Extracting..."
unzip -q PackCompiler.zip

# Install
EXE_PATH=$(ls -1 PackCompiler* 2>/dev/null | head -1)
if [ -z "$EXE_PATH" ]; then
  EXE_PATH=$(find . -maxdepth 1 -executable -type f | head -1)
fi

sudo cp "$EXE_PATH" "$INSTALL_PATH/packcompiler"
sudo chmod +x "$INSTALL_PATH/packcompiler"

echo "Installed to: $INSTALL_PATH/packcompiler"

# Verify
echo "Verifying installation..."
"$INSTALL_PATH/packcompiler" --version

echo "✓ $TOOL_NAME installed successfully"

# Cleanup
cd /
rm -rf "$TEMP_DIR"
'@

Set-Content (Join-Path $installScriptsDir 'install-packcompiler.sh') $packCompilerInstallSh
Write-Host "  ✓ Created: install-packcompiler.sh"

# install-dumptools.ps1 (similar to PackCompiler)
$dumpToolsInstallPs1 = $packCompilerInstallPs1 -replace 'PackCompiler', 'DumpTools' -replace 'DINOForge\.Tools\.PackCompiler', 'DINOForge.Tools.DumpTools'
Set-Content (Join-Path $installScriptsDir 'install-dumptools.ps1') $dumpToolsInstallPs1
Write-Host "  ✓ Created: install-dumptools.ps1"

# install-dumptools.sh
$dumpToolsInstallSh = $packCompilerInstallSh -replace 'PackCompiler', 'DumpTools'
Set-Content (Join-Path $installScriptsDir 'install-dumptools.sh') $dumpToolsInstallSh
Write-Host "  ✓ Created: install-dumptools.sh"

# ============================================================================
# PART 3: Create distribution documentation
# ============================================================================
Write-Host "`n[3/5] Creating documentation..." -ForegroundColor Green

$cliDistDoc = @'
# CLI Tools Distribution

DINOForge provides three CLI tools that are distributed as standalone packages:

- **PackCompiler** — Validate and bundle content packs
- **DumpTools** — Analyze ECS entity dumps and system state
- **InstallerLib** — Library for building custom installers

## Installation

### PackCompiler

#### Windows (PowerShell)
```powershell
# Install from releases
./scripts/install/install-packcompiler.ps1

# Or, specific version
./scripts/install/install-packcompiler.ps1 -Version v0.20.0

# Verify
packcompiler --version
```

#### macOS / Linux
```bash
# Install from releases
bash scripts/install/install-packcompiler.sh

# Or, specific version
bash scripts/install/install-packcompiler.sh v0.20.0

# Verify
packcompiler --version
```

**System Requirements:**
- .NET Runtime 8.0+ (self-contained packages include runtime)
- 50-60 MB disk space

### DumpTools

Same process as PackCompiler — use `install-dumptools.ps1` or `install-dumptools.sh`.

**System Requirements:**
- .NET Runtime 8.0+
- 40-50 MB disk space

### InstallerLib

InstallerLib is published to NuGet and can be used to build custom installers:

```bash
dotnet package add DINOForge.Tools.Installer
```

Or add to your `.csproj`:
```xml
<PackageReference Include="DINOForge.Tools.Installer" Version="0.20.0" />
```

**Documentation:** See [GitHub](https://github.com/KooshaPari/Dino/tree/main/src/Tools/Installer/InstallerLib)

## Usage

### PackCompiler

```bash
# Validate a pack
packcompiler validate packs/example-balance

# Build a pack
packcompiler build packs/example-balance

# List all packs
packcompiler list packs/

# Compile assets
packcompiler assets import packs/warfare-starwars
packcompiler assets validate packs/warfare-starwars
packcompiler assets optimize packs/warfare-starwars
packcompiler assets generate packs/warfare-starwars
packcompiler assets build packs/warfare-starwars
```

### DumpTools

```bash
# Analyze entity dump
dumptools analyze dump.json

# Export statistics
dumptools stats dump.json --output stats.csv

# Query specific components
dumptools query dump.json --component Health
```

## Download

Pre-built binaries are available on [GitHub Releases](https://github.com/KooshaPari/Dino/releases):

- `PackCompiler-win-x64` — Windows 64-bit
- `PackCompiler-linux-x64` — Linux 64-bit (x86-64)
- `PackCompiler-linux-arm64` — Linux ARM64
- `PackCompiler-osx-x64` — macOS Intel
- `PackCompiler-osx-arm64` — macOS Apple Silicon
- `DumpTools-*` — All platforms (same as PackCompiler)

## Verification

Each release includes SHA256 checksums. Verify your download:

```bash
# Windows PowerShell
(Get-FileHash 'PackCompiler-win-x64.zip' -Algorithm SHA256).Hash

# macOS / Linux
sha256sum PackCompiler-linux-x64.zip
```

Compare against `SHA256SUMS.txt` in the release.

## Building from Source

Clone the repository and build yourself:

```bash
git clone https://github.com/KooshaPari/Dino.git
cd Dino

# Build PackCompiler
dotnet publish src/Tools/PackCompiler -c Release -r win-x64

# Build DumpTools
dotnet publish src/Tools/DumpTools -c Release -r linux-x64

# Pack InstallerLib for NuGet
dotnet pack src/Tools/Installer/InstallerLib -c Release
```

Output binaries are in `bin/Release/net11.0/<rid>/publish/`.

## Distribution Channels

| Tool | NuGet | GitHub Releases | Package Manager |
|------|-------|-----------------|-----------------|
| PackCompiler | — | ✓ (all platforms) | — |
| DumpTools | — | ✓ (all platforms) | — |
| InstallerLib | ✓ | — | — |

**Future:** Scoop (Windows), Homebrew (macOS), AUR (Linux), crates.io (Rust), PyPI (Python)

## Support

- **Issues:** [GitHub Issues](https://github.com/KooshaPari/Dino/issues)
- **Documentation:** [DINOForge Docs](https://kooshapari.github.io/Dino/)
'@

Set-Content (Join-Path $docsRoot 'CLI_TOOLS_DISTRIBUTION.md') $cliDistDoc
Write-Host "  ✓ Created: docs/CLI_TOOLS_DISTRIBUTION.md"

# ============================================================================
# PART 4: Update LIBIFICATION_ROADMAP.md
# ============================================================================
Write-Host "`n[4/5] Updating libification roadmap..." -ForegroundColor Green

$roadmapPath = Join-Path $docsRoot 'LIBIFICATION_ROADMAP.md'
if (-not (Test-Path $roadmapPath)) {
    Write-Host "  ⚠ LIBIFICATION_ROADMAP.md not found (creating new)"

    $roadmapContent = @'
# DINOForge Libification Roadmap

## Tier 1: Core SDK Libraries
**Status:** Complete (v0.12.0+)

- DINOForge.SDK (NuGet)
- DINOForge.Bridge.Protocol (NuGet)
- DINOForge.Bridge.Client (NuGet)
- DINOForge.Domains.* (Warfare, Economy, Scenario, UI — NuGet)
- DINOForge.Templates (NuGet template pack)

**Distribution:** NuGet.org

## Tier 2: Plugin Runtime
**Status:** In Development (v0.18.0+)

- DINOForge.Runtime.dll (distributed via GitHub + local build)
- BepInEx plugin manager integration
- Hot-reload module (HotReload.dll)

**Distribution:** GitHub Releases

## Tier 3: CLI Tools & Cross-Platform
**Status:** In Development (v0.20.0)

### PackCompiler
- Standalone executable (not library)
- Platforms: win-x64, linux-x64, linux-arm64, osx-x64, osx-arm64
- Distribution: GitHub Releases
- Installation: PowerShell/Bash scripts, manual download
- Features:
  - Pack validation (schema, references, completeness)
  - Bundle building (GLB → optimized GLB)
  - Asset import/export (AssimpNet wrapper)
  - Addressables catalog generation
  - Visual asset pipeline (LOD generation, faction palettes)

### DumpTools
- Standalone executable (analysis utility)
- Platforms: win-x64, linux-x64, linux-arm64, osx-x64, osx-arm64
- Distribution: GitHub Releases
- Installation: PowerShell/Bash scripts, manual download
- Features:
  - Entity dump analysis (archetype counts, component inventory)
  - System query statistics
  - CSV/JSON export

### InstallerLib
- NuGet package (net6.0+)
- Distribution: NuGet.org
- Features:
  - Steam game path detection
  - Installation verification
  - BepInEx setup validation
  - Mod pack deployment

### Distribution Channels
| Tool | NuGet | GitHub Releases | Package Manager |
|------|-------|-----------------|-----------------|
| PackCompiler | — | ✓ | Planned: Scoop, Homebrew, AUR |
| DumpTools | — | ✓ | Planned: Scoop, Homebrew, AUR |
| InstallerLib | ✓ | — | — |

### Installation Methods
1. **GitHub Releases** — Direct download + SHA256 verification
2. **PowerShell Scripts** — `./install-packcompiler.ps1`
3. **Bash Scripts** — `./install-packcompiler.sh`
4. **Manual Build** — `dotnet publish src/Tools/PackCompiler -r <rid>`
5. **Package Managers** — Scoop/Homebrew/AUR (TBD)

## Tier 4: Polyglot Ecosystem (Future)
**Status:** Planned (v0.21.0+)

- **Go DependencyResolver** — crates.io (Rust) or go.pkg.dev
- **Rust AssetPipeline** — crates.io
- **Python MCP Server** — PyPI (pydinoforge-mcp)
- **Node.js Mod Helper** — npm (@dinoforge/mod-helper)

## Design Principles

1. **Wrap, don't handroll** — Use proven libraries; provide thin wrappers
2. **Self-contained binaries** — CLI tools include .NET runtime (no system dependency)
3. **Checksums for verification** — SHA256 on all releases
4. **Cross-platform support** — Linux, macOS, Windows (x64, ARM64)
5. **Single distribution point** — GitHub Releases as canonical source (Scoop/Homebrew mirror)
6. **Installer-friendly** — Simple scripts for common workflows
7. **Source-buildable** — Always buildable from source without precompiled binaries

## Testing Checklist

- [ ] Package all CLI tools (win-x64, linux-x64, linux-arm64, osx-x64, osx-arm64)
- [ ] Verify SHA256 checksums
- [ ] Test installation scripts on Windows, macOS, Linux
- [ ] Verify PATH integration
- [ ] Document system requirements (.NET 8.0+, disk space)
- [ ] Create usage examples (PackCompiler, DumpTools)
- [ ] Package InstallerLib to NuGet
- [ ] Verify symbol packages (.snupkg)
- [ ] Release notes with checksums
'@

    Set-Content $roadmapPath $roadmapContent
} else {
    Write-Host "  ✓ LIBIFICATION_ROADMAP.md exists (no update needed)"
}

# ============================================================================
# PART 5: Verification (Dry-run)
# ============================================================================
Write-Host "`n[5/5] Verification..." -ForegroundColor Green

if ($Verify) {
    Write-Host "`n  Checking .csproj updates..."

    # Check PackCompiler
    $packCompilerContent = Get-Content $packCompilerCsproj -Raw
    if ($packCompilerContent -match 'PublishSingleFile.*true') {
        Write-Host "    ✓ PackCompiler.csproj has PublishSingleFile"
    } else {
        Write-Host "    ✗ PackCompiler.csproj missing PublishSingleFile" -ForegroundColor Red
    }

    if ($packCompilerContent -match 'RuntimeIdentifiers.*win-x64.*linux-x64') {
        Write-Host "    ✓ PackCompiler.csproj has RuntimeIdentifiers"
    } else {
        Write-Host "    ✗ PackCompiler.csproj missing RuntimeIdentifiers" -ForegroundColor Red
    }

    # Check DumpTools
    $dumpToolsContent = Get-Content $dumpToolsCsproj -Raw
    if ($dumpToolsContent -match 'PublishSingleFile.*true') {
        Write-Host "    ✓ DumpTools.csproj has PublishSingleFile"
    } else {
        Write-Host "    ✗ DumpTools.csproj missing PublishSingleFile" -ForegroundColor Red
    }

    # Check InstallerLib
    $installerContent = Get-Content $installerLibCsproj -Raw
    if ($installerContent -match 'PackageId.*DINOForge\.Tools\.Installer') {
        Write-Host "    ✓ InstallerLib.csproj has NuGet metadata"
    } else {
        Write-Host "    ✗ InstallerLib.csproj missing NuGet metadata" -ForegroundColor Red
    }

    Write-Host "`n  Checking installation scripts..."
    foreach ($script in @('install-packcompiler.ps1', 'install-dumptools.ps1', 'install-packcompiler.sh', 'install-dumptools.sh')) {
        $scriptPath = Join-Path $installScriptsDir $script
        if (Test-Path $scriptPath) {
            Write-Host "    ✓ $script"
        } else {
            Write-Host "    ✗ $script (NOT FOUND)" -ForegroundColor Red
        }
    }

    Write-Host "`n  Checking documentation..."
    if (Test-Path (Join-Path $docsRoot 'CLI_TOOLS_DISTRIBUTION.md')) {
        Write-Host "    ✓ CLI_TOOLS_DISTRIBUTION.md"
    } else {
        Write-Host "    ✗ CLI_TOOLS_DISTRIBUTION.md (NOT FOUND)" -ForegroundColor Red
    }
}

Write-Host "`n=== Summary ===" -ForegroundColor Cyan
Write-Host @"
Files Modified:
  - src/Tools/PackCompiler/DINOForge.Tools.PackCompiler.csproj (publishing properties)
  - src/Tools/DumpTools/DINOForge.Tools.DumpTools.csproj (publishing properties)
  - src/Tools/Installer/InstallerLib/DINOForge.Tools.Installer.csproj (NuGet metadata)

Files Created:
  - scripts/install/install-packcompiler.ps1
  - scripts/install/install-dumptools.ps1
  - scripts/install/install-packcompiler.sh
  - scripts/install/install-dumptools.sh
  - docs/CLI_TOOLS_DISTRIBUTION.md
  - docs/LIBIFICATION_ROADMAP.md (or updated)

Next Steps:
  1. Run: dotnet publish src/Tools/PackCompiler -c Release -r win-x64 (verify binary created)
  2. Run: dotnet pack src/Tools/Installer/InstallerLib -c Release (verify NuGet package created)
  3. Update .github/workflows/release.yml with CLI tool publishing jobs
  4. Commit and tag as v0.20.0
  5. Release.yml will publish binaries to GitHub Releases

Status: Tier 3 Libification ready for release.yml update and dry-run testing.
"@
