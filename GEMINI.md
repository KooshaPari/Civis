# DINOForge — Gemini CLI Context

## Project Overview

**DINOForge** is a general-purpose mod platform and agent-oriented development scaffold for **Diplomacy is Not an Option (DINO)**. It is a mod operating system, not a single mod.

- **Game**: Diplomacy is Not an Option (Unity ECS, BepInEx-compatible)
- **Architecture**: Polyrepo-hexagonal, declarative-first, agent-driven
- **Language**: C# (.NET), YAML/JSON schemas, CLI tooling
- **Mod Loader**: BepInEx + custom ECS plugin loader (`BepInEx/ecs_plugins/`)

## Stack Info

- **.NET Version**: .NET 11 preview (net11.0 for tools, netstandard2.0 for core libraries)
- **Build**: `dotnet build src/DINOForge.sln`
- **Test**: `dotnet test src/DINOForge.sln`
- **Lint**: `dotnet format src/DINOForge.sln --verify-no-changes`
- **Schema**: 24 JSON schemas for validation
- **Registries**: Units, buildings, factions, weapons, doctrines, skills, waves, squads

## Code Conventions

### C# Standards
- C# 12+ with nullable reference types (`#nullable enable`)
- `async/await` over raw Tasks
- XML doc comments on all public APIs (triple-slash `///`)
- Immutable data models preferred (records, init properties)
- Registry pattern for all extensible content — no switch statements on content type IDs
- No `var` for non-obvious types

### File Organization
```
src/
  Runtime/        # BepInEx plugin, ECS bridge, hot reload, debug overlay
  SDK/            # Registries, schemas, content loader, assets
  Domains/        # Warfare, Economy, Scenario domain plugins
  Tools/          # CLI tools, MCP server, pack compiler, dump tools
  Tests/          # Unit and integration tests
packs/            # Content packs (YAML manifests)
schemas/          # JSON Schema definitions
```

### Pack Content
- Pack content is YAML; behavior is C#
- Never mix declarative data with imperative logic
- Every pack has a `pack.yaml` manifest

## Agent Behavior Rules

### Key Invariants
1. All tests must pass before any commit to main
2. Never hardcode content IDs in engine code — always use registry lookup
3. Every public API needs XML doc comments
4. Every new schema needs a test fixture
5. Schemas are source-of-truth
6. No breaking changes without migration path
7. Registry pattern for all extensible content

### Legal Move Classes
- `create schema` — new data shape definition
- `extend registry` — add entries to existing registry
- `add content pack` — new pack with manifest
- `patch mapping` — update vanilla-to-mod mapping
- `write validator` — new validation rule
- `add test fixture` — new test case
- `add migration` — version compatibility migration
- `add compatibility rule` — cross-pack conflict rule

### Wrap Don't Handroll
Always prefer established libraries:
- JSON schema: JsonSchema.Net or NJsonSchema
- YAML: YamlDotNet + System.Text.Json
- CLI: System.CommandLine or Spectre.Console
- Logging: Serilog or NLog via BepInEx
- Testing: xUnit + FluentAssertions + Moq

## Build Commands

```bash
dotnet build src/DINOForge.sln
dotnet test src/DINOForge.sln --verbosity normal
dotnet format src/DINOForge.sln --verify-no-changes
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

## Architecture Layers

```
┌─────────────────────────────────┐
│  Pack (priority 3000+)          │  ← Mod content overrides
├─────────────────────────────────┤
│  Domain Plugin (priority 2000+) │  ← Warfare/Economy defaults
├─────────────────────────────────┤
│  Framework (priority 1000+)     │  ← DINOForge defaults
├─────────────────────────────────┤
│  Base Game (priority 0+)        │  ← Vanilla DINO values
└─────────────────────────────────┘
  Higher priority wins. Same priority = conflict detected.
```

## MCP Server Tools

The `dinoforge` MCP server exposes game automation tools:
- `game_launch`, `game_status`, `game_query_entities`
- `game_get_stat`, `game_apply_override`, `game_reload_packs`
- `game_screenshot`, `game_ui_automation`, `game_input`
- `game_analyze_screen`, `game_wait_and_screenshot`, `game_navigate_to`

Start MCP: `./scripts/start-mcp.ps1 -Detached -Watch`
