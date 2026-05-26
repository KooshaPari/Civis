# v0.17.0 Deployment Guide

## Prerequisites

- Windows 10+ or Linux (Ubuntu 20.04+)
- .NET 11 preview SDK (or .NET 8+ for SDK usage)
- Python 3.10+
- Rust 1.70+, Go 1.21+, Zig 0.16.0-dev (optional, for source builds)
- Task CLI 3.49+ (https://taskfile.dev/)

## Installation

### From NuGet (C#)
```bash
dotnet add package DINOForge.SDK --version 0.17.0
dotnet add package DINOForge.Bridge.Protocol --version 0.17.0
dotnet add package DINOForge.Bridge.Client --version 0.17.0
```

### From Source (All Languages)
```bash
git clone https://github.com/KooshaPari/Dino.git
cd Dino

# Build everything
task build:all

# Test everything
task test:all
```

### Quick Validation
```bash
# C# build + tests
dotnet build src/DINOForge.sln -c Release
dotnet test src/DINOForge.sln

# Pre-commit validation
task precommit

# Individual language tests
task test:csharp
task test:rust
task test:go
task test:zig
task test:python
```

## What's Included

- **SDK**: All registries, validators, pack loaders (C#)
- **Runtime**: BepInEx plugin for Diplomacy is Not an Option
- **MCP Server**: 21 game automation tools (Python)
- **Asset Pipeline**: Rust (FFI), Go (resolver), Zig (mesh ops)
- **Documentation**: Full API docs + developer guides

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `dotnet` not found | Install .NET 11 preview or .NET 8 SDK |
| `zig` not found | Download from https://ziglang.org/download/ |
| pytest fails | `pip install -e ".[dev]"` in DinoforgeMcp dir |
| Task not found | Install task CLI: `choco install task` (Windows) |

## Support

- GitHub Issues: https://github.com/KooshaPari/Dino/issues
- Documentation: docs/POLYGLOT_BUILD.md
- Agent Governance: CLAUDE.md
