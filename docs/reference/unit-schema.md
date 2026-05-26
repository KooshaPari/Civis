---
title: Unit Schema Reference
description: Complete documentation of all unit definition fields and properties
---

# Unit Schema Reference

This document describes every field in `unit.schema.json`, the canonical schema for defining units in DINOForge.

## Overview

Units are the core combat entities in DINO. Each unit is defined in a YAML file under `units/` in your pack. Units must:

- Have a unique `id` within the pack
- Belong to a `faction_id`
- Have a valid `unit_class` from the allowed archetypes
- Define `stats` for combat and movement

## File Format and Location

**File**: `packs/<pack-id>/units/<unit-id>.yaml`

**Example**:
```yaml
id: clone-trooper
display_name: Clone Trooper
unit_class: CoreLineInfantry
faction_id: republic
tier: 1
stats:
  hp: 100
  damage: 12
  armor: 5
  range: 0
  speed: 4
  accuracy: 0.8
  fire_rate: 1.0
  morale: 90
  cost:
    food: 50
    wood: 0
    stone: 0
    iron: 10
    gold: 5
```

## Required Fields

### id
- **Type**: `string`
- **Pattern**: `^[a-z0-9_]+$` (lowercase, underscores, no hyphens)
- **Description**: Unique unit identifier within the pack
- **Example**: `clone_trooper`, `b1_battle_droid`, `arc_trooper`

```yaml
id: clone_trooper
```

### display_name
- **Type**: `string`
- **Min Length**: 1
- **Description**: Human-readable name shown in-game (UI, tooltips, editor)
- **Example**: `Clone Trooper`, `B1 Battle Droid`

```yaml
display_name: Clone Trooper
```

### unit_class
- **Type**: `enum`
- **Required**: Yes
- **Description**: Combat archetype determining role, behavior defaults, and balance tier

**Valid values**:

| Archetype | Role | Example |
|-----------|------|---------|
| `MilitiaLight` | Basic light infantry | Militia archer |
| `CoreLineInfantry` | Standard frontline unit | Pikeman, musketeer |
| `EliteLineInfantry` | High-cost elite unit | Elite swordsman |
| `HeavyInfantry` | Heavily armored slow unit | Hoplite, cataphract |
| `Skirmisher` | Mobile ranged harasser | Archer, crossbowman |
| `AntiArmor` | Anti-vehicle specialist | Pike infantry, AT soldier |
| `ShockMelee` | High-damage melee charge unit | Berserker, paladin |
| `SwarmFodder` | Cheap disposable unit | Skeleton, swarm unit |
| `FastVehicle` | Light vehicle, high speed | Scout car, chariot |
| `MainBattleVehicle` | Armored heavy vehicle | Tank |
| `HeavySiege` | Slow siege unit | War elephant, siege tower |
| `Artillery` | Ranged support, high damage | Catapult, howitzer |
| `WalkerHeavy` | Four-legged heavy unit | War walker, dragon |
| `StaticMG` | Stationary machine gun | Pillbox, bunker |
| `StaticAT` | Stationary anti-tank | AT gun, bunker |
| `StaticArtillery` | Stationary artillery | Gun emplacement, rocket launcher |
| `SupportEngineer` | Support/repair unit | Engineer, medic |
| `Recon` | Scout/reconnaissance | Scout, spy |
| `HeroCommander` | Unique powerful unit | Hero, commander |
| `AirstrikeProxy` | Calls in aerial strikes | Air strike caller |
| `ShieldedElite` | Elite with shields | Spartan-like unit |

```yaml
unit_class: CoreLineInfantry
```

### faction_id
- **Type**: `string`
- **Min Length**: 1
- **Description**: References a faction defined in `factions/<faction-id>.yaml`. The unit belongs to this faction.
- **Example**: `republic`, `cis`, `west`

```yaml
faction_id: republic
```

## Optional Fields

### description
- **Type**: `string`
- **Description**: Flavor text or tooltip description
- **Example**: `Elite clone trooper variant with enhanced armor`

```yaml
description: Elite clone trooper variant with enhanced armor
```

