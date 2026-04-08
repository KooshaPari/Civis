# DINOForge Libification Roadmap

## Overview

Libification is the process of decomposing DINOForge into reusable, independently consumable libraries (NuGet packages, GitHub releases, crates.io packages). This enables external mod developers to integrate DINOForge tooling without adopting the entire platform.

**Status**: Tier 1 complete (v0.18.0+)

---

## Tier 1: Core Bridge & Utilities (v0.18.0 - COMPLETE)

Fundamental libraries for game bridge communication and tooling.

### 1.1 Bridge.Protocol → NuGet Package

**Status**: COMPLETE ✓

- **Package ID**: `DINOForge.Bridge.Protocol`
- **Version**: 0.18.0+
- **NuGet URL**: https://www.nuget.org/packages/DINOForge.Bridge.Protocol/
- **Target Framework**: netstandard2.0 (maximum compatibility)
- **Exports**: JSON-RPC 2.0 message DTOs, IGameBridge interface, Protocol types
- **Dependencies**: System.Text.Json only
- **Use Case**: Build game bridge clients/servers without adopting full DINOForge Runtime

**Consumption**:
```bash
dotnet add package DINOForge.Bridge.Protocol --version 0.18.0
```

```csharp
using DINOForge.Bridge.Protocol;
using System.Text.Json;

// IGameBridge interface allows custom implementations
public class MyCustomGameBridge : IGameBridge
{
    // Implement bridge methods for your game
}
```

---

### 1.2 Bridge.Client → NuGet Package

**Status**: COMPLETE ✓

- **Package ID**: `DINOForge.Bridge.Client`
- **Version**: 0.18.0+
- **NuGet URL**: https://www.nuget.org/packages/DINOForge.Bridge.Client/
- **Target Framework**: netstandard2.0 (maximum compatibility)
- **Exports**: GameClient class, out-of-process bridge communication
- **Dependencies**: DINOForge.Bridge.Protocol (0.18.0+), System.Text.Json
- **Use Case**: Connect to DINO game instances from external .NET applications

**Consumption**:
```bash
dotnet add package DINOForge.Bridge.Client --version 0.18.0
```

```csharp
using DINOForge.Bridge.Client;

var client = new GameClient("localhost", 9000);
await client.ConnectAsync();
var entities = await client.QueryEntitiesAsync("Health");
```

---

### 1.3 Go DependencyResolver → GitHub Release

**Status**: COMPLETE ✓

- **Binary Names**: `dinoforge-resolver.exe` (Windows), `dinoforge-resolver` (Linux)
- **Release URL**: https://github.com/KooshaPari/Dino/releases/tag/go-resolver-v0.1.0+
- **Algorithm**: Kahn's topological sort (dependency resolution)
- **Input**: JSON manifest with available packs and target pack
- **Output**: Ordered load sequence or conflict errors
- **Use Case**: Standalone pack dependency resolver for mod managers, third-party launchers

**Usage**:
```bash
./dinoforge-resolver \
  --input packs.json \
  --output resolved-order.json
```

**Input Format** (packs.json):
```json
{
  "available": [
    {
      "id": "pack-a",
      "name": "Pack A",
      "version": "1.0.0",
      "depends_on": ["pack-b"],
      "load_order": 100
    }
  ],
  "target": { "id": "pack-a", "depends_on": [] }
}
```

**Output Format** (resolved-order.json):
```json
{
  "resolved": ["pack-b", "pack-a"],
  "errors": []
}
```

---

### 1.4 Rust AssetPipeline → GitHub Release

**Status**: COMPLETE ✓

- **Binary Names**: `dinoforge_asset_pipeline.dll` (Windows), `libdinoforge_asset_pipeline.so` (Linux)
- **Release URL**: https://github.com/KooshaPari/Dino/releases/tag/asset-pipeline-rust-v0.1.0+
- **Language**: Rust 1.22+
- **FFI**: PyO3 for Python integration, native C ABI
- **Features**:
  - High-performance GLB/FBX import via Assimp
  - Mesh LOD generation with Rayon parallelism
  - SIMD-friendly array operations (ndarray)
- **Use Case**: Fast asset pipeline for Blender plugins, standalone asset tools

**Build from Source**:
```bash
cd src/Tools/AssetPipelineRust
cargo build --release
# Output: target/release/dinoforge_asset_pipeline.dll (Windows)
#         target/release/libdinoforge_asset_pipeline.so (Linux)
```

---

## Tier 2: Domain Plugins (Planned - v0.19.0+)

Separate reusable domain plugins as NuGet packages.

### 2.1 Domains.Warfare → NuGet Package

- **Package ID**: `DINOForge.Domains.Warfare`
- **Contents**: Archetypes, Doctrines, Roles, Wave systems, balance models
- **Dependencies**: DINOForge.SDK
- **Use Case**: Warfare mod development independent of Economy/Scenario domains

### 2.2 Domains.Economy → NuGet Package

- **Package ID**: `DINOForge.Domains.Economy`
- **Contents**: Trade engine, production models, resource systems
- **Dependencies**: DINOForge.SDK
- **Use Case**: Economy balance mods, resource management systems

