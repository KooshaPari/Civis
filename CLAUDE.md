# DINOForge - CLAUDE.md

## Project Overview

**DINOForge** is a general-purpose mod platform and agent-oriented development scaffold for **Diplomacy is Not an Option (DINO)**. It is a **mod operating system**, not a single mod.

- **Game**: Diplomacy is Not an Option (Unity ECS, BepInEx-compatible)
- **Architecture**: Polyrepo-hexagonal, declarative-first, agent-driven
- **Language**: C# (.NET), YAML/JSON schemas, CLI tooling
- **Mod Loader**: BepInEx + custom ECS plugin loader (`BepInEx/ecs_plugins/`)

## .NET Version Policy (MANDATORY — DO NOT CHANGE WITHOUT CHECKING)

**This repo uses .NET 11 preview.** Locally: `11.0.100-preview.2.26159.112`. This is intentional.
- .NET 11 EXISTS — see https://dotnet.microsoft.com/download/dotnet/11.0
- Tool/app projects target `net11.0`. Core SDK/domain libraries target `net8.0` (netstandard compat).
- CI installs .NET 11 preview via `include-prerelease: true` in `setup-dotnet`.
- **NEVER downgrade `net11.0` TFMs to net9.0/net8.0.** If a build fails on CI due to SDK version, fix the CI workflow to install .NET 11, not the other way around.
- `global.json` pins `11.0.100-preview.2.26159.112` with `latestMajor` rollforward — do not change this.

## Agent Operational Rules (MANDATORY)

### Claude (Orchestrator) Constraints
The top-level Claude instance is ONLY allowed to:
- Read and write documentation files
- Spawn Haiku subagents for ALL other work

**Everything else MUST be delegated to subagents (model: haiku):**
- All Bash/shell commands
- All file reads beyond the first 3 files per task
- All file reads of files > 500 lines
- All code edits and writes
- All builds, deployments, test runs
- All git operations
- All log analysis
- All game launch/kill operations

### Tooling Evolution Rule
Continuously reduce custom workflows to single optimal CLI/MCP calls:
- When a multi-step workflow is identified, create/update a `.claude/commands/` skill or MCP tool to collapse it into one call
- Prefer updating existing slash commands over creating new ones
- After completing any workflow, evaluate whether it should become a permanent command
- Agent plugins, skills, CLI tools, and MCP server MUST be kept up to date to reflect the shortest possible path to any common operation

### Game Launch Protocol (via subagent)
Always use this sequence — never assume a launch succeeded:
1. Kill all existing instances: `Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue`
2. Wait 3 seconds and verify no processes remain
3. Launch: `Start-Process -FilePath '<exe>' -WorkingDirectory '<gamedir>'`
4. Wait 5 seconds, then check `MainWindowTitle` — if "Fatal error" or contains "another instance", launch FAILED
5. Only proceed if MainWindowTitle is empty/game (not an error dialog)

### File Deletion Protocol (MANDATORY)
NEVER use `rm`, `del`, `Remove-Item`, or any command that permanently deletes files.
ALWAYS send files to the Windows Recycle Bin using:

For files:
```powershell
Add-Type -AssemblyName Microsoft.VisualBasic
[Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile(
    '<absolute-path>',
    [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
    [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin)
```

For directories:
```powershell
[Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory(
    '<absolute-path>',
    [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
    [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin)
```

Wrap calls via subagents as:
```powershell
powershell -c "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile('$path','OnlyErrorDialogs','SendToRecycleBin')"
```

## File Governance (MANDATORY)

