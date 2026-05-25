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