### 2.3 Domains.Scenario → NuGet Package

- **Package ID**: `DINOForge.Domains.Scenario`
- **Contents**: Scripting, victory/defeat conditions, difficulty scaling
- **Dependencies**: DINOForge.SDK
- **Use Case**: Custom scenario designers, mission pack creators

### 2.4 Domains.UI → NuGet Package

- **Package ID**: `DINOForge.Domains.UI`
- **Contents**: HUD elements, menu registries, theme system
- **Dependencies**: DINOForge.SDK
- **Use Case**: UI mod developers, custom overlay creators

---

## Tier 3: CLI Tools (Planned - v0.20.0+)

Standalone, redistributable CLI tools.

### 3.1 dinoforge CLI → Standalone Executable

- **Binary**: `dinoforge` (dotnet tool or native executable)
- **Commands**: status, query, override, reload, watch, pack validate, pack build
- **Consumption**: `dotnet tool install -g dinoforge` or GitHub release download

### 3.2 PackCompiler → Standalone Executable

- **Binary**: `dinoforge-pack-compiler`
- **Commands**: validate, build, assets import/optimize/generate
- **Consumption**: GitHub release or Docker image

---

## Tier 4: Extended Platforms (Future - v0.21.0+)

Support for additional ecosystems.

### 4.1 Crates.io (Rust)

- Publish `dinoforge-asset-pipeline` to crates.io
- Enable Rust projects to depend on DINOForge asset tools via Cargo

### 4.2 npm (JavaScript)

- `@dinoforge/bridge-protocol` — TypeScript definitions + Node.js client
- `@dinoforge/pack-validator` — Pack schema validation in JavaScript

### 4.3 Python (PyPI)

- `dinoforge` — Python MCP bridge, pack utilities
- `dinoforge-asset-pipeline` — Python bindings to Rust asset tools

---

## Publishing Checklist

### Per-Release Validation

- [ ] All NuGet packages build without warnings
- [ ] Symbol packages (.snupkg) include full source maps
- [ ] XML documentation is generated and complete
- [ ] Go binaries stripped and optimized (`-ldflags="-s -w"`)
- [ ] Rust binaries compiled with `opt-level=3, lto=true`
- [ ] All checksums generated and verified
- [ ] GitHub releases include detailed changelogs per artifact
- [ ] README files document consumption patterns

### Ongoing Maintenance

- [ ] Keep versions synchronized (all libified components match main DINOForge version)
- [ ] Update README.md with new package links
- [ ] Monitor NuGet download stats and feedback
- [ ] Document breaking changes in CHANGELOG.md
- [ ] Maintain backward compatibility for at least 2 minor versions

---

## Migration Path for External Developers

### Before Libification (Current)

```bash
# Had to clone entire DINOForge repo, build locally
git clone https://github.com/KooshaPari/Dino
cd Dino
dotnet build src/DINOForge.sln
# Copy DLLs manually, no version tracking
```

### After Tier 1 (Now)

```bash
# Package dependencies as NuGet references
dotnet add package DINOForge.Bridge.Protocol
dotnet add package DINOForge.Bridge.Client
dotnet add package DINOForge.SDK

# Download Go/Rust tools from GitHub releases
# Automatic version tracking, no local builds needed
```

### After Tier 2 (Planned)

```bash
# Swap individual domain plugins
dotnet add package DINOForge.Domains.Warfare
dotnet add package DINOForge.Domains.Economy
# Omit Scenario/UI if not needed
```

### After Tier 3 (Planned)

```bash
# Install as global CLI tool
dotnet tool install -g dinoforge --version 0.20.0

# Use directly without repo clone
dinoforge pack validate my-pack/
dinoforge pack build my-pack/
```

---

## FAQ

**Q: Why separate Bridge.Protocol from Bridge.Client?**
A: Decoupling allows game developers to implement their own IGameBridge without depending on our specific JSON-RPC transport. Protocol is contracts; Client is one implementation.

**Q: Will Tier 2 domain plugins ever depend on each other?**
A: Kept as independent as possible. Warfare may depend on SDK base registry system, but not on Economy. This maximizes reuse.

**Q: When will SDK be published to NuGet?**
A: Already published as `DINOForge.SDK` (see docs/DEPLOYMENT.md). Libification extends this pattern to all packages.

**Q: Can I use Bridge.Client without Bridge.Protocol?**
A: No — Bridge.Client depends on Bridge.Protocol. Protocol first, Client builds on it.

**Q: What's the long-term versioning strategy?**
A: All libified components follow semantic versioning aligned with main DINOForge releases. v0.18.0 of Protocol = v0.18.0 of main. Breaks only for major version bumps.

---

## See Also

- [DEPLOYMENT.md](./DEPLOYMENT.md) — Package consumption guide
- [README.md](../README.md) — NuGet packages section
- [CHANGELOG.md](../CHANGELOG.md) — Per-release libification notes
- [.github/workflows/release.yml](../.github/workflows/release.yml) — Automated publishing pipeline