### Desktop Contamination Prevention (MANDATORY)
NEVER write files to `C:\Users\koosh\Desktop\` or any user Desktop path.
The Desktop is a shared, user-visible surface — agents must not pollute it.

ALL agent output files MUST go to designated repo directories:
- Scripts (PS1, CMD, SH): `scripts/game/` or `scripts/game/desktop/` (for captures/automation)
- Screenshots/PNGs:        `docs/screenshots/` or `docs/screenshots/desktop/`
- Logs/TXT reports:        `docs/sessions/` or `docs/sessions/desktop/`
- Research markdown:       `docs/sessions/`
- Temp capture output:     use `$env:TEMP\DINOForge\` (already used by MCP server)

If a script currently writes to Desktop, update its output path to one of the above.
The MCP `game_screenshot` tool already writes to `$env:TEMP\DINOForge\` — use it.

### Script Lifecycle
Any temporary script created for a task MUST be:
1. Written to `scripts/game/` or `docs/scripts/` (not Desktop, not root)
2. Deleted (via Recycle Bin per File Deletion Protocol) when the task is complete
3. Never left as permanent artifacts unless promoted to a named slash command in `.claude/commands/`

## Build Commands

```bash
# Build
dotnet build src/DINOForge.sln

# Test
dotnet test src/DINOForge.sln --verbosity normal

# Lint
dotnet format src/DINOForge.sln --verify-no-changes

# Validate packs
dotnet run --project src/Tools/PackCompiler -- validate packs/

# Package a mod pack
dotnet run --project src/Tools/PackCompiler -- build packs/<pack-name>
```

## Repository Structure

```
DINOForge/
  src/
    Runtime/             # BepInEx plugin: bootstrap, ECS detection, debug overlay
      Bridge/            #   ECS bridge: component mapping, stat modifiers, entity queries
      HotReload/         #   Hot module replacement bridge
      UI/                #   In-game mod menu overlay (F10) and settings panel
    SDK/                 # Public mod API: registries, schemas, pack loaders
      Assets/            #   Asset service, addressables catalog, bundle info
      Dependencies/      #   Pack dependency resolver with cycle detection
      HotReload/         #   Pack file watcher for live reload
      Models/            #   Data models (units, factions, buildings, weapons, etc.)
      Registry/          #   Generic registry system with conflict detection
      Universe/          #   Universe Bible system for total conversions
      Validation/        #   Schema validation (NJsonSchema)
    Bridge/
      Protocol/          #   JSON-RPC message types and IGameBridge interface
      Client/            #   GameClient for out-of-process bridge communication
    Domains/
      Warfare/           #   Warfare domain plugin (factions, doctrines, combat, waves)
      Economy/           #   Economy domain plugin (rates, trade, balance)
      Scenario/          #   Scenario domain plugin (scripting, conditions, validation)
      UI/                #   UI/UX domain plugin (HUD injection, menu management)
    Tools/
      Cli/               #   dinoforge CLI (status, query, override, reload, watch, etc.)
      DinoforgeMcp/      #   FastMCP server for Claude Code integration (HTTP/SSE + stdio)
      PackCompiler/      #   Pack compiler: validate, build, package packs
      DumpTools/         #   Entity/component dump analysis (Spectre.Console)
      Installer/         #   PowerShell/Bash installer for BepInEx + DINOForge
    Tests/               #   Unit tests (xUnit + FluentAssertions)
      Integration/       #   Integration tests (Bridge, ContentLoader, end-to-end)
  packs/                 # Content packs and example mods
    example-balance/     #   Simple stat override example
    warfare-modern/      #   Modern warfare theme (West vs Classic Enemy)
    warfare-starwars/    #   Star Wars Clone Wars theme (Republic vs CIS)
    warfare-guerrilla/   #   Asymmetric warfare (Guerrilla faction)
    economy-balanced/    #   Economy balance pack
    scenario-tutorial/   #   Tutorial scenario pack
  schemas/               # Canonical JSON/YAML schema definitions (24 schemas)
  docs/                  # All project documentation
  manifests/             # System contracts, ownership maps, extension points
