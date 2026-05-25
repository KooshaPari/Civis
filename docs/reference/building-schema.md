---
title: Building Schema Reference
description: Complete documentation of all building definition fields
---

# Building Schema Reference

This document describes every field in `building.schema.json`, the canonical schema for defining buildings and structures in DINOForge.

## Overview

Buildings are static structures placed on the map that provide resource production, troop training, research, and defensive functions. Each building is defined in a YAML file under `buildings/` in your pack.

Buildings must:
- Have a unique `id` within the pack
- Have a human-readable `display_name`
- Define their costs and production (if applicable)

## File Format and Location

**File**: `packs/<pack-id>/buildings/<building-id>.yaml`

**Example**:
```yaml
id: clone_barracks
display_name: Clone Barracks
building_type: barracks
description: Trains clone trooper infantry units

cost:
  food: 0
  wood: 100
  stone: 80
  iron: 40
  gold: 10
  population: 0

health: 300
production:
  infantry_slots: 4
  training_speed: 1.0
```

## Required Fields

### id
- **Type**: `string`
- **Min Length**: 1
- **Pattern**: `^[a-z0-9_]+$` (lowercase, underscores, no hyphens)
- **Description**: Unique building identifier within the pack
- **Example**: `clone_barracks`, `droid_factory`, `galactic_senate`

```yaml
id: clone_barracks
```

### display_name
- **Type**: `string`
- **Min Length**: 1
- **Description**: Human-readable name shown in-game (UI, menus, tooltips)
- **Example**: `Clone Barracks`, `B1 Factory`, `Droid Control Ship`

```yaml
display_name: Clone Barracks
```

## Optional Fields

### description
- **Type**: `string`
- **Description**: Flavor text or tooltip description explaining the building's purpose and effects
- **Example**: `Trains elite clone trooper units. Increases unit morale by 10%.`

```yaml
description: Trains clone trooper infantry units
```

### building_type
- **Type**: `string`
- **Description**: Functional category for organization and filtering
- **Common values** (not enforced, use what fits your game):
  - `barracks` — Unit training
  - `factory` — Unit manufacturing
  - `economy` — Resource production
  - `defense` — Defense structures
  - `research` — Technology/research
  - `command` — Command center
  - `storage` — Resource storage

```yaml
building_type: barracks
```

### cost
- **Type**: `object`
- **Description**: Resource cost to construct this building

#### cost.food
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Food cost
- **Default**: 0

#### cost.wood
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Wood cost
- **Default**: 0

#### cost.stone
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Stone cost
- **Default**: 0

#### cost.iron
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Iron cost
- **Default**: 0

#### cost.gold
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Gold cost
- **Default**: 0

#### cost.population
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Population slots required (typically 0 for buildings)
- **Default**: 0

**Example**:
```yaml
cost:
  food: 0
  wood: 100
  stone: 80
  iron: 40
  gold: 10
  population: 0
```

### health
- **Type**: `integer`
- **Minimum**: 0
- **Description**: Total hit points. Building is destroyed when health reaches 0.
- **Typical ranges**:
  - **100-200**: Light structures (walls, watchtowers)
  - **300-600**: Medium structures (barracks, factories)
  - **800+**: Heavy fortifications (fortresses, command centers)

```yaml
health: 300
```

### production
- **Type**: `object`
- **Additional Properties**: `integer`
- **Description**: Resource or unit production rates. Keys are arbitrary production types defined by your modding scheme.

**Common keys** (examples — customize as needed):
- `food_per_tick` — Food generated per game tick
- `wood_per_tick` — Wood generated per game tick
- `infantry_slots` — Maximum infantry units in training queue
- `training_speed` — Multiplier for unit training speed (1.0 = normal)
- `research_speed` — Multiplier for research speed
- `morale_bonus` — Morale boost to nearby units
- `armor_bonus` — Armor boost to nearby structures

**Example**:
```yaml
production:
  food_per_tick: 2
  wood_per_tick: 1
  infantry_slots: 4
  training_speed: 1.0
```

**Another example** (economy building):
```yaml
production:
  food_per_tick: 5
  wood_per_tick: 3
```

