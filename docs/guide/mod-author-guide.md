---
title: Getting Started for Mod Authors
description: Write your first DINOForge mod pack in 15 minutes
---

# Getting Started: Writing Your First Mod Pack

A step-by-step guide to creating and deploying your first mod pack for Diplomacy is Not an Option using DINOForge.

## What is DINOForge?

DINOForge is a **general-purpose mod platform and operating system** for Diplomacy is Not an Option. It is not a single mod — it's a framework that lets you create mod packs declaratively using YAML and JSON schemas, with full support for:

- **Pack Management**: Versioned, dependency-resolved mod packs with conflict detection
- **Declarative Content**: Define units, buildings, factions, weapons, and balance changes in YAML
- **Schema Validation**: Every pack is automatically validated against 24 canonical JSON/YAML schemas
- **Hot Reload**: Change pack content and reload without restarting the game
- **ECS Integration**: Direct integration with DINO's Unity ECS architecture — no Harmony patches needed
- **Multi-Domain Support**: Warfare, Economy, Scenario, and UI plugins for extensibility

## Prerequisites

Before you start, you'll need:

1. **Diplomacy is Not an Option** — Purchased and installed from Steam
2. **BepInEx 5 with Unity ECS Support** — The modified fork for ECS plugins (see [Getting Started](/guide/getting-started) for installation)
3. **DINOForge Runtime** — Deployed to `BepInEx/ecs_plugins/` (installed automatically or built from source)
4. **.NET 8 SDK** (optional) — Only if you want to build C# plugins in addition to YAML packs
5. **A text editor** — VS Code, Sublime, or any editor that supports YAML and JSON

## Install DINOForge

### For Users (Recommended)