### tier
- **Type**: `integer`
- **Range**: 1-3
- **Description**: Tech tier affecting unit costs and availability
  - **1**: Early game, cheap, weak
  - **2**: Mid game, moderate cost
  - **3**: Late game, expensive, strong
- **Example**: `1` for early units, `3` for late-game heroes

```yaml
tier: 1
```

### stats
- **Type**: `object`
- **Description**: Combat and movement statistics (see Stats section below)

### weapon
- **Type**: `string`
- **Description**: References a weapon ID from `weapons/<weapon-id>.yaml`. Determines attack type and damage characteristics.
- **Example**: `dc15_rifle`, `vibroblade`

```yaml
weapon: dc15_rifle
```

### defense_tags
- **Type**: `array` of strings
- **Description**: Armor/defense class. Affects damage taken and counterplay mechanics.

**Valid values**:
- `Unarmored` — No armor, takes normal damage
- `InfantryArmor` — Light armor, 10-20% damage reduction
- `HeavyArmor` — Heavy armor, 25-40% damage reduction
- `Fortified` — Built structures with fixed defenses
- `Shielded` — Unit has shield/barrier protection
- `Mechanical` — Robotic/vehicle armor
- `Biological` — Organic armor (carapace, scales)
- `Heroic` — Special hero-class resilience

```yaml
defense_tags:
  - Mechanical
  - Shielded
```

### behavior_tags
- **Type**: `array` of strings
- **Description**: Tactical behavior and role modifiers. Affects AI decisions and combat tactics.

**Valid values**:
- `HoldLine` — Defends position, stands ground
- `AdvanceFire` — Moves forward while attacking
- `Charge` — High-velocity melee assault
- `Kite` — Retreats while attacking (ranged skirmishers)
- `Swarm` — Weak individual, strong in numbers
- `SiegePriority` — Focuses on destroying buildings
- `AntiStructure` — Extra damage to buildings
- `AntiMass` — Extra damage to grouped enemies
- `AntiArmor` — Extra damage to armored targets
- `MoralePressure` — Affects enemy morale
- `Aerial` — Unit can fly (requires `aerial` section)
- `AntiAir` — Can engage aerial targets

```yaml
behavior_tags:
  - HoldLine
  - AdvanceFire
  - AntiArmor
```

### stats (Details)
- **Type**: `object`
- **Description**: Numeric combat and cost properties

#### stats.hp
- **Type**: `number`
- **Minimum**: 0
- **Description**: Hit points. Unit dies when HP reaches 0.
- **Default**: 1
- **Typical range**: 25-200 for infantry, 300+ for vehicles

```yaml
stats:
  hp: 100
```

#### stats.damage
- **Type**: `number`
- **Minimum**: 0
- **Description**: Base damage per attack
- **Default**: 0
- **Typical range**: 5-50 depending on archetype

```yaml
damage: 12
```

#### stats.armor
- **Type**: `number`
- **Minimum**: 0
- **Description**: Flat damage reduction per hit
- **Default**: 0
- **Typical range**: 0-20 for most units, up to 40 for heavy armor

```yaml
armor: 5
```

#### stats.range
- **Type**: `number`
- **Minimum**: 0
- **Description**: Attack range in world units. 0 = melee only.
- **Default**: 0
- **Typical ranges**:
  - **0**: Melee (swords, spears)
  - **1-3**: Short range (spears with reach)
  - **5-8**: Ranged (arrows, rifles)
  - **10-15**: Artillery (catapults, mortars)
  - **20+**: Long-range support (cannons)

```yaml
range: 8
```

#### stats.speed
- **Type**: `number`
- **Minimum**: 0
- **Description**: Movement speed in world units per second
- **Default**: 0
- **Typical ranges**:
  - **2-3**: Heavy, slow (tanks, elephants)
  - **3-5**: Standard infantry
  - **5-7**: Fast (skirmishers, scouts)
  - **7+**: Very fast (cavalry, motorcycles)

```yaml
speed: 4
```

#### stats.accuracy
- **Type**: `number`
- **Range**: 0.0-1.0
- **Description**: Hit chance. 1.0 = 100% never misses, 0.5 = 50% chance
- **Default**: 0.7
- **Examples**:
  - **0.9+**: Precise (archers, snipers)
  - **0.7-0.8**: Standard ranged
  - **0.6**: Less precise (siege weapons)

