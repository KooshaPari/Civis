# Hello World Example Pack

A complete beginner-friendly example mod pack for DINOForge. This pack demonstrates core modding concepts and is safe to enable alongside other mods.

## What This Pack Does

This pack adds:

1. **Hello World Faction** — A simple custom faction with basic settings
2. **Swordsman Unit Override** — Boosted version of vanilla swordsmen
3. **UI Theme** — Custom colors visible in the main menu and F10 debug panel
4. **Settings Integration** — A greeting message customizable in the mod menu

It loads with `load_order: 50`, ensuring early loading without conflicts.

## Pack Structure

```
example-hello-world/
├── pack.yaml                         # Pack metadata and settings
├── factions/
│   └── hello-world-faction.yaml      # Faction definition
├── units/
│   └── hello-world-swordsman.yaml    # Unit definition
└── README.md                         # This file
```

## Key Files Explained

### `pack.yaml` — Your Pack's Identity Card

Every DINOForge pack must have a `pack.yaml` file at the root. This file defines:

| Field | Purpose | Example |
|-------|---------|---------|
| `id` | Unique identifier (kebab-case) | `example-hello-world` |
| `name` | Human-readable name | `Hello World Example` |
| `version` | Semantic version | `1.0.0` |
| `framework_version` | DINOForge compatibility | `>=0.24.0 <0.26.0` |
| `author` | Your name | `DINOForge` |
| `type` | Pack category | `content` / `balance` / `scenario` |
| `description` | What your pack does | Multi-line text |
| `loads` | Files to load | Lists of YAML files |

**Key insight**: The `loads` section tells DINOForge which files to parse. Files not listed in `loads` are ignored at startup.

Example from this pack:
```yaml
loads:
  factions:
    - factions/hello-world-faction.yaml
  units:
    - units/hello-world-swordsman.yaml
```

### `factions/hello-world-faction.yaml` — Adding a Faction

A faction is a playable side in the game. This file defines:

| Section | Purpose |
|---------|---------|
| `faction.id` | Identifier the game engine uses to reference this faction |
| `economy` | Multipliers for gathering, research, and building speed |
| `army` | Morale style, unit cap, training speed |
| `roster` | Maps abstract roles (cheap_infantry, hero_commander) to unit IDs |
| `buildings` | Maps building roles to building IDs |
| `visuals` | Colors and asset bundles for appearance |
| `audio` | Sound effect and music packages |

**Key insight**: The `roster` section connects this faction to units. When the game AI needs to train cheap infantry, it looks up `roster.cheap_infantry` to find the unit ID.

This pack's faction uses `hello_world_swordsman` for all roles—a minimal example. Real packs diversify units by role.

### `units/hello-world-swordsman.yaml` — Modifying Unit Stats

Units are the soldiers, vehicles, and structures that fight. This file defines:

| Field | Purpose | Example |
|-------|---------|---------|
| `id` | Unit identifier | `hello_world_swordsman` |
| `display_name` | Name shown in menus | `Hello World Swordsman` |
| `unit_class` | Type (infantry, vehicle, artillery) | `CoreLineInfantry` |
| `faction_id` | Which faction owns this unit | `hello_world_faction` |
| `stats` | HP, damage, cost, speed, etc. | Object with 8+ fields |
| `defense_tags` | Armor types (affects damage multipliers) | Array: `[Unarmored, InfantryArmor]` |
| `behavior_tags` | Combat tactics (affects AI) | Array: `[HoldLine, AdvanceFire]` |

**Key insight**: The `vanilla_mapping` field (commented out in this example) connects your custom unit to vanilla replacements. Uncomment to make this unit override the vanilla swordsman throughout the game.

## How to Use This Pack

### 1. Enable the Pack in Game

```bash
# Launch game with mod menu active (F10)
dinoforge deploy

# Or manually:
# 1. Launch Diplomacy is Not an Option
# 2. Go to Mods menu (F10)
# 3. Find "Hello World Example" in the pack list
# 4. Toggle it ON
# 5. Restart the game
```

