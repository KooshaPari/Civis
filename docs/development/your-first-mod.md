# Your First DINOForge Mod — Complete Tutorial

Welcome to DINOForge! This tutorial walks you through creating and testing your first mod pack in 15 minutes.

## Prerequisites

- DINOForge installed (see [INSTALLATION.md](../INSTALLATION.md))
- Diplomacy is Not an Option (DINO) installed and working
- Text editor (VS Code recommended)
- Command line familiarity (PowerShell or Bash)

## What You'll Build

A simple mod pack called `my-first-mod` that:

1. Adds a custom faction with boosted unit stats
2. Overrides vanilla swordsmen with a more powerful version
3. Applies custom UI theme colors
4. Configurable greeting message in the mod menu

**Time**: ~15 minutes
**Difficulty**: Beginner
**Result**: A working, deployable mod pack

## Step 1: Scaffold a New Pack (2 min)

Open your terminal and navigate to the DINOForge repository:

```bash
cd C:\Users\YourName\Dino
```

Create a new pack using the built-in scaffolder:

```bash
dotnet run --project src/Tools/Cli -- new my-first-mod --author "Your Name" --type content --description "My first mod pack"
```

You'll see:

```
Created pack 'my-first-mod' at packs/my-first-mod/

Next steps:
  1. Edit packs/my-first-mod/pack.yaml to customize metadata
  2. Add units/buildings/factions YAML in subdirectories
  3. Validate: dinoforge verify-pack packs/my-first-mod
  4. Deploy: dotnet build -p:DeployToGame=true
```

Verify the pack was created:

```bash
ls packs/my-first-mod/
```

You should see:
```
pack.yaml
README.md
units/
```

## Step 2: Understand pack.yaml (2 min)

Open `packs/my-first-mod/pack.yaml`:

```yaml
id: my-first-mod
name: My First Mod
version: 0.1.0
framework_version: ">=0.24.0 <0.26.0"
author: Your Name
type: content
description: My first mod pack

depends_on: []
conflicts_with: []

loads:
  units: []
```

This file is your pack's "identity card". Let's customize it:

```yaml
id: my-first-mod
name: My First Awesome Mod
version: 1.0.0
framework_version: ">=0.24.0 <0.26.0"
author: Your Name
type: content
description: |
  My very first DINOForge mod pack!
  
  This pack demonstrates:
  - Custom faction
  - Boosted units
  - UI theme colors

depends_on: []
conflicts_with: []
load_order: 50

# NEW: Add ui_theme section
ui_theme:
  title: "MY AWESOME MOD"
  subtitle: "Welcome to modding!"
  primary_color: "#FF6B6B"
  accent_color: "#4ECDC4"
  text_color: "#FFFFFF"

# NEW: Add settings
settings:
  - key: greeting
    display_name: "Greeting Message"
    type: string
    default_value: "Welcome to my first mod!"
    editable: true

loads:
  factions:
    - factions/my-faction.yaml      # NEW
  units:
    - units/my-swordsman.yaml       # NEW
```

**Key changes**:
- Updated `name`, `description`, `author` to personalize it
- Added `ui_theme` with your custom colors (visible in main menu)
- Added `settings` for a user-customizable greeting
- Updated `loads` to reference files we'll create next

Save the file.

## Step 3: Create a Custom Faction (3 min)

Create the factions directory:

```bash
mkdir -p packs/my-first-mod/factions
```

Create `packs/my-first-mod/factions/my-faction.yaml`:

```yaml
faction:
  id: my_faction
  theme: custom
  archetype: order
  doctrine: elite_discipline
  display_name: My Faction

economy:
  gather_bonus: 1.1        # 10% faster resource gathering
  upkeep_modifier: 0.9     # 10% cheaper unit upkeep
  research_speed: 1.0
  build_speed: 1.0

army:
  morale_style: disciplined
  unit_cap_modifier: 1.1   # 10% larger army limit
  elite_cost_modifier: 0.95 # 5% cheaper elite units
  spawn_rate_modifier: 1.0

roster:
  # Map all roles to our custom swordsman
  cheap_infantry: my_swordsman
  line_infantry: my_swordsman
  elite_infantry: my_swordsman
  anti_armor: my_swordsman
  support_weapon: my_swordsman
  recon: my_swordsman
  light_vehicle: my_swordsman
  heavy_vehicle: my_swordsman
  artillery: my_swordsman
  hero_commander: my_swordsman
  spike_unit: my_swordsman

buildings:
  # Use vanilla buildings (no custom building defs yet)
  barracks: vanilla_barracks
  workshop: vanilla_barracks
  artillery_foundry: vanilla_barracks
  tower_mg: vanilla_barracks
  heavy_defense: vanilla_barracks
  command_center: vanilla_barracks
  economy_primary: vanilla_barracks
  economy_secondary: vanilla_barracks
  research_facility: vanilla_barracks
  wall_segment: vanilla_barracks

visuals:
  primary_color: "#FF6B6B"
  accent_color: "#4ECDC4"
  projectile_pack: "vanilla"
  ui_skin: "default"

audio:
  weapon_pack: "vanilla"
  structure_pack: "vanilla"
  ambient_pack: "vanilla"
  music_pack: "vanilla"
```

