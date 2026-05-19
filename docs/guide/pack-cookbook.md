---
title: Pack Cookbook
description: Practical recipes for common DINOForge mod patterns
---

# Pack Cookbook — Recipes for Common Patterns

Copy-pasteable recipes for the most common mod patterns in DINOForge. Each recipe shows the goal, minimal YAML structure, and how to validate.

---

## Recipe 1: Override a Single Unit Stat

**Goal**: Buff or nerf a vanilla unit (e.g., double archer damage) without touching anything else.

**Pack Structure**:
```
packs/archer-buff/
  pack.yaml
  units/
    archer_buffed.yaml
```

**pack.yaml**:
```yaml
id: archer-buff
name: Archer Buff Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: You
type: balance
description: Doubles archer damage

depends_on: []
conflicts_with: []

loads:
  units:
    - archer_buffed

overrides:
  units:
    - vanilla_archer
```

**units/archer_buffed.yaml**:
```yaml
id: archer_buffed
vanilla_mapping: archer
display_name: Elite Archer

stats:
  damage: 25        # doubled from ~12
```

**Validate**: `dotnet run --project src/Tools/PackCompiler -- validate packs/archer-buff`

---

## Recipe 2: Add a New Unit to a Vanilla Faction

**Goal**: Create a brand-new unit type that uses vanilla faction colors/theme.

**pack.yaml**:
```yaml
id: new-unit-pack
name: New Unit Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: You
type: content

loads:
  units:
    - custom_crossbowman
```

**units/custom_crossbowman.yaml**:
```yaml
id: custom_crossbowman
display_name: Crossbowman
unit_class: CoreLineInfantry
faction_id: vanilla
tier: 2

stats:
  hp: 120
  damage: 18
  armor: 0
  range: 10
  speed: 3.0
  accuracy: 0.75
  fire_rate: 0.8
  morale: 85
  cost:
    food: 50
    wood: 20

weapon: BallisticLight
defense_tags:
  - Unarmored
behavior_tags:
  - HoldLine
```