```yaml
accuracy: 0.85
```

#### stats.fire_rate
- **Type**: `number`
- **Minimum**: 0
- **Description**: Attacks per second
- **Default**: 1.0
- **Examples**:
  - **0.5**: Slow (heavy weapons)
  - **1.0**: Standard
  - **2.0**: Fast (swarm units)

```yaml
fire_rate: 1.2
```

#### stats.morale
- **Type**: `number`
- **Minimum**: 0
- **Description**: Starting morale value. Low morale causes rout/panic behavior.
- **Default**: 100
- **Typical range**: 50-150
  - **50-70**: Low morale, easy to break
  - **100**: Average
  - **120+**: High morale, resistant to breaks

```yaml
morale: 90
```

#### stats.cost
- **Type**: `object`
- **Description**: Resource cost to recruit this unit

##### stats.cost.food
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Food cost
- **Default**: 0

##### stats.cost.wood
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Wood cost
- **Default**: 0

##### stats.cost.stone
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Stone cost
- **Default**: 0

##### stats.cost.iron
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Iron cost
- **Default**: 0

##### stats.cost.gold
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Gold cost
- **Default**: 0

##### stats.cost.population
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Population slots required
- **Default**: 0

```yaml
stats:
  cost:
    food: 50
    wood: 10
    stone: 0
    iron: 15
    gold: 5
    population: 1
```

### visuals
- **Type**: `object`
- **Description**: Visual asset overrides (icons, portraits, 3D models, VFX)

#### visuals.icon
- **Type**: `string`
- **Description**: Icon asset path for UI selection panel
- **Example**: `assets/icons/clone_trooper.png`

#### visuals.portrait
- **Type**: `string`
- **Description**: Portrait asset path for unit info/tooltip panel
- **Example**: `assets/portraits/clone_trooper.jpg`

#### visuals.model_override
- **Type**: `string`
- **Description**: Addressables key for custom 3D model prefab
- **Example**: `sw-clone-trooper-lod0`

#### visuals.projectile_vfx
- **Type**: `string`
- **Description**: VFX effect for projectiles fired by this unit
- **Example**: `sw-blaster-bolt-effect`

#### visuals.muzzle_vfx
- **Type**: `string`
- **Description**: Muzzle flash VFX when unit fires
- **Example**: `sw-blaster-muzzle`

```yaml
visuals:
  icon: assets/icons/clone_trooper.png
  portrait: assets/portraits/clone_trooper.jpg
  model_override: sw-clone-trooper-lod0
  projectile_vfx: sw-blaster-bolt
  muzzle_vfx: sw-blaster-muzzle
```

### audio
- **Type**: `object`
- **Description**: Audio asset overrides (attack, death, selection, movement sounds)

#### audio.attack_sound
- **Type**: `string`
- **Description**: Sound effect when attacking
- **Example**: `sw-blaster-fire`

#### audio.death_sound
- **Type**: `string`
- **Description**: Sound effect when unit dies
- **Example**: `sw-soldier-death`

#### audio.select_sound
- **Type**: `string`
- **Description**: Sound effect when unit is selected
- **Example**: `sw-clone-select`

#### audio.move_sound
- **Type**: `string`
- **Description**: Sound effect when unit moves
- **Example**: `sw-march-footsteps`

```yaml
audio:
  attack_sound: sw-blaster-fire
  death_sound: sw-soldier-death
  select_sound: sw-clone-select
  move_sound: sw-march-footsteps
```

### vanilla_mapping
- **Type**: `string`
- **Description**: ID of vanilla DINO unit this replaces or maps to. Used for override packs.
- **Example**: `archer` (replaces vanilla archer), `soldier_light` (replaces light soldier)

```yaml
vanilla_mapping: archer
```

### tech_requirement
- **Type**: `string`
- **Description**: Tech node ID required to unlock this unit. References a tech tree node.
- **Example**: `tech_firearms`, `tech_armored_units`

```yaml
tech_requirement: tech_blasters
```

### visual_asset
- **Type**: `string`
- **Description**: Addressables catalog key for unit's 3D model (LOD0 prefab). Set automatically by asset pipeline; do not edit manually.
- **Example**: `sw-clone-trooper-lod0`