```

## Agent Governance

### Agents MUST:
- Work through manifests and registries
- Use generators/templates for new content
- Update docs/contracts when changing public surfaces
- Add tests for new public APIs
- Log failure modes explicitly
- Keep features pack-based when possible
- Run `dotnet test` before considering work complete

### Agents MUST NOT:
- **Handroll what a library already solves** - always search for existing packages first
- Patch runtime internals unless assigned runtime-layer work
- Invent new registry patterns casually
- Duplicate schemas
- Bypass validators
- Hardcode content IDs in engine glue
- Add undocumented extension points
- Skip tests
- Merge without compatibility checks

## Legal Move Classes (Agent Operations)

Agents should reduce all work to one of these forms:
- `create schema` - new data shape definition
- `extend registry` - add entries to existing registry
- `add content pack` - new pack with manifest
- `patch mapping` - update vanilla-to-mod mapping
- `write validator` - new validation rule
- `add test fixture` - new test case
- `add debug view` - new diagnostic surface
- `add migration` - version compatibility migration
- `add compatibility rule` - cross-pack conflict rule
- `add documentation manifest` - update docs

## Code Style

- C# 12+ with nullable reference types enabled
- `async/await` over raw Tasks
- XML doc comments on all public APIs
- Immutable data models preferred
- Registry pattern for all extensible domains
- No `var` for non-obvious types
- Meaningful names over comments

## Key Design Principles

1. **Wrap, don't handroll** - Use established libraries/tools and wrap them. Never build from scratch what a proven package already solves. Prefer thin wrappers and adapters over custom implementations. This is a vibecoding-only environment: maximize feature coverage and minimize risk by standing on existing shoulders.
2. **Framework before content** - platform first, themed mods second
3. **Declarative before imperative** - YAML/JSON manifests over C# patches
4. **Stable abstraction over unstable internals** - isolate ECS glue
5. **Agent-first repo design** - optimize for autonomous agent dev
6. **Observability is first-class** - logs, overlays, reports, validators
7. **Domain extensibility** - warfare is first plugin, not the only one
8. **Compatibility-aware packaging** - explicit deps, conflicts, versions
9. **Graceful degradation** - fail loudly with fallbacks

## Build vs Wrap Decision Rule

**ALWAYS prefer** (in order):
1. Direct use of an existing library/tool as-is
2. Thin wrapper / adapter around an existing library
3. Composition of multiple existing libraries
4. Modified fork of an existing library (last resort before handroll)

**ONLY handroll when**:
- No existing solution covers the need (e.g. DINO-specific ECS glue)
- Wrapping would be more complex than a simple implementation
- The scope is tiny and self-contained (< 50 lines)

**Rationale**: This is a fully agent-driven (vibecoding) environment. Agents produce more reliable output when integrating proven code than when generating novel implementations. Handrolled code has higher defect rates, lacks community testing, and creates maintenance burden that agents handle poorly without human review. Every handrolled component is a liability; every wrapped dependency is borrowed reliability.

### Concrete Examples

| Need | DO | DON'T |
|------|----|-------|
| YAML/JSON schema validation | Use JsonSchema.Net or NJsonSchema | Write custom validator |
| Pack dependency resolution | Use NuGet's resolver or Semver.NET | Write custom semver solver |
| Logging | Use Serilog or NLog via BepInEx | Write custom logger |
| CLI tooling | Use System.CommandLine or Spectre.Console | Write custom arg parser |
| Config management | Use BepInEx ConfigurationManager | Write custom config system |
| ECS introspection | Wrap Unity.Entities reflection APIs | Write custom reflection |
| File watching / hot reload | Use FileSystemWatcher | Write custom polling loop |
| Serialization | Use YamlDotNet + System.Text.Json | Write custom parsers |
| Diffing | Use DiffPlex or similar | Write custom diff engine |
| Testing | Use xUnit + FluentAssertions + Moq | Write custom test framework |

## Pack System

Every mod is a pack with a `pack.yaml` manifest and explicit metadata:
```yaml
id: example-pack
name: Example Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: DINOForge Agents
type: content  # content | balance | ruleset | total_conversion | utility
depends_on: []
conflicts_with: []
loads:
  factions: []
  units: []
  buildings: []