**Defense building example**:
```yaml
production:
  armor_bonus: 5
  morale_bonus: 10
```

## Complete Examples

### Military Building — Barracks

```yaml
id: clone_barracks
display_name: Clone Barracks
building_type: barracks
description: Trains clone trooper infantry units. Increases nearby unit morale by 15%.

cost:
  food: 0
  wood: 100
  stone: 80
  iron: 40
  gold: 10
  population: 0

health: 400

production:
  infantry_slots: 4
  training_speed: 1.0
  morale_bonus: 15
```

### Economy Building — Farm

```yaml
id: moisture_farm
display_name: Moisture Farm
building_type: economy
description: Extracts water and cultivates food. Generates consistent food supply.

cost:
  food: 0
  wood: 50
  stone: 40
  iron: 0
  gold: 0
  population: 0

health: 200

production:
  food_per_tick: 3
```

### Defense Structure — Turret

```yaml
id: combat_turret
display_name: Combat Turret
building_type: defense
description: Defensive turret with automated targeting. Guards the area.

cost:
  food: 0
  wood: 0
  stone: 60
  iron: 80
  gold: 20
  population: 0

health: 250

production:
  armor_bonus: 8
  fire_range: 10
  damage_output: 8
```

### Command Center — Senate

```yaml
id: galactic_senate
display_name: Galactic Senate
building_type: command
description: Command center for faction leadership. Increases research speed and provides vision.

cost:
  food: 0
  wood: 200
  stone: 150
  iron: 100
  gold: 50
  population: 0

health: 600

production:
  research_speed: 1.3
  vision_range: 50
  morale_bonus: 25
```

### Manufacturing — Droid Factory

```yaml
id: droid_factory
display_name: B1 Battle Droid Factory
building_type: factory
description: Mass-produces B1 battle droid units for rapid deployment.

cost:
  food: 0
  wood: 80
  stone: 100
  iron: 120
  gold: 40
  population: 0

health: 500

production:
  droid_slots: 6
  training_speed: 1.2
```

## Design Patterns

### Tier 1 (Early Game) Structure
```yaml
cost:
  wood: 50
  stone: 30
  iron: 0
  gold: 0
health: 150
production:
  basic_slots: 2
```

### Tier 2 (Mid Game) Structure
```yaml
cost:
  wood: 100
  stone: 80
  iron: 40
  gold: 10
health: 400
production:
  standard_slots: 4
  speed_bonus: 1.0
```

### Tier 3 (Late Game) Structure
```yaml
cost:
  wood: 200
  stone: 150
  iron: 100
  gold: 50
health: 800
production:
  elite_slots: 6
  speed_bonus: 1.3
  vision_range: 50
```

## Validation Rules

When validating building files, DINOForge checks:

1. **Required fields present**: `id`, `display_name`
2. **ID format**: Matches `^[a-z0-9_]+$`
3. **Type correctness**: All numbers are integers, cost is object with number values
4. **Numeric ranges**: `health` >= 0, cost values >= 0
5. **Production values**: All production values are non-negative integers

## Best Practices

- **Balanced costs**: More powerful buildings should cost more
- **Health values**: Reflect structural durability (walls are fragile, fortresses are resilient)
- **Production naming**: Use descriptive keys like `training_slots` instead of generic `value1`
- **Descriptions**: Always include flavor text describing the building's function
- **Consistency**: Use the same cost multipliers across all buildings of a tier
- **Production bonuses**: Keep bonus values reasonable (10-30% range for most modifiers)
- **Building types**: Use consistent naming across your pack for organization

## Relationship to Units

Buildings typically:
- Train units (via `infantry_slots`, `droid_slots`, etc.)
- Boost nearby unit performance (via `morale_bonus`, `armor_bonus`)
- Provide economy (via `*_per_tick` production)
- Support research and tech advancement

Reference the [Unit Schema Reference](/reference/unit-schema) to understand units trained by buildings.

## See Also

- [Unit Schema Reference](/reference/unit-schema)
- [Creating Packs](/guide/creating-packs)
- [Quick Start](/guide/quick-start)
- [Asset Pipeline](/reference/asset-pipeline) — Add 3D models to buildings
