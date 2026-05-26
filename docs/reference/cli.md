# CLI Reference

DINOForge includes command-line tools for pack validation, building, game interaction, and diagnostics.

## dinoforge CLI

The primary CLI for interacting with the game and managing packs. Located at `src/Tools/Cli/`.

### Commands

| Command | Description |
|---------|-------------|
| `status` | Check game connection and runtime status |
| `query` | Query entities by type, component, or tag |
| `override` | Apply stat overrides from a pack |
| `reload` | Reload all packs in the running game |
| `watch` | Watch pack directory for changes and auto-reload |
| `dump` | Dump game state (entities, components, systems) |
| `resources` | Get current resource levels |
| `screenshot` | Capture a game screenshot |
| `verify` | Verify a mod pack against the running game |
| `component-map` | Get the component mapping table |

### Usage

```bash
# Check game status
dotnet run --project src/Tools/Cli -- status

# Query entities
dotnet run --project src/Tools/Cli -- query --type unit

# Apply an override
dotnet run --project src/Tools/Cli -- override --pack my-mod

# Watch for pack changes and auto-reload
dotnet run --project src/Tools/Cli -- watch packs/my-mod

# Reload all packs
dotnet run --project src/Tools/Cli -- reload

# Dump game state
dotnet run --project src/Tools/Cli -- dump

# Get resource levels
dotnet run --project src/Tools/Cli -- resources

# Verify a pack
dotnet run --project src/Tools/Cli -- verify packs/my-mod
```

---

## MCP Server

The MCP (Model Context Protocol) server enables Claude Code to interact with the running game. Located at `src/Tools/McpServer/`.

### Start

```bash
dotnet run --project src/Tools/McpServer
```

### Available Tools (17 Total)

| Tool | Description |
|------|-------------|
| `GameStatusTool` | Check game connection status |
| `GameLaunchTool` | Launch the game process |
| `GameWaitForWorldTool` | Wait for ECS World to be ready |
| `GameQueryEntitiesTool` | Query entities by type or component |
| `GameGetComponentMapTool` | Get component mapping table |
| `GameGetStatTool` | Get stat values for entities |
| `GameApplyOverrideTool` | Apply stat overrides to entities |
| `GameReloadPacksTool` | Reload all mod packs |
| `GameGetResourcesTool` | Get current resource levels |
| `GameDumpStateTool` | Dump full game state |
| `GameScreenshotTool` | Capture game screenshot |
| `GameVerifyModTool` | Verify a mod against game state |
| `GameAnalyzeScreenTool` | Capture screenshot + detect UI elements via OmniParser (health bars, unit portraits, buttons, faction indicators) |
| `GameInputTool` | Inject keyboard/mouse input to game without requiring focus (Win32 SendInput) |
| `GameWaitAndScreenshotTool` | Poll for visual change then capture screenshot (configurable timeout/interval) |
| `GameNavigateTool` | Navigate to game state (main_menu/gameplay/pause_menu) via input sequences |
| `GameUIAutomationTool` | Automate game UI interactions (menus, dialogs, forms) |

---

## PackCompiler

Pack validation and build tool. Located at `src/Tools/PackCompiler/`.

### Validate

Check packs against schemas and dependency rules:

```bash
# Validate a single pack
dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack

# Validate all packs in directory
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

**What it checks:**
- Manifest schema conformance
- All YAML/JSON files match their respective schemas
- Required fields present with correct types
- ID format and uniqueness
- Semantic version format
- Dependency graph resolution (no cycles, no missing deps)
- Asset reference integrity
- ECS registration conflict detection
- Cross-pack compatibility

**Exit codes:**
| Code | Meaning |
|------|---------|
| 0 | All validations passed |
| 1 | Validation errors found |
| 2 | Fatal error (missing files, invalid arguments) |

### Build

Compile a validated pack into a distributable artifact:

```bash
dotnet run --project src/Tools/PackCompiler -- build packs/my-pack
```

**Build pipeline:**
1. Runs full validation (same as `validate`)
2. Resolves all cross-file references
3. Checks for missing assets
4. Checks for circular dependencies
5. Produces pack artifact
6. Emits compatibility metadata

### Assets

Manage pack assets:

```bash
# List assets referenced by a pack
dotnet run --project src/Tools/PackCompiler -- assets list packs/my-pack

# Check for missing asset references
dotnet run --project src/Tools/PackCompiler -- assets check packs/my-pack
```

## Asset Intake CLI (Pre-Impl)

Planned commands for intake and normalization are documented in:

- `docs/asset-intake/assetctl-prd.md`
- `docs/adr/ADR-010-asset-intake-pipeline.md`

```bash
assetctl search "star wars b1 battle droid" --source sketchfab --limit 10
assetctl intake sketchfab:abc123
assetctl normalize sw_b1_droid_sketchfab_001
assetctl validate sw_b1_droid_sketchfab_001 --strict
assetctl register sw_b1_droid_sketchfab_001
```

---

## DumpTools

Offline analysis of entity/component dumps from the Runtime. Located at `src/Tools/DumpTools/`.

```bash
# Run the dump analyzer
dotnet run --project src/Tools/DumpTools
```

DumpTools uses Spectre.Console for interactive terminal UI. It can:
- Parse entity dumps from the Runtime's Entity Dumper
- Analyze component distributions
- Compare dumps across game versions
- Generate mapping tables for the SDK

---

## Installer

Install BepInEx and DINOForge into a DINO game directory. Located at `src/Tools/Installer/`.

```powershell
# Windows (PowerShell)
.\src\Tools\Installer\Install-DINOForge.ps1

# Linux/macOS (Bash)
./src/Tools/Installer/install.sh
```

The installer:
- Detects the DINO game directory
- Downloads and installs BepInEx 5.4.23.5 (modified ECS build)
- Copies DINOForge Runtime DLL to `BepInEx/ecs_plugins/`
- Creates the `dinoforge_packs/` directory

---

## Build Commands

Standard .NET build and test commands:

```bash
# Build everything
dotnet build src/DINOForge.sln

# Run tests
dotnet test src/DINOForge.sln --verbosity normal

# Check formatting
dotnet format src/DINOForge.sln --verify-no-changes

# Fix formatting
dotnet format src/DINOForge.sln
```