**Key Difference from Recipe 1**: No `vanilla_mapping` (it's new, not an override). Set `faction_id: vanilla` to inherit vanilla colors.

---

## Recipe 3: Create a Total Conversion (New Faction)

**Goal**: Add a completely new faction with unique units, buildings, and theme.

**pack.yaml**:
```yaml
id: warfare-scifi
name: Sci-Fi Total Conversion
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: You
type: total_conversion

depends_on:
  - dino-warfare-domain

loads:
  factions:
    - scifi_alliance
    - scifi_collective
  units:
    - alliance_trooper
    - alliance_tank
    - collective_drone
    - collective_carrier
  buildings:
    - alliance_barracks
    - collective_hive
```

**factions/scifi_alliance.yaml**:
```yaml
id: scifi_alliance
display_name: Galactic Alliance
color_primary: "1E90FF"    # Dodger blue
color_secondary: "FFD700"  # Gold
description: High-tech military faction
vanilla_mapping: null      # not mapped to vanilla
```

**units/alliance_trooper.yaml**:
```yaml
id: alliance_trooper
display_name: Alliance Trooper
unit_class: CoreLineInfantry
faction_id: scifi_alliance
tier: 1

stats:
  hp: 100
  damage: 15
  range: 6
  speed: 4.0
  cost:
    food: 30
    wood: 0

weapon: BlasterRifle
```

**Validate & Build**: `dotnet run --project src/Tools/PackCompiler -- build packs/warfare-scifi`

---

## Recipe 4: Add a Custom HUD Element

**Goal**: Display a custom UI widget (e.g., morale meter) on the game HUD.

**pack.yaml**:
```yaml
id: ui-morale-meter
name: Morale Meter HUD
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: You
type: utility

loads:
  hud_elements:
    - morale_indicator
```

**hud_elements/morale_indicator.yaml**:
```yaml
id: morale_indicator
display_name: Morale Indicator
hud_anchor: top_left
hud_offset: [10, 10]
width: 200
height: 50
background_color: "1A1A1A"
border_color: "00FF00"
border_width: 2

content:
  type: stat_bar
  stat_key: average_army_morale
  bar_color: "00FF00"
  warning_threshold: 0.5
  critical_threshold: 0.2

visibility_rules:
  show_in_gameplay: true
  show_in_pause_menu: true
```

**Validation**: `dotnet run --project src/Tools/PackCompiler -- validate packs/ui-morale-meter`

---

## Recipe 5: Define a Scenario with Victory Conditions

**Goal**: Create a custom game scenario with scripted events and win/lose conditions.

**pack.yaml**:
```yaml
id: scenario-defend-castle
name: Defend the Castle Scenario
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: You
type: ruleset

loads:
  scenarios:
    - defend_castle
```

**scenarios/defend_castle.yaml**:
```yaml
id: defend_castle
display_name: Defend the Castle
description: Hold the central castle for 30 waves
difficulty: hard

initial_state:
  player_faction: vanilla
  starting_resources:
    food: 500
    wood: 300
  player_spawn_building: castle

victory_conditions:
  - type: survive_waves
    waves_to_survive: 30
    reward_gold: 5000

defeat_conditions:
  - type: building_destroyed
    building_id: castle
    failure_message: "The castle has fallen!"
  - type: morale_depleted
    faction_id: vanilla
    failure_message: "Your army has lost morale!"

scripted_events:
  - at_wave: 10
    action: spawn_reinforcements
    amount: 50
    unit_type: archer
```

---

## Recipe 6: Hot-Reload Your Pack During Testing

**Goal**: Iterate on a pack without restarting the game.

**Prerequisites**: Game must be running. Use `/dev-harness` to start MCP server.

**Workflow**:

1. Edit your pack YAML (e.g., `units/knight.yaml`)
2. Revalidate: `dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack`
3. Rebuild the pack: `dotnet run --project src/Tools/PackCompiler -- build packs/my-pack`
4. In-game: Press **F10** → **Reload Packs**

Or via MCP:
```bash
# In a separate terminal with MCP running:
dinoforge reload-packs --pack-id my-pack
```

**Tip**: Check `BepInEx/dinoforge_debug.log` for validation errors:
```bash
tail -20 "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
```

---

## Recipe 7: Cross-Pack Dependencies (Modular Mod Packs)

**Goal**: Create a pack that depends on another pack (e.g., theme pack depends on balance foundation).

**foundation-pack/pack.yaml**:
```yaml
id: balance-foundation
name: Balance Foundation
version: 1.0.0
type: balance

loads:
  units:
    - balanced_archer
    - balanced_knight
```

**theme-pack/pack.yaml**:
```yaml
id: fantasy-theme
name: Fantasy Theme Pack
version: 1.0.0
type: content

depends_on:
  - balance-foundation     # require foundation to load first

loads:
  units:
    - elf_archer           # extends balanced_archer with visuals
    - dwarf_knight         # extends balanced_knight with visuals
```

**theme-pack/units/elf_archer.yaml**:
```yaml
id: elf_archer
vanilla_mapping: archer
display_name: Elf Archer

# Inherits stats from balance-foundation's balanced_archer via dependency chain
# Add visual overrides:
visual_asset: fantasy-elf-archer-bundle
```

**Load Order**: Foundation loads first (lower ID), then theme pack uses its definitions.

---

## Recipe 8: Asset Bundle References for Custom Visuals

**Goal**: Reference a custom 3D model bundle for a unit or building.

**Prerequisites**: Asset bundle exists at `packs/my-pack/assets/bundles/my-unit-prefab`.

**units/custom_warrior.yaml**:
```yaml
id: custom_warrior
display_name: Custom Warrior
unit_class: HeavyInfantry
faction_id: vanilla

stats:
  hp: 150
  damage: 20
  armor: 5
  range: 2
  speed: 2.5
  cost:
    food: 80
    wood: 40

# Reference the asset bundle
visual_asset: custom-warrior-model

# If the bundle has LOD variants:
visual_lods:
  high: custom-warrior-model-lod0
  medium: custom-warrior-model-lod1
  low: custom-warrior-model-lod2

weapon: Sword
```

**Building example** (`buildings/custom_tower.yaml`):
```yaml
id: custom_tower
display_name: Custom Tower
building_class: DefensiveStructure

stats:
  hp: 500
  armor: 10
  range: 12
  damage: 25

visual_asset: custom-tower-model
visual_scale: 1.0
```

**Asset Pipeline Integration**: See [Asset Pipeline](/reference/asset-pipeline) for bundle creation workflow. Bundles must be built with **Unity 2021.3.45f2**.

---

## Validation & Testing Checklist

Before shipping a pack:

```bash
# 1. Validate YAML syntax and schemas
dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack

# 2. Build and package
dotnet run --project src/Tools/PackCompiler -- build packs/my-pack

# 3. Deploy to game (optional)
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release \
  -p:DeployToGame=true

# 4. Check logs after game launch
tail -50 "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
```

---

## Tips & Gotchas

- **ID Naming**: Pack IDs and content IDs must match `^[a-z][a-z0-9-]*$` (lowercase, hyphens OK).
- **Vanilla Mapping**: Use `vanilla_mapping: <vanilla-id>` to override an existing vanilla unit. Omit it for new content.
- **Faction Colors**: Hex colors are `RRGGBB` format (no `#` prefix). Use [color picker](https://www.w3schools.com/colors/colors_converter.asp).
- **Asset References**: If a bundle doesn't exist, the unit falls back to a placeholder in-game. Check `dinoforge_debug.log` for warnings.
- **Conflicts**: If two packs define the same ID, the last loaded pack wins. Use `load_order` in `pack.yaml` to control load sequence.
- **Testing**: Use `/game-test` slash command to run automated scenario proofs.

---

## Next Steps

- **More Examples**: Browse `packs/` directory for real-world pack structures.
- **Concepts**: Learn about [Registries](/concepts/registry-system) and [ECS Bridge](/concepts/ecs-bridge).
- **Schemas**: Check all schema definitions in [Schema Reference](/reference/schemas).