**What this does**:
- Defines a faction with ID `my_faction`
- Boosts gathering speed (+10%) and reduces upkeep (-10%)
- Maps all unit roles to `my_swordsman` (which we'll create next)
- Uses vanilla buildings (we're not modifying buildings today)
- Sets theme colors matching your UI theme

## Step 4: Create a Custom Unit (3 min)

Create the units directory:

```bash
mkdir -p packs/my-first-mod/units
```

Create `packs/my-first-mod/units/my-swordsman.yaml`:

```yaml
id: my_swordsman
display_name: My Awesome Swordsman
description: A boosted swordsman with improved stats

unit_class: CoreLineInfantry
faction_id: my_faction
tier: 1

stats:
  hp: 140                # Vanilla: ~100. We boost to 140.
  damage: 14             # Vanilla: ~12. We boost to 14.
  armor: 2
  range: 1               # Melee
  speed: 3.5
  cost:
    resource_1: 30       # Food (vanilla: ~40)
    resource_2: 0
    resource_3: 0
    resource_4: 0
    population: 1
  accuracy: 0.85
  fire_rate: 1.0
  morale: 100

defense_tags:
  - Unarmored

behavior_tags:
  - HoldLine
  - AdvanceFire
  - Charge

# Optional: uncomment to replace vanilla swordsmen throughout the game
# vanilla_mapping: line_infantry
```

**What this does**:
- Defines a unit with ID `my_swordsman`
- Assigns it to your custom faction (`my_faction`)
- Boosts HP (+40%) and damage (+16.7%) vs vanilla swordsmen
- Reduces cost (30 food vs 40) to make it more attractive
- Sets behavior tags (HoldLine, AdvanceFire, Charge) for combat AI

Save the file.

## Step 5: Validate Your Pack (2 min)

Before deploying, validate that your YAML files are correct:

```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/my-first-mod
```

You should see:

```
✓ Validating packs/my-first-mod...
✓ pack.yaml: VALID
✓ factions/my-faction.yaml: VALID
✓ units/my-swordsman.yaml: VALID
✓ All content references resolved successfully

Summary: 3 files validated, 0 errors, 0 warnings
```

If you see errors, fix the YAML (common issues: missing fields, wrong indentation, typos in IDs).

## Step 6: Deploy to Game (1 min)

Build and deploy your pack:

```bash
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
```

This will:
1. Compile your C# code
2. Validate all packs (including yours)
3. Copy your pack to the game's mod directory
4. Copy the DINOForge DLL to BepInEx

Wait for the build to complete (~30 seconds).

## Step 7: Test in Game (2 min)

### Launch the game:

```bash
# Windows
Start-Process -FilePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"

# Or from Steam (same result)
```

Wait for the game to load (should take 20-30 seconds).

### Enable your pack:

1. Click the **Mods** button in the main menu (or press **F10**)
2. Look for "My First Awesome Mod" in the pack list
3. Toggle it **ON** (checkbox should be green)
4. You should see a confirmation message
5. **Restart the game** (the message will tell you to do this)

### Verify it works:

After restart:

1. **Check the main menu** — You should see "MY AWESOME MOD" title in your custom colors (#FF6B6B red)
2. **Open Mods menu (F10)** — Should show your pack as enabled with your greeting message
3. **Check F9 telemetry panel** — Look for "My Awesome Mod" section with your custom setting
4. **Start a new game** — Play a level with your faction to verify unit stats

## Step 8: Test Vanilla Mapping (Optional, 3 min)

To make your swordsmen replace **all** vanilla swordsmen:

1. Edit `packs/my-first-mod/units/my-swordsman.yaml`
2. Uncomment the `vanilla_mapping` line:

```yaml
vanilla_mapping: line_infantry
```

3. Rebuild and redeploy:

```bash
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
```

4. Restart the game
5. Start a new game — all swordsmen (including vanilla factions') should have your boosted stats

## Congratulations!

You've successfully created and deployed your first DINOForge mod! 🎉

You've learned:

- How to scaffold a pack with `dinoforge new`
- How to define a faction with custom economy and morale settings
- How to create a custom unit with boosted stats
- How to apply UI theme colors
- How to validate and deploy your pack
- How to test in the game and verify changes
- How to use `vanilla_mapping` to override vanilla content

## Next Steps

### Add More Units

Copy `units/my-swordsman.yaml` and customize it:
- Change `id` to `my_archer`
- Change `display_name` to `My Awesome Archer`
- Change stats (lower cost, longer range, lower damage)
- Update faction roster to use archer for `recon` and `anti_armor` roles

### Add Custom Buildings

Create `buildings/my-barracks.yaml` similar to units, then update faction roster.

### Customize Economy

Edit `factions/my-faction.yaml` to tweak:
- `economy.gather_bonus` (faster/slower resource gathering)
- `economy.upkeep_modifier` (cheaper/more expensive units)
- `army.unit_cap_modifier` (allow bigger/smaller armies)

### Add Scenarios

Create a `scenarios/` directory with custom mission definitions. See `packs/scenario-tutorial/` for examples.

### Publish Your Pack

Once you're happy:

1. Commit to git: `git add packs/my-first-mod && git commit -m "feat: add my first mod"`
2. Test thoroughly with multiple game sessions
3. Write comprehensive pack.yaml description
4. Consider sharing on Thunderstore or GitHub!

## Troubleshooting

### "Pack didn't load"

Check the debug log:

```bash
cat "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" | tail -50
```

Look for lines with `ERROR` or `Exception`.

### "Swordsman still has vanilla stats"

1. Verify pack is enabled (F10 menu should show green checkbox)
2. Make sure you restarted the game (F5 reload is not enough)
3. Check that `vanilla_mapping` is NOT set (it overrides all vanilla swordsmen, which you probably don't want yet)
4. Look at F9 telemetry panel to see what stats are actually loaded

### "UI colors don't appear"

1. Verify `ui_theme` block is in `pack.yaml`
2. Check hex color format: must be `#RRGGBB` (e.g. `#FF6B6B`)
3. Restart the game (not just reload)
4. Check the F10 mod menu — your theme colors should appear in the pack details pane

### "Settings don't appear in F10 menu"

1. Verify `settings` block is in `pack.yaml`
2. Check that `editable: true` is set
3. Verify `type` is a valid type: `string`, `number`, `boolean`, `enum`
4. Restart the game and open F10 again

## Further Reading

- **[Packs README](../../packs/README.md)** — Overview of all example packs
- **[example-hello-world](../../packs/example-hello-world/README.md)** — Minimal reference implementation
- **[warfare-starwars](../../packs/warfare-starwars/README.md)** — Complex example with 11 units per faction
- **[economy-balanced](../../packs/economy-balanced/README.md)** — Economy pack with resource multipliers
- **[PackCompiler CLI Reference](../../docs/PACKCOMPILER_CLI.md)** — Full pack validation and build commands

Happy modding! 🚀