```yaml
visual_asset: sw-clone-trooper-lod0
```

### aerial
- **Type**: `object`
- **Description**: Aerial flight parameters. **Only applicable to units with `behavior_tags: [Aerial]`**. See [Aviation Guide](/concepts/aviation) for full details.

#### aerial.cruise_altitude
- **Type**: `number`
- **Minimum**: 0
- **Description**: Operational altitude in world units (Y-axis) during normal flight
- **Typical values**:
  - **10-15**: Low flight (helicopters, balloons)
  - **15-25**: Medium flight (fighters, bombers)
  - **25+**: High altitude (interceptors)

```yaml
aerial:
  cruise_altitude: 20
```

#### aerial.ascend_speed
- **Type**: `number`
- **Minimum**: 0
- **Description**: Rate of altitude gain in world units per second
- **Typical range**: 3-8

```yaml
ascend_speed: 5
```

#### aerial.descend_speed
- **Type**: `number`
- **Minimum**: 0
- **Description**: Rate of altitude loss in world units per second (when attacking/landing)
- **Typical range**: 2-5

```yaml
descend_speed: 3
```

#### aerial.anti_air
- **Type**: `boolean`
- **Description**: Whether this aerial unit can engage other aerial targets
- **Example**: `true` for interceptor, `false` for transport

```yaml
aerial:
  cruise_altitude: 20
  ascend_speed: 5
  descend_speed: 3
  anti_air: true
```

## Complete Example

```yaml
id: clone_commander
display_name: Clone Commander
description: Elite clone trooper commander with superior equipment

unit_class: HeroCommander
faction_id: republic
tier: 3

stats:
  hp: 150
  damage: 18
  armor: 8
  range: 6
  speed: 4.5
  accuracy: 0.9
  fire_rate: 1.3
  morale: 120
  cost:
    food: 100
    wood: 20
    stone: 0
    iron: 40
    gold: 30
    population: 2

weapon: dc15_rifle_enhanced

defense_tags:
  - HeavyArmor
  - Mechanical

behavior_tags:
  - HoldLine
  - AdvanceFire
  - MoralePressure

visuals:
  icon: assets/icons/clone_commander.png
  portrait: assets/portraits/clone_commander.jpg
  model_override: sw-clone-commander-lod0
  projectile_vfx: sw-blaster-enhanced
  muzzle_vfx: sw-blaster-muzzle-enhanced

audio:
  attack_sound: sw-blaster-fire-enhanced
  death_sound: sw-commander-death
  select_sound: sw-clone-commander-select
  move_sound: sw-march-footsteps-elite

vanilla_mapping: elite_soldier
tech_requirement: tech_elite_units
visual_asset: sw-clone-commander-lod0
```

## Validation Rules

When validating unit files, DINOForge checks:

1. **Required fields present**: `id`, `display_name`, `unit_class`, `faction_id`
2. **ID format**: Matches `^[a-z0-9_]+$`
3. **Type correctness**: All numbers are numbers, all arrays are arrays
4. **Enum values**: `unit_class` is in allowed list, tags are valid
5. **Numeric ranges**: `tier` is 1-3, `accuracy` is 0.0-1.0
6. **Reference integrity**: `faction_id`, `weapon` reference actual definitions
7. **Aerial consistency**: If `behavior_tags` contains `Aerial`, `aerial` block must exist

## Best Practices

- Keep costs balanced relative to unit strength (stronger = more expensive)
- Use meaningful `description` values for player understanding
- Include both `attack_sound` and `death_sound` for audio feedback
- Set `morale` based on unit type (heroes higher, swarm lower)
- Use `vanilla_mapping` for override packs to ensure correct unit replacement
- For aerial units, ensure `cruise_altitude` > movement height obstacles
- Define complete `stats.cost` even if some resources are 0

## See Also

- [Building Schema Reference](/reference/building-schema)
- [Aviation Guide](/concepts/aviation) — Detailed aerial unit mechanics
- [Warfare Overview](/warfare/overview) — Unit archetypes and roles
- [Creating Packs](/guide/creating-packs) — Full pack authoring
