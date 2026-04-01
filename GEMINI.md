# GEMINI.md — DINOForge Agent Methodology Guide

## Project Overview

**DINOForge** is a mod platform and agent-driven development scaffold for **Diplomacy is Not an Option (DINO)**.

- **Game**: Diplomacy is Not an Option (Unity ECS, BepInEx-compatible)
- **Architecture**: Polyrepo-hexagonal, declarative-first, agent-driven
- **Language**: C# (.NET), YAML/JSON schemas, CLI tooling
- **Mod Loader**: BepInEx + custom ECS plugin loader

## Stack Information

| Component | Technology |
|-----------|------------|
| .NET | 11.0 preview (net11.0 for apps, netstandard2.1 for libraries) |
| Language | C# 12+ with nullable reference types |
| Schemas | JSON/YAML via NJsonSchema, YamlDotNet |
| Testing | xUnit + FluentAssertions + Moq |
| CLI | System.CommandLine, Spectre.Console |
| Mod Loader | BepInEx |
| Game Bridge | MCP server (HTTP/SSE + stdio) |

### Build Commands

```bash
dotnet build src/DINOForge.sln
dotnet test src/DINOForge.sln --verbosity normal
dotnet format src/DINOForge.sln --verify-no-changes
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

## Code Conventions

- **C# 12+** with nullable reference types (`#nullable enable`)
- **`async/await`** over raw Tasks
- **XML doc comments** on all public APIs (triple-slash `///`)
- **Immutable data models** preferred (records, init properties)
- **Registry pattern** for all extensible content — no switch statements on content type IDs
- **No `var`** for non-obvious types
- **Meaningful names** over inline comments
- **Wrap, don't handroll** — prefer established libraries over custom implementations

## Agent Behavior Rules

### Pre-Submission Gates

Before any commit to main:

1. `dotnet test src/DINOForge.sln` — all tests must pass
2. `dotnet format src/DINOForge.sln --verify-no-changes` — no formatting drift
3. Update CHANGELOG.md [Unreleased] section
4. Commit with descriptive message + `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`

### File Governance

- **Desktop contamination**: Never write to `C:\Users\koosh\Desktop\` — use designated repo directories
- **Script lifecycle**: Temp scripts go to `scripts/game/` or `docs/scripts/`, deleted via Recycle Bin when done
- **Desktop files deletion**: Use `Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile(...)` — never `rm`/`del`

### Legal Move Classes

Reduce all work to one of these forms:

| Operation | Description |
|-----------|-------------|
| `create schema` | New data shape definition |
| `extend registry` | Add entries to existing registry |
| `add content pack` | New pack with manifest |
| `patch mapping` | Update vanilla-to-mod mapping |
| `write validator` | New validation rule |
| `add test fixture` | New test case |
| `add debug view` | New diagnostic surface |
| `add migration` | Version compatibility migration |
| `add compatibility rule` | Cross-pack conflict rule |
| `add documentation manifest` | Update docs |

### Agent Must-Nots

- **Handroll what a library already solves** — always search for existing packages first
- Patch runtime internals unless assigned runtime-layer work
- Invent new registry patterns casually
- Duplicate schemas
- Bypass validators
- Hardcode content IDs in engine glue
- Add undocumented extension points
- Skip tests
- Merge without compatibility checks

### Game Launch Protocol

1. Kill all existing instances: `Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue`
2. Wait 3 seconds and verify no processes remain
3. Launch: `Start-Process -FilePath '<exe>' -WorkingDirectory '<gamedir>'`
4. Wait 5 seconds, then check `MainWindowTitle` — if "Fatal error" or contains "another instance", launch FAILED
5. Only proceed if `MainWindowTitle` is empty/game (not an error dialog)

### Asset Pipeline (MANDATORY for asset work)

Follow this sequence **in order**:

1. **Define** — Create/update `asset_pipeline.yaml` in pack root
2. **Download** — `dotnet run -- sync download <pack>`
3. **Import** — `dotnet run -- assets import <pack>`
4. **Validate** — `dotnet run -- assets validate <pack>`
5. **Optimize** — `dotnet run -- assets optimize <pack>`
6. **Generate** — `dotnet run -- assets generate <pack>`
7. **Verify** — `dotnet run -- assets build <pack>`
8. **Commit** — Git commit all artifacts + updated definitions

Never manually edit game definitions when assets change, skip validation/optimization steps, or create ad-hoc asset directories outside `packs/<pack>/assets/`.

## Key Design Principles

1. **Wrap, don't handroll** — Use established libraries/tools and wrap them
2. **Framework before content** — platform first, themed mods second
3. **Declarative before imperative** — YAML/JSON manifests over C# patches
4. **Stable abstraction over unstable internals** — isolate ECS glue
5. **Agent-first repo design** — optimize for autonomous agent dev
6. **Observability is first-class** — logs, overlays, reports, validators
7. **Domain extensibility** — warfare is first plugin, not the only one
8. **Compatibility-aware packaging** — explicit deps, conflicts, versions
9. **Graceful degradation** — fail loudly with fallbacks

## Repository Structure

```
DINOForge/
  src/
    Runtime/             # BepInEx plugin: bootstrap, ECS detection, debug overlay
    SDK/                 # Public mod API: registries, schemas, pack loaders
    Bridge/              # JSON-RPC message types and IGameBridge interface
    Domains/             # Domain plugins (Warfare, Economy, Scenario, UI)
    Tools/               # CLI, MCP server, PackCompiler, DumpTools
    Tests/               # Unit tests + integration tests
  packs/                 # Content packs (warfare-*, economy-*, scenario-*)
  schemas/               # Canonical JSON/YAML schema definitions
  docs/                  # All project documentation
  manifests/             # System contracts, ownership maps, extension points
```

## MCP Server Tools

Available when game is running with BepInEx and DINOForge Runtime loaded:

| Tool | Purpose |
|------|---------|
| `game_status` | Check if game is running and mods loaded |
| `list_packs` | List all loaded content packs with versions |
| `query_entity` | Inspect a specific ECS entity |
| `list_units` | Enumerate all registered units with stats |
| `spawn_unit` | Request unit spawn at world position |
| `apply_override` | Apply a stat override at runtime |
| `reload_packs` | Trigger hot module replacement |
| `dump_world` | Dump current ECS world state |
| `get_logs` | Read dinoforge_debug.log |

## See Also

- `CLAUDE.md` — General context management and delegation strategy
- `AGENTS.md` — Detailed documentation organization and linting governance