```

## Testing Philosophy

- **BDD-first**: Behavior specs define acceptance criteria before implementation
- **SDD**: Schema-driven development - schemas validate before runtime
- **TDD**: Unit tests for all public API surfaces
- Property-based tests for balance/combat model validation
- Pack validation tests (schema, references, completeness)
- Integration tests against mock ECS runtime

## Asset Pipeline Governance (v0.7.0+)

### Unified Asset Workflows in PackCompiler

All asset operations (3D models, textures, VFX, etc.) MUST go through **PackCompiler commands**, never fragmented tools:

```bash
# Asset import pipeline (unified, declarative)
dotnet run --project src/Tools/PackCompiler -- assets import <pack>
dotnet run --project src/Tools/PackCompiler -- assets validate <pack>
dotnet run --project src/Tools/PackCompiler -- assets optimize <pack>
dotnet run --project src/Tools/PackCompiler -- assets generate <pack>
dotnet run --project src/Tools/PackCompiler -- assets build <pack>

# Content sync
dotnet run --project src/Tools/PackCompiler -- sync download <pack> --phase <version>

# VFX generation (wraps VFXPrefabGenerator)
dotnet run --project src/Tools/PackCompiler -- vfx generate <pack>
```

### Asset Configuration: asset_pipeline.yaml

Every pack with assets MUST define `asset_pipeline.yaml` with:
- Model sources (GLB/FBX file paths)
- LOD targets (polycount percentages, screen thresholds)
- Material definitions (faction colors, emission)
- Addressables keys (for runtime loading)
- Definition updates (inject visual_asset references)

**Schema**: `schemas/asset_pipeline.schema.json` (validates all configs)

### Mandatory Asset Workflow Steps

Agents importing assets MUST follow this sequence **in order**:

1. **Define** — Create/update `asset_pipeline.yaml` in pack root
2. **Download** — `dotnet run -- sync download <pack>`
3. **Import** — `dotnet run -- assets import <pack>`
4. **Validate** — `dotnet run -- assets validate <pack>`
5. **Optimize** — `dotnet run -- assets optimize <pack>` (generates LOD)
6. **Generate** — `dotnet run -- assets generate <pack>` (creates prefabs)
7. **Verify** — `dotnet run -- assets build <pack>` (full pipeline + tests)
8. **Commit** — Git commit all artifacts + updated definitions

**Agents MUST NOT**:
- Manually edit game definitions when assets change
- Skip validation/optimization steps
- Create ad-hoc asset directories outside `packs/<pack>/assets/`
- Hardcode polycount targets or LOD percentages in C#
- Use separate/legacy tools (old download scripts, etc.)

### Asset Services (PackCompiler)

Core services in `src/Tools/PackCompiler/Services/`:

| Service | Responsibility | Tests |
|---------|-----------------|-------|
| `AssetImportService` | GLB/FBX → JSON (via AssimpNet) | 4+ tests |
| `AssetOptimizationService` | Mesh decimation → LOD variants | 4+ tests |
| `PrefabGenerationService` | JSON → .prefab (serialized) | 4+ tests |
| `AddressablesService` | YAML → catalog entries | 2+ tests |
| `DefinitionUpdateService` | Inject visual_asset into YAML | 2+ tests |

### Extension Pattern

Custom asset processors/validators can be registered:

```csharp
// In PackCompiler/Program.cs DI setup
public static IServiceCollection AddCustomAssetProcessors(this IServiceCollection services)
{
    services.AddAssetProcessor<CustomLightsaberGlowProcessor>();
    services.AddAssetValidator<StarWarsColorValidator>();
    services.AddAssetExporter<AlternativeFormatExporter>();
    return services;
}
```

Implementations MUST inherit from `IAssetProcessor`, `IAssetValidator`, `IAssetExporter` interfaces defined in PackCompiler.

### Testing Requirements for Assets

New asset features MUST include:

- **Unit tests** for each service (import, optimize, generate)
- **Integration tests** for full pipeline (download → build)
- **Regression tests** for known assets (v0.6.0 models, v0.7.0 critical)
- **Performance tests** (import < 5s/model, full pipeline < 5min for 9 models)
- **Schema validation tests** (asset_pipeline.yaml)

All asset tests live in `src/Tests/AssetPipelineTests.cs`

### Documentation Requirements

Agents changing asset workflows MUST update:

1. `ASSET_PIPELINE_CLI.md` — Command reference
2. `asset_pipeline.schema.json` — Config schema
3. `CLAUDE.md` (this section) — Governance changes
4. Inline XML docs in PackCompiler services
5. Test cases documenting new behavior

---

## Game Automation & Testing

### Game Install Path

```
G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\
  Diplomacy is Not an Option.exe   ← launch directly (not via Steam) for 2nd instance
  BepInEx\
    plugins\DINOForge.Runtime.dll  ← deployed by: dotnet build -p:DeployToGame=true
    dinoforge_packs\               ← deployed packs (auto-copied on build)
    dinoforge_debug.log            ← DINOForge Runtime log (read for swap/entity info)
    LogOutput.log                  ← BepInEx log (scene changes, plugin load errors)
