# Getting Started

This guide walks you through setting up DINOForge for development or mod authoring.

## Prerequisites

- [.NET 8 SDK](https://dotnet.microsoft.com/download) or later
- [Node.js 20+](https://nodejs.org/) (for docs only)
- Diplomacy is Not an Option (Steam)
- Git

## Install BepInEx and DINOForge

DINOForge requires the **modified BepInEx 5 with Unity ECS support** from Nexus Mods. Standard BepInEx from GitHub will **not** work — this game requires a custom-forked `winhttp.dll` (Doorstop) that enables loading multiple assemblies from the `ecs_plugins/` directory, which is how Unity ECS plugins are bootstrapped. The fork was created by devopsdinosaur specifically for this game.

### One required boot.config fix

After installing BepInEx, edit `&lt;GameRoot&gt;/Diplomacy is Not an Option_Data/boot.config` and ensure it contains:
```
single-instance=0
```
This disables Unity's native single-instance enforcement (the empty `single-instance=` value it ships with causes "Another instance is running" fatal errors on relaunch).

### Option A: Automated Installer (Recommended for Users)

DINOForge includes a GUI installer that handles BepInEx and DINOForge setup automatically.

1. Download the latest DINOForge installer from [Releases](https://github.com/KooshaPari/Dino/releases)
2. Run `DINOForge.Installer.exe`
3. Select your DINO game installation directory
4. The installer will:
   - Download and install the modified BepInEx 5 (ECS fork) from Nexus Mods
   - Apply the `boot.config` fix
   - Install DINOForge Runtime and example packs
   - Verify the installation
5. Launch the game and verify mods load (F10 to open mod menu)

### Option B: Manual Installation (For Developers)

1. Download **BepInEx 5 with Unity ECS Support** from [Nexus Mods](https://www.nexusmods.com/diplomacyisnotanoption/mods/1) (version `5.4.23.2-ecs`)
2. Extract into your DINO game directory (where the `.exe` lives)
3. Apply the boot.config fix (see above)
4. Verify the folder structure:

```
GameRoot/
  winhttp.dll              # Modified Doorstop (ECS multi-assembly fork)
  doorstop_config.ini
  Diplomacy is Not an Option_Data/
    boot.config            # Must contain: single-instance=0
  BepInEx/
    ecs_plugins/           # DINOForge.Runtime.dll goes here
    plugins/               # Standard mono plugins (unused by DINOForge)
    config/
```

5. Launch the game once to let BepInEx initialize, then close it.
6. Copy the built `DINOForge.Runtime.dll` to `BepInEx/ecs_plugins/`

## Clone and Build

```bash
git clone https://github.com/KooshaPari/Dino.git
cd Dino
```

Build the solution:

```bash
dotnet build src/DINOForge.sln
```

Run tests to verify everything works:

```bash
dotnet test src/DINOForge.sln --verbosity normal
```

## Project Structure

```
DINOForge/
  src/
    Runtime/           # BepInEx plugin — bootstrap, ECS detection, hooks
      Bridge/          #   ECS bridge: component mapping, stat modifiers
      HotReload/       #   Hot reload bridge
      UI/              #   Mod menu (F10), settings panel
    SDK/               # Public mod API — registries, schemas, pack loaders
      Assets/          #   Asset service, addressables catalog
      Dependencies/    #   Dependency resolver
      HotReload/       #   Pack file watcher
      Models/          #   Content data models
      Registry/        #   Generic registry with conflict detection
      Universe/        #   Universe Bible system
      Validation/      #   Schema validation (NJsonSchema)
    Bridge/
      Protocol/        #   JSON-RPC message types, IGameBridge
      Client/          #   Out-of-process game client
    Domains/
      Warfare/         #   Factions, doctrines, combat, waves
      Economy/         #   Rates, trade, balance
      Scenario/        #   Scripting, conditions
      UI/              #   HUD injection, menus
    Tools/
      Cli/             #   dinoforge CLI
      McpServer/       #   MCP server for Claude Code
      PackCompiler/    #   Pack compiler (validate, build)
      DumpTools/       #   Entity dump analysis (Spectre.Console)
      Installer/       #   BepInEx + DINOForge installer
    Tests/             # Unit tests (xUnit + FluentAssertions)
      Integration/     # Integration tests
  packs/               # Content packs (6 example packs)
  schemas/             # JSON/YAML schema definitions (17 schemas)
  docs/                # This documentation site
```

## Load a Pack

Once built, the Runtime DLL goes into `BepInEx/ecs_plugins/`:

```
BepInEx/ecs_plugins/DINOForge.Runtime.dll
```

Content packs live in a `packs/` directory relative to the game root. The runtime discovers and loads them at boot.

## Next Steps

- [Quick Start](/guide/quick-start) — Create your first pack in 5 minutes
- [Creating Packs](/guide/creating-packs) — Full pack authoring guide
- [Architecture](/concepts/architecture) — Understand the layered design