### 2. Verify It Works

After enabling and restarting:

1. **Check the main menu** — Should show "HELLO WORLD" theme colors (cyan header)
2. **Check the F10 debug panel** — Look for "Hello World Example" with your greeting message
3. **Check F9 telemetry** — Look for greeting_message setting showing your custom text
4. **Check unit stats** — Swordsman should have 150 HP (vs vanilla ~100)

### 3. Customize It

To modify this pack:

1. Edit `factions/hello-world-faction.yaml` to change faction stats
2. Edit `units/hello-world-swordsman.yaml` to change unit stats
3. Uncomment `vanilla_mapping: line_infantry` to replace vanilla swordsmen
4. Change colors in `pack.yaml` under `ui_theme.primary_color`
5. Run `dinoforge deploy` to rebuild and test

### 4. Next Steps: Create Your Own Pack

Use the `dinoforge new` command to scaffold a new pack from this template:

```bash
# Create a new pack called "my-awesome-mod"
dinoforge new my-awesome-mod --author "Your Name" --type content --description "My first awesome mod"

# This creates: packs/my-awesome-mod/ with this same structure
# Edit the files to add your content
# Deploy with: dinoforge deploy
```

## Validation

Before deploying, validate your pack:

```bash
# Validate this pack
dinoforge verify-pack packs/example-hello-world

# Or validate all packs
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

Validation checks:
- Schema compliance (all required fields present)
- Reference integrity (factions exist, units belong to factions)
- No duplicate IDs across packs
- Semantic version correctness

## Common Tasks

### Add a New Unit to This Faction

1. Create `units/hello-world-archer.yaml` with similar structure
2. Add to `pack.yaml`'s `loads.units` list:
   ```yaml
   loads:
     units:
       - units/hello-world-swordsman.yaml
       - units/hello-world-archer.yaml
   ```
3. Update `factions/hello-world-faction.yaml` roster to use the archer where appropriate:
   ```yaml
   roster:
     recon: hello_world_archer  # Archers are good for scouting
     anti_armor: hello_world_archer
   ```

### Change Faction Colors

Edit `pack.yaml`:
```yaml
ui_theme:
  primary_color: "#FF0000"  # Red
  accent_color: "#FFFF00"   # Yellow
```

### Override Vanilla Swordsmen Globally

Uncomment in `units/hello-world-swordsman.yaml`:
```yaml
vanilla_mapping: line_infantry
```

This makes your boosted swordsman replace all vanilla swordsmen.

### Add a Custom Setting

Edit `pack.yaml` and add to the `settings` list:
```yaml
settings:
  - key: greeting_message
    display_name: "Greeting Message"
    type: string
    default_value: "Hello from DINOForge!"
    editable: true
```

Settings appear in the F10 mod menu for users to customize.

## Troubleshooting

### Pack doesn't load

1. Run `dinoforge verify-pack packs/example-hello-world`
2. Check `BepInEx/dinoforge_debug.log` for errors
3. Ensure all files listed in `pack.yaml`'s `loads` section exist

### Swordsman doesn't look different

1. Check F9 telemetry panel for unit stat values
2. Verify pack is enabled (F10 mod menu, should be green)
3. Start a new game (don't continue an old save)

### UI theme colors don't apply

1. Restart the game (not just reload)
2. Check that `ui_theme` is present in `pack.yaml`
3. Verify hex colors are valid: `#RRGGBB` format

## Further Reading

- **Creating More Units** — See `warfare-starwars/units/` for 11 unit examples
- **Custom Factions** — See `warfare-modern/factions/` for realistic modern warfare
- **Stat Overrides** — See `warfare-aerial/units/` for aerial combat units
- **Economy Packs** — See `economy-balanced/` for economic balance tweaks
- **Scenario Packs** — See scenario-tutorial for scripted game modes

## Questions?

See the DINOForge developer documentation at https://kooshapari.github.io/Dino