```

### MCP Bridge (game automation)

The `dinoforge` MCP server (registered in MCP transport config) exposes:

| Tool | Purpose |
|------|---------|
| `game_launch` | Launch game exe; `hidden=True` uses CreateDesktop for isolated launch |
| `game_status` | Running state, entity count, loaded packs |
| `game_query_entities` | Query ECS entities by component type |
| `game_get_stat` | Read a stat value on an entity |
| `game_apply_override` | Apply a stat override |
| `game_reload_packs` | Hot-reload packs without restarting |
| `game_dump_state` | Trigger entity dump to file |
| `game_screenshot` | Capture game window screenshot |
| `game_verify_mod` | Verify mod is loaded and active |
| `game_wait_for_world` | Wait until ECS world is ready |
| `game_ui_automation` | Automate game UI interactions |
| `game_launch_test` | Launch TEST instance (optional `hidden=True` for invisible desktop) |
| `game_analyze_screen` | Capture screenshot + detect UI elements via OmniParser (health bars, unit portraits, buttons, faction indicators) |
| `game_input` | Inject keyboard/mouse input to game without requiring focus (Win32 SendInput) |
| `game_wait_and_screenshot` | Poll for visual change then capture screenshot (configurable timeout/interval) |
| `game_navigate_to` | Navigate to game state (main_menu/gameplay/pause_menu) via input sequences |

The runtime includes additional asset, catalog, logging, and pack-management tools beyond this excerpt (full tool surface is maintained in `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py`).

MCP runs from the FastMCP Python server in HTTP mode on `http://127.0.0.1:8765` and is managed by
`scripts/start-mcp.ps1`.

The repo-tracked CC hook can keep MCP alive automatically across sessions using `./scripts/start-mcp.ps1 -Action start -Detached`.

#### Display Isolation Strategy (TODO)

Future: Install a dedicated DINOForge virtual display driver (IDD/WDDM) to provide isolated headless game launches without requiring user's personal Parsec VDD. For now, `game_launch(hidden=True)` uses Win32 CreateDesktop (security isolation).

### Agent Slash Commands for Game Work

| Command | Role |
|---------|------|
| `/launch-game` | Launch 2nd game instance for testing |
| `/test-swap` | Full loop: build → deploy → launch → verify swaps |
| `/check-game` | Read debug log, report swap/entity status |
| `/game-test` | Automated test suite via MCP bridge |
| `/game-test-task <task>` | TITAN-inspired coverage-driven automated game test |
| `/game-coverage` | Show coverage memory statistics |
| `/entity-dump` | Parse ECS entity dump, analyze archetype counts |
| `/pack-deploy` | Validate + build + deploy pack + DLL |
| `/asset-create` | GLB/FBX → normalize → stylize → bundle → register |