1. Download the latest **DINOForge Installer** from [Releases](https://github.com/KooshaPari/Dino/releases)
2. Run `DINOForge.Installer.exe`
3. Select your DINO game installation directory
4. The installer will:
   - Install/update BepInEx with ECS support
   - Deploy DINOForge Runtime
   - Copy example packs to `BepInEx/dinoforge_packs/`
   - Verify the installation

5. Launch the game and press **F10** to open the mod menu

### For Developers

Clone the repository and build from source:

```bash
git clone https://github.com/KooshaPari/Dino.git
cd Dino
dotnet build src/DINOForge.sln
dotnet build src/Runtime/DINOForge.Runtime.csproj -p:DeployToGame=true
```

This copies `DINOForge.Runtime.dll` to your game's `BepInEx/ecs_plugins/` directory.

## Your First Pack: 15-Minute Tutorial

Let's create a simple **balance pack** that doubles archer damage.

### Step 1: Create the Pack Directory

```bash
# Navigate to DINOForge repo or your mods directory
cd packs
mkdir my-first-pack
cd my-first-pack
```

Structure:
```
my-first-pack/
  pack.yaml          # Pack metadata (required)
  units/             # Unit overrides
    archer.yaml
```

### Step 2: Write the Pack Manifest

Create `pack.yaml`:

```yaml
id: my-first-pack
name: My First Pack
version: 0.1.0
framework_version: ">=0.24.0 <1.0.0"
author: Your Name
type: balance
description: Doubles archer damage and adds +10 HP

depends_on: []
conflicts_with: []

loads:
  units:
    - archer_buffed
  buildings: []
```

**Key Fields:**
- `id`: Unique identifier (lowercase, hyphens OK, matches `^[a-z][a-z0-9-]*$`)
- `version`: Semantic versioning (`MAJOR.MINOR.PATCH`)
- `type`: One of `content`, `balance`, `ruleset`, `total_conversion`, `utility`
- `framework_version`: Which DINOForge versions support this pack (semver range)
- `loads`: What content this pack registers (units, buildings, factions, etc.)

### Step 3: Define a Unit Override

Create `units/archer_buffed.yaml`:

```yaml
id: archer_buffed
display_name: Elite Archer
unit_class: CoreLineInfantry
faction_id: vanilla
tier: 1

stats:
  hp: 110              # +10 from vanilla
  damage: 25           # doubled from vanilla ~12
  armor: 0
  range: 8
  speed: 3.5
  accuracy: 0.8
  fire_rate: 1.2
  morale: 85
  cost:
    food: 40
    wood: 10

weapon: BallisticLight

defense_tags:
  - Unarmored
  - Biological

behavior_tags:
  - HoldLine
  - AdvanceFire

vanilla_mapping: archer
```

**Notes:**
- `vanilla_mapping: archer` — This tells DINOForge to **override** the vanilla archer (not create a new unit)
- All `stats` fields are optional and default to reasonable values
- `weapon` is optional — defaults to the vanilla unit's weapon

### Step 4: Validate Your Pack

Before deploying, validate the pack using the PackCompiler CLI:

```bash
cd ../..  # Back to DINOForge repo root
dotnet run --project src/Tools/PackCompiler -- validate packs/my-first-pack
```

Expected output:
```
✓ Pack manifest schema valid
✓ Unit definitions valid
✓ 1 unit loaded (archer_buffed)
✓ No dependency conflicts
✓ Ready to deploy
```

If there are errors, the validator will tell you exactly what's wrong (e.g., missing required fields, invalid stat values).

### Step 5: Build the Pack

```bash
dotnet run --project src/Tools/PackCompiler -- build packs/my-first-pack
```

This outputs the compiled pack to `packs/my-first-pack/dist/my-first-pack-0.1.0.dinopack`.

### Step 6: Deploy to Game

Copy your pack to the BepInEx directory:

```powershell
# PowerShell on Windows
Copy-Item "packs/my-first-pack/dist/my-first-pack-0.1.0.dinopack" `
  -Destination "$env:STEAM_LIBRARY\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_packs\"
```

Or manually copy the `.dinopack` file to:
```
Game Root/BepInEx/dinoforge_packs/my-first-pack-0.1.0.dinopack
```

### Step 7: Launch and Test

1. **Launch the game** from Steam (or use `/launch-game` in Claude Code)
2. **Start a scenario** or custom game
3. **Build an archer** and check the health/damage in the unit info panel
4. **Verify the changes** — archer should have 110 HP and ~25 damage

To view active packs and their status:

```bash
dotnet run --project src/Tools/Cli -- status
```

## Pack Structure Reference

For a complete guide to pack organization, see [Creating Packs](/guide/creating-packs).

**Common directory layout:**

```
packs/my-pack/
  pack.yaml                          # Required
  units/                             # Unit definitions
    unit_id.yaml
  buildings/                         # Building definitions
    building_id.yaml
  weapons/                           # Weapon definitions
    weapon_id.yaml
  factions/                          # Faction definitions
    faction_id.yaml
  doctrines/                         # Warfare doctrines
    doctrine_id.yaml
  waves/                             # Wave templates
    wave_id.yaml
  assets/                            # Icons, audio, VFX
    icons/icon.png
    audio/sound.wav
  localization/                      # Text/strings
    en.yaml
    es.yaml
```

## Schema Reference

Every content type has a canonical JSON schema. View them in the repo:

```
schemas/
  pack-manifest.schema.json          # pack.yaml format
  unit.schema.json                   # unit definitions
  building.schema.json               # building definitions
  weapon.schema.json                 # weapon definitions
  faction.schema.json                # faction definitions
  doctrine.schema.json               # doctrine definitions
  wave.schema.json                   # wave definitions
```

Or view them online at [Schema Reference](/reference/schemas).

## Next Steps

### Expand Your Pack

- **Add buildings**: Create `buildings/` directory with `.yaml` files
- **Add factions**: Create a custom faction in `factions/faction_id.yaml`
- **Add doctrines**: Create `doctrines/` directory with doctrine definitions
- **Add waves**: Create `waves/` directory with wave templates

### Learn the Domains

- **[Warfare Domain](/warfare/overview)** — Factions, units, combat, balance
- **[Economy Domain](/concepts/registry-system)** — Resources, rates, trade
- **[Scenario Domain](/concepts/registry-system)** — Scripting, victory conditions, difficulty
- **[UI Domain](/concepts/registry-system)** — HUD elements, menus, themes

### Use the CLI Tools

```bash
# View all available CLI commands
dotnet run --project src/Tools/Cli -- --help

# Query game state (units, buildings, factions)
dotnet run --project src/Tools/Cli -- query units

# Apply stat overrides at runtime
dotnet run --project src/Tools/Cli -- override --unit archer_buffed --stat damage=30

# Hot-reload packs without restarting
dotnet run --project src/Tools/Cli -- reload
```

### Enable Hot Reload

While developing, use hot reload to test changes without restarting the game:

```bash
# Watch for pack changes and reload automatically
dotnet run --project src/Tools/Cli -- watch packs/my-first-pack
```

Changes to YAML files in your pack will automatically reload in-game. Press **F9** to apply the reload without restarting.

### Join the Community

- **GitHub Issues**: Report bugs or request features at [github.com/KooshaPari/Dino/issues](https://github.com/KooshaPari/Dino/issues)
- **Discussions**: Ask questions at [github.com/KooshaPari/Dino/discussions](https://github.com/KooshaPari/Dino/discussions)
- **Example Packs**: Study the example packs in `packs/` to see full implementations:
  - `example-balance/` — Simple stat overrides
  - `warfare-modern/` — Complete faction with units and buildings
  - `warfare-starwars/` — Total conversion with visual assets
  - `economy-balanced/` — Economy rebalance

## Troubleshooting

### "Pack validation failed"
- Check that all required fields are present in `pack.yaml` (id, name, version, etc.)
- Verify YAML syntax (no tabs, proper indentation)
- Ensure `vanilla_mapping` matches an actual vanilla unit ID

### "Pack did not load in game"
- Check `BepInEx/dinoforge_debug.log` for error messages
- Verify the `.dinopack` file was copied to `BepInEx/dinoforge_packs/`
- Ensure `pack.yaml` version matches the compiled filename
- Press F10 to open the mod menu and check if the pack appears

### "Changes not taking effect"
- Use hot reload: `dotnet run --project src/Tools/Cli -- reload`
- Or restart the game to reload all packs fresh
- Verify the YAML was saved correctly (no syntax errors)

### "Archer still has vanilla stats"
- Check `vanilla_mapping: archer` is present in your unit definition
- Verify the unit `id` matches what's in the `loads:` section of `pack.yaml`
- Confirm the pack is enabled (visible in F10 mod menu)

## References

- **[Pack System](/guide/creating-packs)** — Full pack format and types
- **[Schema Reference](/reference/schemas)** — All 24 canonical schemas
- **[CLI Reference](/reference/cli)** — Complete command reference
- **[DINO Game Notes](/reference/dino-game-notes)** — Game mechanics and balance data
- **[Architecture Overview](/concepts/architecture)** — How DINOForge works under the hood