### Critical ECS Facts (DO NOT FORGET)

- **ALL DINO entities are ECS Prefab entities** — every `EntityQuery` MUST use `EntityQueryOptions.IncludePrefab` or it returns 0 results
- `World.Systems` returns `NoAllocReadOnlyCollection` — index access only, no IEnumerable cast
- `MonoBehaviour.Update()` never runs — use `SystemBase.OnUpdate()`
- Asset swap Phase 2 (live entity swap) is the primary visual mechanism — Phase 1 (catalog disk patch) is optional/best-effort
- The Addressables catalog uses custom address keys, NOT Unity asset paths, for unit prefabs

### Deploying Fixes

```bash
# Deploy to MAIN instance (overwrites main save — safe for CI/CD):
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true

# Deploy to TEST instance (isolated, no save impact — use during active dev):
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true -p:GameInstallPath="G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"

# Check results after ~12s (600-frame delay):
tail -50 "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
```

### Test Instance (Second Concurrent Game)

Unity's native mutex (in UnityPlayer.dll) prevents launching a second instance from the same directory. To support concurrent test instances:

- **Test instance path**: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\`
- **Deploy to TEST**: `dotnet build -p:DeployToGame=true -p:GameInstallPath="...\Diplomacy is Not an Option_TEST"`
- **MCP tool**: `game_launch_test(hidden=True)` — launches TEST instance on hidden Win32 desktop (CreateDesktop), no visible window
- **Path config**: `.dino_test_instance_path` in repo root (auto-detected by MCP server)
- Both instances are completely independent; each has its own BepInEx installation

### AgilePlus PM Dashboard

**AgilePlus** (kooshapari/agileplus) is the spec-driven development engine used for DINOForge project management.

- **Repo**: `C:\Users\koosh\agileplus`
- **Launch**: `cd C:\Users\koosh\agileplus && bun run dev` (or `npm run dev`)
- **Purpose**: User stories, sprint tracking, spec management, roadmap visualization
- **Integration**: Specs in `docs/specs/` map to AgilePlus stories

---

## Asset Bundle Creation

### Unity Version Requirement

Asset bundles for DINO **must be built with Unity 2021.3.45f2**. Bundles from other versions will fail to load silently at runtime.

### Bundle → Pack Registration Flow

```
1. source.glb/fbx
2. → blender normalize_asset.py → normalized.glb (polycount reduction, material cleanup)
3. → blender stylize_asset.py → stylized.glb (faction palette applied)
4. → Unity 2021.3: import stylized.glb, build AssetBundle → <asset-id> (no extension)
5. → packs/<pack-id>/assets/bundles/<asset-id>   (bundle file)
6. → unit/building YAML: visual_asset: <asset-id>
7. → dotnet build -p:DeployToGame=true
```

### Bundle Naming Convention

- Bundle filename = `visual_asset` key = Addressable key used by AssetSwapRegistry
- Example: `sw-rep-clone-trooper` for file `packs/warfare-starwars/assets/bundles/sw-rep-clone-trooper`
- The asset name **inside** the bundle is the Unity prefab name (e.g. `sw-rep-clone-trooper.prefab`)
- AssetSwapSystem uses `bundle.LoadAllAssets()` fallback to handle name mismatches

### Faction Palettes (hardcoded in `AssetctlPipeline.BuildFactionPalette`)

| Faction | Primary | Secondary | Roughness | Metallic |
|---------|---------|-----------|-----------|---------|
| republic | `#F5F5F5` | `#1A3A6B` | 0.3 | 0.1 |
| cis | `#C8A87A` | `#5C3D1E` | 0.7 | 0.2 |
| neutral | `#888888` | — | 0.5 | 0.0 |
